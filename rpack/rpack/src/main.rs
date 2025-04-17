use std::env;
use std::fs::File;
use std::io::{Read, Write};
use std::path::Path;
use std::os::unix::fs::PermissionsExt;
use lz4_flex::compress;
use aes::AES128;
use whitebox::{create_whitebox, encrypt_func};
use ndarray::Array1;
use bincode;

const STUB_DATA: &[u8] = include_bytes!("../../target/stub.bin");

fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() != 3 {
        eprintln!("Usage: {} <input_binary> <output_packed_binary>", args[0]);
        std::process::exit(1);
    }
    let input_path = &args[1];
    let output_path = &args[2];

    let mut input_file = File::open(input_path).expect("Failed to open input binary");
    let mut input_data = Vec::new();
    input_file.read_to_end(&mut input_data).expect("Failed to read input binary");

    if input_data.len() < 4 || &input_data[0..4] != b"\x7FELF" {
        eprintln!("Error: Input file is not a valid ELF binary");
        std::process::exit(1);
    }

    println!("[*] Generating white-box data...");
    let (pub_enc_data, white_data) = create_whitebox();

    println!("[*] Generating AES key...");
    let aes_key = AES128::generate_key();


    let aes_key_bits: Vec<i64> = aes_key.iter()
        .flat_map(|&byte| (0..8).map(move |i| ((byte >> i) & 1) as i64))
        .collect();
    let mut message_padded = vec![0i64; pub_enc_data.degree];
    message_padded[0..128].copy_from_slice(&aes_key_bits);
    let message_array = Array1::from_vec(message_padded);

    println!("[*] Encrypting AES key with white-box...");
    let (a1, a2) = encrypt_func(&message_array, &pub_enc_data.pka, &pub_enc_data.pkb, pub_enc_data.degree, pub_enc_data.modulus);

    // Serialize whitebox data and encrypted key
    let serialized_white_data = bincode::serialize(&white_data).expect("Failed to serialize WhiteData");
    let serialized_a1 = bincode::serialize(&a1).expect("Failed to serialize a1");
    let serialized_a2 = bincode::serialize(&a2).expect("Failed to serialize a2");

    println!("[*] Compressing input binary...");
    let aes = AES128::new(&aes_key);
    let compressed_data = compress(&input_data);

    println!("[*] Encrypting compressed data...");
    let padded_data = aes::pad_pkcs7(&compressed_data, 16);
    let encrypted_data = (aes.encrypt)(&aes, &padded_data);
    let encrypted_size = encrypted_data.len() as u64;
    let decompressed_size = input_data.len() as u64;

    println!("[*] Generating the packed binary...");
    let mut packed_data = STUB_DATA.to_vec();
    packed_data.extend_from_slice(&encrypted_data);
    packed_data.extend_from_slice(&serialized_a1);
    packed_data.extend_from_slice(&serialized_a2);
    packed_data.extend_from_slice(&serialized_white_data);
    packed_data.extend_from_slice(&encrypted_size.to_le_bytes());
    packed_data.extend_from_slice(&(serialized_a1.len() as u64).to_le_bytes());
    packed_data.extend_from_slice(&(serialized_a2.len() as u64).to_le_bytes());
    packed_data.extend_from_slice(&(serialized_white_data.len() as u64).to_le_bytes());
    packed_data.extend_from_slice(&decompressed_size.to_le_bytes());

    let mut output_file = File::create(output_path).expect("Failed to create output file");
    output_file.write_all(&packed_data).expect("Failed to write packed binary");

    let mut permissions = output_file.metadata().expect("Failed to get metadata").permissions();
    permissions.set_mode(0o755); // rwxr-xr-x
    std::fs::set_permissions(output_path, permissions).expect("Failed to set permissions");

    println!("Packed binary created at {}", output_path);
}
