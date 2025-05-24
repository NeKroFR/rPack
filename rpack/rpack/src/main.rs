use std::env;
use std::fs::File;
use std::io::{Read, Write};
use std::os::unix::fs::PermissionsExt;
use aes::AES128;
use lz4_flex::compress;
use whitebox::{create_whitebox, encrypt_func};
use checksum::{compute_blake3, hash_to_hex};
use ndarray::Array1;
use bincode;

const STUB_DATA: &[u8] = include_bytes!("../../target/stub.bin");

fn validate_elf(data: &[u8]) -> bool {
    data.len() >= 4 && &data[0..4] == b"\x7FELF"
}

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

    if  !validate_elf(&input_data) {
        eprintln!("Error: Input file is not a valid ELF binary");
        std::process::exit(1);
    }

    if input_data.len() < 100 {
        eprintln!("Warning: Input file is very small, ensuring minimum size");
        // Pad to ensure minimum size
        while input_data.len() < 100 {
            input_data.push(0);
        }
    }

    println!("[*] Computing original binary checksum...");
    let original_hash = compute_blake3(&input_data);
    
    println!("[*] Generating white-box data...");
    let (pub_enc_data, white_data) = create_whitebox();

    println!("[*] Generating AES key...");
    let aes_key = AES128::generate_key();
    let aes_key_hash = compute_blake3(&aes_key);

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
    let compressed_data = compress(&input_data);
    
    println!("[*] Computing compressed data checksum...");
    let compressed_hash = compute_blake3(&compressed_data);

    println!("[*] Encrypting compressed data...");
    let aes = AES128::new(&aes_key);
    let padded_data = aes::pad_pkcs7(&compressed_data, 16);
    let encrypted_data = (aes.encrypt)(&aes, &padded_data);
    let encrypted_size = encrypted_data.len() as u64;
    let decompressed_size = input_data.len() as u64;

    println!("[*] Generating the packed binary...");
    let mut packed_data = STUB_DATA.to_vec();
    
    // Organize data sections consistently
    // Format: [STUB] [encrypted_data] [a1] [a2] [white_data] [sizes] [checksums] [final_checksum]
    packed_data.extend_from_slice(&encrypted_data);
    packed_data.extend_from_slice(&serialized_a1);
    packed_data.extend_from_slice(&serialized_a2);
    packed_data.extend_from_slice(&serialized_white_data);
    
    // Add size fields
    packed_data.extend_from_slice(&encrypted_size.to_le_bytes());
    packed_data.extend_from_slice(&(serialized_a1.len() as u64).to_le_bytes());
    packed_data.extend_from_slice(&(serialized_a2.len() as u64).to_le_bytes());
    packed_data.extend_from_slice(&(serialized_white_data.len() as u64).to_le_bytes());
    packed_data.extend_from_slice(&decompressed_size.to_le_bytes());
    
    // Add checksums - now with Blake3 only
    packed_data.extend_from_slice(&original_hash);
    packed_data.extend_from_slice(&compressed_hash);
    packed_data.extend_from_slice(&aes_key_hash);
    
    // Compute final checksum for the entire packed binary (except this checksum itself)
    let final_hash = compute_blake3(&packed_data);
    packed_data.extend_from_slice(&final_hash);

    // Ensure minimum size for the packed binary
    if packed_data.len() < 1024 {
        eprintln!("Warning: Packed binary is small, ensuring minimum size");
        while packed_data.len() < 1024 {
            packed_data.push(0); // Padding to ensure minimum size
        }
    }

    let mut output_file = File::create(output_path).expect("Failed to create output file");
    output_file.write_all(&packed_data).expect("Failed to write packed binary");

    let mut permissions = output_file.metadata().expect("Failed to get metadata").permissions();
    permissions.set_mode(0o755); // rwxr-xr-x
    std::fs::set_permissions(output_path, permissions).expect("Failed to set permissions");

    println!("Packed binary created at {}", output_path);
    println!("Original checksum: {}", hash_to_hex(&original_hash));
    println!("Final checksum: {}", hash_to_hex(&final_hash));
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_validate_elf_valid() {
        assert!(validate_elf(b"\x7FELF\x01\x01\x01\x00\x00\x00\x00\x00\x00\x00\x00"));
    }

    #[test]
    fn test_validate_elf_invalid() {
        assert!(!validate_elf(b"Not an ELF file"));
        assert!(!validate_elf(b"\x7FEL"));
    }
}
