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

use std::env;
use std::fs::{File, OpenOptions};
use std::io::{Read, Seek, SeekFrom, Write};

mod crypto;
mod compress;

use compress::lz77::*;
use crypto::aes::AES128;

fn read_elf_header(file_path: &str) -> Vec<u8> {
    let mut file = File::open(file_path).expect("Unable to open file");
    let mut buffer = vec![0; 64]; // ELF header size
    file.read_exact(&mut buffer).expect("Couldn't read file, is the provided file really an ELF file?");
    buffer
}

fn read_section_headers(file_path: &str) -> Vec<(u64, u64, u64)> {
    let mut file = File::open(file_path).expect("Unable to open file");
    
    let mut buffer = [0; 64];
    file.read_exact(&mut buffer).expect("Unable to open file, is the provided file really an ELF file?");

    let section_header_offset = u64::from_le_bytes(buffer[0x28..0x30].try_into().unwrap());
    let section_entry_size = u16::from_le_bytes(buffer[0x3A..0x3C].try_into().unwrap()) as u64;
    let num_sections = u16::from_le_bytes(buffer[0x3E..0x40].try_into().unwrap());

    let file_size = file.metadata().expect("Unable to read file size").len();
    if section_header_offset >= file_size {
        panic!("Section header offset off limits : 0x{:x}", section_header_offset);
    }

    let mut sections = Vec::new();

    for i in 0..num_sections {
        let offset = section_header_offset + (i as u64) * section_entry_size;

        if offset + 64 > file_size {
            eprintln!("Section {} crosses EOF, BREAK", i);
            break;
        }

        file.seek(SeekFrom::Start(offset)).expect("Unable to access ELF section");

        let mut shdr = [0; 64];
        file.read_exact(&mut shdr).expect("Failed to read elf section");

        let sh_addr = u64::from_le_bytes(shdr[12..20].try_into().unwrap());
        let sh_offset = u64::from_le_bytes(shdr[24..32].try_into().unwrap());
        let sh_size = u64::from_le_bytes(shdr[32..40].try_into().unwrap());

        sections.push((sh_addr, sh_offset, sh_size));
    }

    sections
}


fn compress_and_encrypt_section(file_path: &str, section_offset: u64, section_size: u64, key: &[u8; 16]) {
    let mut file = OpenOptions::new().read(true).write(true).open(file_path).expect("Unable to open file");
    
    file.seek(SeekFrom::Start(section_offset)).unwrap();
    let mut section_data = vec![0; section_size as usize];
    file.read_exact(&mut section_data).unwrap();

    println!("Original size : {} Bytes", section_data.len());

    let compressed = lz77_compress(&section_data, 1024);
    let mut compressed_binary = Vec::new();
    write_compressed_data(&compressed, &mut compressed_binary).unwrap();

    println!("Compressed size : {} Bytes", compressed_binary.len());

    let aes = AES128::new(key);
    let block_size = 16;
    let pad_len = block_size - (compressed_binary.len() % block_size);
    compressed_binary.extend(vec![pad_len as u8; pad_len]);

    let encrypted_data = (aes.encrypt)(&aes, &compressed_binary);

    file.seek(SeekFrom::Start(section_offset)).unwrap();
    file.write_all(&encrypted_data).unwrap();
}

fn modify_entrypoint(file_path: &str, new_entry: u64) {
    let mut file = OpenOptions::new().write(true).open(file_path).expect("Unable to open file");
    file.seek(SeekFrom::Start(0x18)).unwrap();
    file.write_all(&new_entry.to_le_bytes()).unwrap();
}

fn inject_stub(file_path: &str, old_entry: u64) -> u64 {
    let mut file = OpenOptions::new().read(true).write(true).open(file_path)
        .expect("Unable to open file");

    let mut stub = vec![];
    let mut stub_file = File::open("stub.bin").expect("Failed to read stub file");
    stub_file.read_to_end(&mut stub).expect("Error reading stub file");

    let stub_offset = file.seek(SeekFrom::End(0)).expect("Seek failed");
    file.write_all(&stub).expect("Failed to write stub");

    file.seek(SeekFrom::Start(stub_offset + 8)).expect("Seek failed");
    file.write_all(&old_entry.to_le_bytes()).expect("Failed to write old entrypoint");

    println!("Stub injected at offset 0x{:x}", stub_offset);
    stub_offset
}


fn main() {
    // Récupérer le chemin du fichier ELF depuis les arguments
    let args: Vec<String> = env::args().collect();
    if args.len() < 2 {
        eprintln!("Usage: {} <ELF file>", args[0]);
        std::process::exit(1);
    }
    let file_path = &args[1];

    println!("Target ELF file: {}", file_path);

    // Lire l'ELF et identifier les sections
    let sections = read_section_headers(file_path);
    
    // Trouver la section à packer (ex: `.data` ou `.rodata`)
    let (section_addr, section_offset, section_size) = sections.iter()
        .find(|(_, _, size)| *size > 0)  // Prendre la première section non vide
        .expect("No section found!");
    
    println!("Target secetion: offset=0x{:x}, size=0x{:x}", section_offset, section_size);

    // Générer une clé AES
    let key = AES128::generate_key();
    println!("AES key: {:?}", key);

    // Compresser et chiffrer la section
    compress_and_encrypt_section(file_path, *section_offset, *section_size, &key);

    // Ajouter un stub de déchiffrement/décompression et modifier l'entrypoint
    let new_entry = inject_stub(file_path, &key);
    
    // Modifier le point d'entrée ELF pour exécuter le stub au démarrage
    modify_entrypoint(file_path, new_entry);

    println!("Successfully packed!");
}
