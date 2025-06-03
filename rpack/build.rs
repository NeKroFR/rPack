use std::process::Command;
use std::fs;

fn main() {
    println!("cargo:rerun-if-changed=../stub/src/main.rs");

    // Build the stub with release optimizations
    let build_status = Command::new("cargo")
        .args(&[
            "build",
            "--release",
            "--manifest-path",
            "../stub/Cargo.toml",
        ])
        .env("RUSTFLAGS", "-C lto=yes -C codegen-units=1 -C debuginfo=0")
        .status()
        .expect("Failed to build stub");

    if !build_status.success() {
        panic!("Failed to build stub binary");
    }

    // Copy the binary
    fs::copy("../target/release/stub", "../target/stub.bin")
        .expect("Failed to copy stub binary");

    // Strip the binary
    let strip_status = Command::new("strip")
        .arg("../target/stub.bin")
        .status()
        .expect("Failed to execute strip command");

    if !strip_status.success() {
        panic!("Failed to strip stub binary");
    }

    println!("cargo:rerun-if-changed=../target/release/stub");
}
