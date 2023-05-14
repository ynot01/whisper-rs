#![allow(clippy::uninlined_format_args)]

extern crate bindgen;

use std::env;
use std::path::PathBuf;

fn main() {
    let target = env::var("TARGET").unwrap();
    // Link C++ standard library
    if let Some(cpp_stdlib) = get_cpp_link_stdlib(&target) {
        println!("cargo:rustc-link-lib=dylib={}", cpp_stdlib);
    }
    // Link macOS Accelerate framework for matrix calculations
    if target.contains("apple") {
        println!("cargo:rustc-link-lib=framework=Accelerate");
        #[cfg(feature = "coreml")]
        {
            println!("cargo:rustc-link-lib=framework=Foundation");
            println!("cargo:rustc-link-lib=framework=CoreML");
        }
    }

    println!("cargo:rustc-link-search={}", env::var("OUT_DIR").unwrap());
    println!("cargo:rustc-link-lib=static=whisper");
    #[cfg(feature = "coreml")]
    println!("cargo:rustc-link-lib=static=whisper.coreml");
    println!("cargo:rerun-if-changed=wrapper.h");

    if env::var("WHISPER_DONT_GENERATE_BINDINGS").is_ok() {
        let _: u64 = std::fs::copy(
            "src/bindings.rs",
            env::var("OUT_DIR").unwrap() + "/bindings.rs",
        )
        .expect("Failed to copy bindings.rs");
    } else {
        let bindings = bindgen::Builder::default()
            .header("wrapper.h")
            .clang_arg("-I./whisper.cpp")
            .parse_callbacks(Box::new(bindgen::CargoCallbacks))
            .generate();

        match bindings {
            Ok(b) => {
                let out_path = PathBuf::from(env::var("OUT_DIR").unwrap());
                b.write_to_file(out_path.join("bindings.rs"))
                    .expect("Couldn't write bindings!");
            }
            Err(e) => {
                println!("cargo:warning=Unable to generate bindings: {}", e);
                println!("cargo:warning=Using bundled bindings.rs, which may be out of date");
                // copy src/bindings.rs to OUT_DIR
                std::fs::copy(
                    "src/bindings.rs",
                    env::var("OUT_DIR").unwrap() + "/bindings.rs",
                )
                .expect("Unable to copy bindings.rs");
            }
        }
    };

    // stop if we're on docs.rs
    if env::var("DOCS_RS").is_ok() {
        return;
    }

    // build libwhisper.a
    env::set_current_dir("whisper.cpp").expect("Unable to change directory to whisper.cpp");
    _ = std::fs::remove_dir_all("build");
    _ = std::fs::create_dir("build");
    env::set_current_dir("build").expect("Unable to change directory to whisper.cpp build");

    #[cfg(feature = "coreml")]
    let code = std::process::Command::new("cmake")
        .arg("..")
        .arg("-DCMAKE_BUILD_TYPE=Release")
        .arg("-DBUILD_SHARED_LIBS=OFF")
        .arg("-DWHISPER_ALL_WARNINGS=OFF")
        .arg("-DWHISPER_ALL_WARNINGS_3RD_PARTY=OFF")
        .arg("-DWHISPER_BUILD_TESTS=OFF")
        .arg("-DWHISPER_BUILD_EXAMPLES=OFF")
        .arg("-DWHISPER_COREML=1")
        .arg("-DWHISPER_COREML_ALLOW_FALLBACK=1")
        .status()
        .expect("Failed to generate build script");

    #[cfg(not(feature = "coreml"))]
    let code = std::process::Command::new("cmake")
        .arg("..")
        .arg("-DCMAKE_BUILD_TYPE=Release")
        .arg("-DBUILD_SHARED_LIBS=OFF")
        .arg("-DWHISPER_ALL_WARNINGS=OFF")
        .arg("-DWHISPER_ALL_WARNINGS_3RD_PARTY=OFF")
        .arg("-DWHISPER_BUILD_TESTS=OFF")
        .arg("-DWHISPER_BUILD_EXAMPLES=OFF")
        .status()
        .expect("Failed to generate build script");
    if code.code() != Some(0) {
        panic!("Failed to generate build script");
    }

    let code = std::process::Command::new("cmake")
        .arg("--build")
        .arg(".")
        .arg("--config")
        .arg("Release")
        .status()
        .expect("Failed to build libwhisper.a");
    if code.code() != Some(0) {
        panic!("Failed to build libwhisper.a");
    }

    // move libwhisper.a to where Cargo expects it (OUT_DIR)
    #[cfg(target_os = "windows")]
    {
        std::fs::copy(
            "Release/whisper.lib",
            format!("{}/whisper.lib", env::var("OUT_DIR").unwrap()),
        )
        .expect("Failed to copy libwhisper.lib");
    }

    #[cfg(not(target_os = "windows"))]
    {
        std::fs::copy(
            "libwhisper.a",
            format!("{}/libwhisper.a", env::var("OUT_DIR").unwrap()),
        )
        .expect("Failed to copy libwhisper.a");
    }
    #[cfg(feature = "coreml")]
    #[cfg(not(target_os = "windows"))]
    {
        std::fs::copy(
            "libwhisper.coreml.a",
            format!("{}/libwhisper.coreml.a", env::var("OUT_DIR").unwrap()),
        )
        .expect("Failed to copy libwhisper.coreml.a");
    }

    // clean the whisper build directory to prevent Cargo from complaining during crate publish
    env::set_current_dir("..").expect("Unable to change directory to whisper.cpp");
    _ = std::fs::remove_dir_all("build");
    // for whatever reason this file is generated during build and triggers cargo complaining
    _ = std::fs::remove_file("bindings/javascript/package.json");
}

// From https://github.com/alexcrichton/cc-rs/blob/fba7feded71ee4f63cfe885673ead6d7b4f2f454/src/lib.rs#L2462
fn get_cpp_link_stdlib(target: &str) -> Option<&'static str> {
    if target.contains("msvc") {
        None
    } else if target.contains("apple") || target.contains("freebsd") || target.contains("openbsd") {
        Some("c++")
    } else if target.contains("android") {
        Some("c++_shared")
    } else {
        Some("stdc++")
    }
}
