use std::process::Command;
use std::fs;

fn main() {
    // Tell Cargo to re-run if stub source changes
    println!("cargo:rerun-if-changed=../stub/src/main.rs");

    // Build the stub crate in release mode
    let status = Command::new("cargo")
        .args(&["build", "--release", "--manifest-path", "../stub/Cargo.toml"])
        .status()
        .expect("Failed to build stub");

    if !status.success() {
        panic!("Failed to build stub binary");
    }

    // Copy the stub binary to a known location in the shared target directory
    fs::copy("../target/release/stub", "../target/stub.bin")
        .expect("Failed to copy stub binary");

    // Re-run if the stub binary changes
    println!("cargo:rerun-if-changed=../target/release/stub");
}

