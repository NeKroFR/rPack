use std::env;
use std::fs::File;
use std::io::{Read, Write};
use std::path::Path;
use std::os::unix::fs::PermissionsExt;
use lz4_flex::compress;

const STUB_DATA: &[u8] = include_bytes!("../../target/stub.bin");

fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() != 3 {
        eprintln!("Usage: {} <input_binary> <output_packed_binary>", args[0]);
        std::process::exit(1);
    }
    let input_path = &args[1];
    let output_path = &args[2];

    let mut packed_data = Vec::from(STUB_DATA);
    let mut input_file = File::open(input_path).expect("Failed to open input binary");
    let mut input_data = Vec::new();
    input_file.read_to_end(&mut input_data).expect("Failed to read input binary");

    let compressed_data = compress(&input_data);
    let compressed_size = compressed_data.len() as u64;
    let decompressed_size = input_data.len() as u64;

    packed_data.extend_from_slice(&compressed_data);
    packed_data.extend_from_slice(&compressed_size.to_le_bytes());
    packed_data.extend_from_slice(&decompressed_size.to_le_bytes());

    let mut output_file = File::create(output_path).expect("Failed to create output file");
    output_file.write_all(&packed_data).expect("Failed to write packed binary");

    let mut permissions = output_file.metadata().expect("Failed to get metadata").permissions();
    permissions.set_mode(0o755); // rwxr-xr-x
    std::fs::set_permissions(output_path, permissions).expect("Failed to set permissions");

    println!("Packed binary created at {}", output_path);
}

