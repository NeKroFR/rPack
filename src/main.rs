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
