use std::process::Command;
use std::path::Path;
use std::fs;

fn main() {
    // Re-run the build if stub.rs changes
    println!("cargo:rerun-if-changed=src/stub.rs");

    // Only build the stub if the build-stub feature is NOT enabled
    if std::env::var("CARGO_FEATURE_BUILD_STUB").is_err() {
        let status = Command::new("cargo")
            .args(&["build", "--bin", "stub", "--features", "build-stub"])
            .status()
            .expect("Failed to execute cargo build for stub");

        if !status.success() {
            panic!("Failed to build stub binary");
        }
    }

    // Copy the stub binary to a known location
    let stub_binary = "target/debug/stub";
    let stub_output = "target/stub.bin";

    if Path::new(stub_binary).exists() {
        fs::copy(stub_binary, stub_output)
            .expect("Failed to copy stub binary to target/stub.bin");
    } else {
        panic!("Stub binary not found at {}", stub_binary);
    }

    // Re-run the build if the stub binary changes
    println!("cargo:rerun-if-changed={}", stub_output);
}
