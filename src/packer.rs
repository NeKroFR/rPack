use std::env;
use std::fs::File;
use std::io::{Read, Write};
use std::path::Path;
use std::os::unix::fs::PermissionsExt;
use lz4_flex::compress;  // Import LZ4 compression function

fn main() {
    // Parse command-line arguments
    let args: Vec<String> = env::args().collect();
    if args.len() != 3 {
        eprintln!("Usage: {} <input_binary> <output_packed_binary>", args[0]);
        std::process::exit(1);
    }
    let input_path = &args[1];
    let output_path = &args[2];

    // Path to the precompiled stub binary
    let stub_path = "target/debug/stub";

    // Read the stub binary
    let mut stub_file = File::open(stub_path).expect("Failed to open stub binary");
    let mut stub_data = Vec::new();
    stub_file.read_to_end(&mut stub_data).expect("Failed to read stub binary");

    // Read the input binary
    let mut input_file = File::open(input_path).expect("Failed to open input binary");
    let mut input_data = Vec::new();
    input_file.read_to_end(&mut input_data).expect("Failed to read input binary");

    // Compress the input binary with LZ4
    let compressed_data = compress(&input_data);
    let compressed_size = compressed_data.len() as u64;  // Size of compressed data
    let decompressed_size = input_data.len() as u64;     // Original size

    // Construct the packed binary: stub + compressed_data + compressed_size + decompressed_size
    let mut packed_data = stub_data;
    packed_data.extend_from_slice(&compressed_data);
    packed_data.extend_from_slice(&compressed_size.to_le_bytes());
    packed_data.extend_from_slice(&decompressed_size.to_le_bytes());

    // Write the packed binary to the output file
    let mut output_file = File::create(output_path).expect("Failed to create output file");
    output_file.write_all(&packed_data).expect("Failed to write packed binary");

    // Set the output file as executable
    let mut permissions = output_file.metadata().expect("Failed to get metadata").permissions();
    permissions.set_mode(0o755); // rwxr-xr-x
    std::fs::set_permissions(output_path, permissions).expect("Failed to set permissions");

    println!("Packed binary created at {}", output_path);
}
