/* TESTING AES (seems to work)
mod crypto;
use crypto::aes::AES128;

fn main() {
    let message = b"meow meow meow";
    println!("Original message: {:?}", String::from_utf8_lossy(message));
    
    let key = AES128::generate_key();
    println!("Generated key: {:?}", key);

    let block_size = 16;
    let pad_len = block_size - (message.len() % block_size);
    let mut padded = message.to_vec();
    padded.extend(vec![pad_len as u8; pad_len]);
    let aes = AES128::new(&key);

    let encrypted = (aes.encrypt)(&aes, &padded);
    println!("Encrypted bytes (hex): {:?}", encrypted.iter().map(|b| format!("{:02x}", b)).collect::<Vec<String>>().join(""));

    let decrypted_padded = (aes.decrypt)(&aes, &encrypted);
    let pad_len = decrypted_padded[decrypted_padded.len() - 1] as usize;
    let decrypted = &decrypted_padded[..decrypted_padded.len() - pad_len];
    println!("Decrypted message: {:?}", String::from_utf8_lossy(decrypted));

    assert_eq!(message, decrypted);
}
*/

/*
mod crypto;
mod compress;

use std::env;
use std::process::{self, Command};
use std::fs::File;
use std::io::{self, Read, Write};

use crate::crypto::aes::encrypt_content;
use crate::crypto::WB::create_whitebox;
use crate::compress::compress_content;

fn print_help() {
    eprintln!("Usage: <program> <input_file>");
    eprintln!("The input file must be a 64-bit ELF executable.");
}

fn get_filename_from_args() -> String {
    let args: Vec<String> = env::args().collect();
    if args.len() != 2 {
        print_help();
        process::exit(1);
    }
    args[1].clone()
}

fn is_elf_file(filename: &str) -> bool {
    let mut file = match File::open(filename) {
        Ok(f) => f,
        Err(_) => return false,
    };
    let mut magic = [0u8; 4];pub mod crypto;


fn pack(content: Vec<u8>) -> io::Result<Vec<u8>> {
    // to do
    Ok(encrypted)
}

fn open_file(filename: &str) -> io::Result<Vec<u8>> {
    let mut file = File::open(filename)?;
    let mut content = Vec::new();
    file.read_to_end(&mut content)?;
    Ok(content)
}

fn save_file(content: &[u8], filename: &str) -> io::Result<()> {
    let mut file = File::create(filename)?;
    file.write_all(content)?;
    println!("Success: the file has been successfully packed.");
    Ok(())
}

fn main() {
    let filename = get_filename_from_args();
    if !is_elf_file(&filename) {
        eprintln!("Error: The file is not a valid 64-bit ELF executable.");
        process::exit(1);
    }

    let content = open_file(&filename).unwrap();
    let packed_content = pack(content).unwrap();
    let out_filename = format!("{}.packed", filename);
    save_file(&packed_content, &out_filename).unwrap();

    Command::new("chmod")
        .arg("+x")
        .arg(&out_filename)
        .output()
        .expect("Failed to run chmod.");
}
*/

<<<<<<< HEAD
=======
/* main.rs (the real one)
mod crypto;
use crypto::WB::wb::{load_whitebox_data, WBVector};
use crypto::WB::ntru_vector::NTRUVector;
use std::fs;

fn main() {
    // Load whitebox data
    let wb_data = load_whitebox_data();
    
    // Load ciphertext (example - should be replaced with actual data)
    let ciphertext = fs::read_to_string("ciphertext.json")
        .expect("Failed to read ciphertext");
    let ciphertext: serde_json::Value = serde_json::from_str(&ciphertext).unwrap();
    
    // Initialize vectors
    let degree = wb_data.beta.len();
    let modulus = wb_data.beta.iter().product::<i64>();
    
    let mut a1 = NTRUVector::new(degree, modulus);
    let mut a2 = NTRUVector::new(degree, modulus);
    
    // Load ciphertext data (pseudo-code)
    if let (Some(a1_vec), Some(a2_vec)) = (
        ciphertext.get("a1").and_then(|v| v.as_array()),
        ciphertext.get("a2").and_then(|v| v.as_array()),
    ) {
        a1.vector = a1_vec.iter().map(|v| v.as_i64().unwrap()).collect();
        a2.vector = a2_vec.iter().map(|v| v.as_i64().unwrap()).collect();
    }
    
    // Convert to WB vectors
    let mut wb_a1 = WBVector::new(degree, modulus, wb_data);
    let mut wb_a2 = WBVector::new(degree, modulus, wb_data);
    wb_a1.vector = a1.vector.clone();
    wb_a2.vector = a2.vector.clone();
    
    // Perform NTT transforms
    wb_a1.goto_ntt(wb_data.root);
    wb_a2.goto_ntt(wb_data.root);
    
    // Montgomery multiplication
    let mut result = wb_a1.montgomery_multiply(&wb_a2);
    
    // Convert back from NTT
    result.goback_ntt(wb_data.unroot, wb_data.ninv);
    
    // Apply mask and rotation
    let decrypted = apply_mask_and_rotation(&result.vector, wb_data);
    
    // Convert to bytes
    let bytes = binary_to_bytes(&decrypted);
    
    println!("Decrypted message: {}", String::from_utf8_lossy(&bytes));
}

fn apply_mask_and_rotation(vec: &[i64], wb_data: &WhiteboxData) -> Vec<i64> {
    let mut result = vec.to_vec();
    
    // Apply rotation
    result.rotate_right(wb_data.rotate);
    
    // Apply mask
    for (i, val) in result.iter_mut().enumerate() {
        let masked = *val + wb_data.mask[i % wb_data.mask.len()];
        *val = if masked > wb_data.beta.iter().product::<i64>() / 2 {
            1 - (masked % 2)
        } else {
            masked % 2
        };
    }
    
    result
}

fn binary_to_bytes(bits: &[i64]) -> Vec<u8> {
    bits.chunks(8)
        .map(|chunk| {
            let mut byte = 0;
            for (i, &bit) in chunk.iter().enumerate() {
                if bit != 0 {
                    byte |= 1 << (7 - i);
                }
            }
            byte as u8
        })
        .collect()
}
*/

/* TESTING COMPRESSION
use std::env;
use std::fs::{File, OpenOptions};
use std::io::{Read, Seek, SeekFrom, Write};
>>>>>>> 6a4a0e4 (rust is hell)

// TEST AES (seems to work)
mod crypto;
use crypto::aes::AES128;

fn main() {
    let message = b"meow meow meow";
    println!("Original message: {:?}", String::from_utf8_lossy(message));
    
    let key = AES128::generate_key();
    println!("Generated key: {:?}", key);

    let block_size = 16;
    let pad_len = block_size - (message.len() % block_size);
    let mut padded = message.to_vec();
    padded.extend(vec![pad_len as u8; pad_len]);
    let aes = AES128::new(&key);

    let encrypted = (aes.encrypt)(&aes, &padded);
    println!("Encrypted bytes (hex): {:?}", encrypted.iter().map(|b| format!("{:02x}", b)).collect::<Vec<String>>().join(""));

    let decrypted_padded = (aes.decrypt)(&aes, &encrypted);
    let pad_len = decrypted_padded[decrypted_padded.len() - 1] as usize;
    let decrypted = &decrypted_padded[..decrypted_padded.len() - pad_len];
    println!("Decrypted message: {:?}", String::from_utf8_lossy(decrypted));

    assert_eq!(message, decrypted);
}
*/
/* TESTING WHITEBOX */
pub mod crypto;

use std::time::Instant;
use crypto::WB::create_wb::write_data;

fn main() -> Result<(), String> {
    // generate the whitebox
    let degree = 512;
    let modulus = 1231873;
    let k = 5;
    let beta = vec![13, 16, 19, 27, 29];
    let beta_p = vec![11, 17, 23, 25, 31];
    let chal = 2;

    let start_time = Instant::now();
    write_data(degree, modulus, &beta, &beta_p, k, chal)?;
    let duration = start_time.elapsed();
    println!("WB data generation completed in: {:?}", duration);
    Ok(())
}
