use std::collections::{HashMap, hash_map::Entry};

use cargo_metadata::Message;

#[derive(Debug, Clone, Copy)]
pub enum CrateTypes {
    Shared,
    Static,
    Both,
}

#[derive(Debug, Clone)]
pub struct BuildConfiguration {
    pub triple: String,
    pub profile: String,
    pub features: Vec<String>,
    pub crate_types: CrateTypes,
}

#[derive(Debug, Clone, Copy)]
pub enum LibraryKind {
    Shared,
    Static,
}

#[derive(Debug, Clone, Copy)]
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
    paths: Vec<(std::path::PathBuf, ArtifactRole)>,
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
            // It actually looks like path+file:///C:/Users/alexandre/Projects/oribotics/festival/rtori/rtori-core-ffi#0.1.0
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
        Ok(BuildOutput { paths: files })
    } else {
        Err(BuildError::ExitStatus(output))
    }
}

fn generate_cmake() {
    let header = r#"
cmake_minimum_required(VERSION 3.21...3.31)
set(prefix {{prefix}})

add_library(rtori_core::Headers INTERFACE IMPORTED)
set_target_properties(rtori_core::Headers PROPERTIES
    INTERFACE_INCLUDE_DIRECTORIES "${prefix}/headers/c"
)

add_library(rtori_core::Shared SHARED IMPORTED)
set_target_properties(rtori_core::Shared PROPERTIES
    IMPORTED_LOCATION "${prefix}/shared/rtori_core.dll"
    INTERFACE_LINK_LIBRARIES rtori_core::Headers
)
if(WIN32)
    set_target_properties(rtori_core::Shared PROPERTIES
        IMPORTED_IMPLIB "${prefix}/shared/rtori_core.dll.lib"
    )
endif()

add_library(rtori_core::Static STATIC IMPORTED)
set_target_properties(rtori_core::Shared PROPERTIES
    IMPORTED_LOCATION "${prefix}/static/rtori_core.lib"
    INTERFACE_LINK_LIBRARIES rtori_core::Headers 
    INTERFACE_COMPILE_DEFINITIONS "-DRTORI_STATIC"
)
"#;

    println!("{}", header);
}

fn main() {
    let outputs = build(BuildConfiguration {
        triple: String::from("x86_64-pc-windows-msvc"),
        profile: String::from("release"),
        features: Vec::new(),
        crate_types: CrateTypes::Both,
    })
    .unwrap();

    println!("outputs: {outputs:?}");

    /* Copying the files */
    std::fs::create_dir_all("output/headers/c/rtori/").unwrap();
    std::fs::create_dir_all("output/shared").unwrap();
    std::fs::create_dir_all("output/static").unwrap();

    outputs.paths.iter().for_each(|(path, kind)| {
        let dst = match kind {
            ArtifactRole::Archive | ArtifactRole::AssociatedWith(LibraryKind::Static) => {
                std::path::PathBuf::from("output/static/").join(path.file_name().unwrap())
            }
            ArtifactRole::SharedLibrary
            | ArtifactRole::DebugSymbols
            | ArtifactRole::ImportLibrary
            | ArtifactRole::AssociatedWith(LibraryKind::Shared) => {
                std::path::PathBuf::from("output/shared/").join(path.file_name().unwrap())
            }
        };

        println!("Copying {:?} to {:?}", path, dst);
        std::fs::copy(path, dst).unwrap();
    });

    /* Header generation */
    let mut command = std::process::Command::new("diplomat-tool")
        .args(&[
            "cpp",
            "output/headers/cpp/rtori",
            "--entry",
            "rtori-core-ffi/src/lib.rs",
        ])
        .spawn()
        .unwrap();

    let mut command = std::process::Command::new("diplomat-tool")
        .args(&[
            "c",
            "output/headers/c/rtori",
            "--entry",
            "rtori-core-ffi/src/lib.rs",
        ])
        .spawn()
        .unwrap();

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
}
