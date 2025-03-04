use std::{
    collections::{hash_map::Entry, HashMap},
    io::Write,
};

use cargo_metadata::Message;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum CrateTypes {
    Shared,
    Static,
    Both,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct BuildConfiguration {
    pub triple: String,
    pub profile: String,
    pub features: Vec<String>,
    pub crate_types: CrateTypes,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum LibraryKind {
    Shared,
    Static,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum ArtifactRole {
    /// .lib on windows, .a otherwise
    Archive,
    SharedLibrary,
    /// Only on windows, goes with the shared library
    ImportLibrary,
    /// .pdb on windows, possible .sym on unix
    DebugSymbols,
    AssociatedWith(LibraryKind),
}

#[derive(Debug, Clone)]
pub struct BuildOutput {
    pub config: BuildConfiguration,
    pub paths: Vec<(std::path::PathBuf, ArtifactRole)>,
}

impl BuildOutput {
    pub fn platform(&self) -> Option<&'static platforms::Platform> {
        platforms::Platform::find(&self.config.triple)
    }

    pub fn get(&self, role: ArtifactRole) -> Option<&std::path::Path> {
        self.paths.iter().find_map(|(path, candidate_role)| {
            if candidate_role == &role {
                Some(path.as_path())
            } else {
                None
            }
        })
    }
}

fn contains_subslice<T: PartialEq>(data: &[T], needle: &[T]) -> bool {
    data.windows(needle.len()).any(|w| w == needle)
}

pub fn kind_for_path<P: AsRef<std::path::Path>>(path: P) -> Option<ArtifactRole> {
    let path = path.as_ref();
    let base = path.file_name()?;
    let base_lower = base.to_ascii_lowercase();
    let base_bytes = base_lower.as_encoded_bytes();

    if base_bytes.ends_with(b".dll")
        || base_bytes.ends_with(b".so")
        || base_bytes.ends_with(b".dylib")
        || base_bytes.ends_with(b".wasm")
    {
        return Some(ArtifactRole::SharedLibrary);
    }

    if base_bytes.ends_with(b".imp")
        || base_bytes.ends_with(b".dll.imp")
        || base_bytes.ends_with(b".dll.lib")
        || base_bytes.ends_with(b".dll.a")
    {
        return Some(ArtifactRole::ImportLibrary);
    }

    if base_bytes.ends_with(b".pdb") || base_bytes.ends_with(b".sym") {
        return Some(ArtifactRole::DebugSymbols);
    }

    if base_bytes.ends_with(b".lib") || base_bytes.ends_with(b".a") {
        return Some(ArtifactRole::Archive);
    }

    if contains_subslice(base_bytes, b".dll.")
        || contains_subslice(base_bytes, b".so.")
        || contains_subslice(base_bytes, b".dylib.")
    {
        return Some(ArtifactRole::AssociatedWith(LibraryKind::Shared));
    } else {
        return Some(ArtifactRole::AssociatedWith(LibraryKind::Static));
    }
}

#[derive(Debug)]
pub enum BuildError {
    NoMatchingArtifactMessage,
    CouldNotGetExitStatus(std::io::Error),
    ExitStatus(std::process::ExitStatus),
}

fn build(b: BuildConfiguration) -> Result<BuildOutput, BuildError> {
    let mut command = std::process::Command::new("cargo")
        .args(&[
            "build",
            "--package",
            "rtori-core-ffi",
            "--message-format=json-render-diagnostics",
            "--target",
            &b.triple,
            "--profile",
            &b.profile,
        ])
        .stdout(std::process::Stdio::piped())
        .spawn()
        .unwrap();

    let reader = std::io::BufReader::new(command.stdout.take().unwrap());

    let stream = cargo_metadata::Message::parse_stream(reader);
    let artifacts = stream
        .filter_map(|message| match message.unwrap() {
            // It actually looks like path+file:///C:/Users/alexandre/Projects/oribotics/festival/rtori/core-ffi#0.1.0
            // TODO: Make it resolve & check rather than do this hack
            Message::CompilerArtifact(artifact)
                if artifact.package_id.repr.contains("rtori-core-ffi") =>
            {
                Some(artifact)
            }
            _ => None,
        })
        .last()
        .ok_or(BuildError::NoMatchingArtifactMessage)?;

    let files = artifacts
        .filenames
        .into_iter()
        .filter_map(|filename| {
            let kind = kind_for_path(&filename);
            kind.map(|kind| (filename.into_std_path_buf(), kind))
        })
        .collect();

    let output = command
        .wait()
        .map_err(|e| BuildError::CouldNotGetExitStatus(e))?;
    if output.success() {
        Ok(BuildOutput {
            config: b,
            paths: files,
        })
    } else {
        Err(BuildError::ExitStatus(output))
    }
}

fn generate_cmake<P: AsRef<std::path::Path>>(outputs: &BuildOutput, output: P) {
    #[derive(askama::Template)]
    #[template(path = "../resources/CMakeLists.txt.in", escape = "none")]
    struct Template<'a> {
        target_triple: &'a str,
        target_arch: &'a str,
        target_os: &'a str,
        target_env: Option<&'a str>,
        static_path_rel: Option<&'a typed_path::UnixPath>,
        shared_path_rel: Option<&'a typed_path::UnixPath>,
    }

    let shared_path_rel = outputs.get(ArtifactRole::SharedLibrary).map(|path| {
        typed_path::UnixPath::new("shared").join(path.file_name().unwrap().as_encoded_bytes())
    });
    let static_path_rel = outputs.get(ArtifactRole::Archive).map(|path| {
        typed_path::UnixPath::new("static").join(path.file_name().unwrap().as_encoded_bytes())
    });
    let platform = outputs.platform().unwrap();
    let template = Template {
        target_triple: &outputs.config.triple,
        target_arch: platform.target_arch.as_str(),
        target_os: platform.target_os.as_str(),
        target_env: if let platforms::Env::None = platform.target_env {
            None
        } else {
            Some(platform.target_env.as_str())
        },
        shared_path_rel: shared_path_rel.as_ref().map(|v| v.as_path()),
        static_path_rel: static_path_rel.as_ref().map(|v| v.as_path()),
    };

    let mut dest = std::fs::OpenOptions::new()
        .truncate(true)
        .write(true)
        .create(true)
        .open(&output)
        .expect(&format!(
            "couldn't open <{:?}> for writing",
            output.as_ref()
        ));

    use askama::Template as _;
    let str = template.render().unwrap();
    dest.write_all(str.as_bytes()).unwrap();
}

#[derive(clap::Parser, Debug)]
struct Args {
    #[arg(short, long, default_value_t = target_triple::HOST.into())]
    target: String,

    #[arg(short, long, default_value_t = String::from("release"))]
    profile: String,

    /// Directory where the output should be generated
    #[arg(short, long, default_value = "./output")]
    output: std::path::PathBuf,
}

fn main() {
    let args = <Args as clap::Parser>::parse();

    let outputs = build(BuildConfiguration {
        triple: args.target,
        profile: args.profile,
        features: Vec::new(),
        crate_types: CrateTypes::Both,
    })
    .unwrap();

    println!("outputs: {outputs:?}");

    /* Copying the files */
    let headers_dir = args.output.join("headers");
    std::fs::create_dir_all(&headers_dir).unwrap();

    let shared_dir = args.output.join("shared");
    std::fs::create_dir_all(&shared_dir).unwrap();

    let static_dir = args.output.join("static");
    std::fs::create_dir_all(&static_dir).unwrap();

    outputs.paths.iter().for_each(|(path, kind)| {
        let dst = match kind {
            ArtifactRole::Archive | ArtifactRole::AssociatedWith(LibraryKind::Static) => {
                static_dir.join(path.file_name().unwrap())
            }
            ArtifactRole::SharedLibrary
            | ArtifactRole::DebugSymbols
            | ArtifactRole::ImportLibrary
            | ArtifactRole::AssociatedWith(LibraryKind::Shared) => {
                shared_dir.join(path.file_name().unwrap())
            }
        };

        println!("Copying {:?} to {:?}", path, dst);
        std::fs::copy(path, dst).unwrap();
    });

    /* Header generation */
    let mut c_bindings = std::process::Command::new("diplomat-tool")
        .args(&[
            "cpp".into(),
            headers_dir.join("cpp"),
            "--entry".into(),
            "core/core-ffi/src/lib.rs".into(),
        ])
        .spawn()
        .unwrap();

    let mut cpp_bindings = std::process::Command::new("diplomat-tool")
        .args(&[
            "c".into(),
            headers_dir.join("c/rtori"),
            "--entry".into(),
            "core/core-ffi/src/lib.rs".into(),
        ])
        .spawn()
        .unwrap();

    generate_cmake(&outputs, args.output.join("CMakeLists.txt"));
    /* CPS output */

    let cps = cps_deps::cps::Package {
        name: String::from("rtori_core"),
        cps_version: String::from("0.13.0"),
        platform: Some(cps_deps::cps::Platform {
            kernel: Some(String::from("windows")),
            isa: Some(String::from("x86_64")),
            c_runtime_vendor: Some(String::from("microsoft")),
            ..Default::default()
        }),
        configurations: Some(vec![String::from("release"), String::from("debug")]),
        components: HashMap::from_iter(vec![(
            String::from("rtori-core-static"),
            cps_deps::cps::MaybeComponent::Component(cps_deps::cps::Component::Archive(
                cps_deps::cps::ComponentFields {
                    location: Some(String::from("potato")),
                    ..Default::default()
                },
            )),
        )]),
        ..Default::default()
    };

    let cps_str = serde_json::to_string_pretty(&cps).unwrap();
    println!("cps: {cps_str}");

    c_bindings.wait().unwrap();
    cpp_bindings.wait().unwrap();
}
