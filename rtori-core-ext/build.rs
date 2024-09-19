use cbindgen::Language;
use std::env;

fn main() {
    println!("cargo:rerun-if-changed=src/lib.rs");

    let manifest_dir = env::var("CARGO_MANIFEST_DIR").unwrap();

    // C
    {
        let config = cbindgen::Config::from_file("cbindgen_c.toml").unwrap();
        println!("cargo:rerun-if-changed=cbindgen_c.toml");

        cbindgen::Builder::new()
            .with_crate(&manifest_dir)
            .with_language(Language::C)
            .with_config(config)
            .generate()
            .expect("Unable to generate C bindings")
            .write_to_file("target/rtori_core.h");
    }

    // C++
    {
        let config = cbindgen::Config::from_file("cbindgen_cpp.toml").unwrap();
        println!("cargo:rerun-if-changed=cbindgen_cpp.toml");

        cbindgen::Builder::new()
            .with_crate(&manifest_dir)
            .with_language(Language::Cxx)
            .with_config(config)
            .generate()
            .expect("Unable to generate C++ bindings")
            .write_to_file("target/rtori_core.hpp");
    }
}