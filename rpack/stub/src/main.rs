use libc::{memfd_create, fexecve, c_char, fcntl, F_SETFD, FD_CLOEXEC, prctl, PR_SET_DUMPABLE};
use std::env;
use std::ffi::CString;
use std::fs::File;
use std::io::{Read, Seek, SeekFrom, Write};
use std::os::unix::io::{FromRawFd, AsRawFd};
use std::time::{Instant, Duration};
use std::process;
use lz4_flex::decompress;
use aes::AES128;
use whitebox::{decrypt_message, NTRUVector, WhiteData};
use bincode;
use serde::Deserialize;
use checksum::{compute_blake3, validate_blake3};

const BIGMONKE_BYTES: &[u8] = include_bytes!("BIGMONKE");

const BLAKE3_SIZE: usize = 32;

fn is_being_traced() -> bool {
    use std::fs::File;
    use std::io::Read;

    let mut file = match File::open("/proc/self/status") {
        Ok(f) => f,
        Err(_) => return true,
    };
    let mut contents = String::new();
    if file.read_to_string(&mut contents).is_err() {
        return true;
    }

    for line in contents.lines() {
        if line.starts_with("TracerPid:") {
            let parts: Vec<&str> = line.split_whitespace().collect();
            if parts.len() >= 2 {
                let tracer_pid = parts[1].parse::<i32>().unwrap_or(0);
                return tracer_pid != 0;
            }
        }
    }
    false
}

fn bait() {
    let name = CString::new("meow").unwrap();
    let fd = unsafe { memfd_create(name.as_ptr(), 0) };
    if fd < 0 {
        eprintln!("Error in memfd_create");
        process::exit(1);
    }

    let mut memfd_file = unsafe { File::from_raw_fd(fd) };
    memfd_file.write_all(BIGMONKE_BYTES).expect("Failed to write to memfd");

    let prog_path = format!("/proc/self/fd/{}", fd);
    let prog_name = CString::new(prog_path).unwrap();
    let argv: [*const c_char; 2] = [prog_name.as_ptr(), std::ptr::null()];
    let envp: [*const c_char; 1] = [std::ptr::null()];

    unsafe {
        fexecve(fd, argv.as_ptr(), envp.as_ptr());
    }

    eprintln!("Failed to execute fexecve");
    process::exit(1);
}

macro_rules! is_traced {
    () => {
        if is_being_traced() {
            bait();
        }
    };
}

macro_rules! timecheck {
    () => {
        is_traced!();
        let t1 = Instant::now();
        let t2 = Instant::now();
        if t2.duration_since(t1) > Duration::from_millis(100) {
            bait();
        }
    };
    ($beg:expr, $delay:expr) => {
        is_traced!();
        if Instant::now().duration_since($beg) > $delay {
            bait();
        }
    };
}

fn main() {    
    timecheck!();
    unsafe {
        if prctl(PR_SET_DUMPABLE, 0, 0, 0, 0) == -1 {
            eprintln!("Failed to disable PR_SET_DUMPABLE");
            std::process::exit(1);
        }
    }

    let current_exe = env::current_exe().expect("Failed to get current executable path");
    let mut file = File::open(&current_exe).expect("Failed to open current executable");
    let fd = file.as_raw_fd();
    unsafe {
        fcntl(fd, F_SETFD, FD_CLOEXEC);
    }

    let total_size = file.metadata().expect("Failed to get file metadata").len();
    
    // Read the final checksum (last 32 bytes for Blake3)
    file.seek(SeekFrom::End(-(BLAKE3_SIZE as i64))).expect("Failed to seek to final checksum");
    let mut final_hash = [0u8; BLAKE3_SIZE];
    file.read_exact(&mut final_hash).expect("Failed to read final hash");
    
    // Check binary integrity (excluding the final hash)
    let binary_size = total_size - (BLAKE3_SIZE as u64);
    file.seek(SeekFrom::Start(0)).expect("Failed to seek to start");
    let mut binary_data = vec![0u8; binary_size as usize];
    file.read_exact(&mut binary_data).expect("Failed to read binary data");
    
    if !validate_blake3(&binary_data, &final_hash) {
        eprintln!("ERROR: Packed binary integrity check failed!");
        bait();
    }
    
    // Size of all hashes at the end: 
    // Blake3 (3): original, compressed, aes_key, final
    const CHECKSUMS_SIZE: usize = 3 * BLAKE3_SIZE;
    
    // Size of all size fields: 
    // 8 (encrypted_size) + 8 (a1_size) + 8 (a2_size) + 8 (white_data_size) + 8 (decompressed_size)
    const SIZE_FIELDS_SIZE: usize = 40;
    
    // Read all sizes in one go (they're before the checksums)
    file.seek(SeekFrom::End(-((BLAKE3_SIZE + CHECKSUMS_SIZE + SIZE_FIELDS_SIZE) as i64)))
        .expect("Failed to seek to sizes");
    let mut sizes_bytes = [0u8; SIZE_FIELDS_SIZE];
    file.read_exact(&mut sizes_bytes).expect("Failed to read sizes");

    let size_encrypted_payload = u64::from_le_bytes(sizes_bytes[0..8].try_into().unwrap());
    let size_a1 = u64::from_le_bytes(sizes_bytes[8..16].try_into().unwrap());
    let size_a2 = u64::from_le_bytes(sizes_bytes[16..24].try_into().unwrap());
    let size_white_data = u64::from_le_bytes(sizes_bytes[24..32].try_into().unwrap());
    let decompressed_size = u64::from_le_bytes(sizes_bytes[32..40].try_into().unwrap());
    
    // Read all checksums
    file.seek(SeekFrom::End(-((BLAKE3_SIZE + CHECKSUMS_SIZE) as i64)))
        .expect("Failed to seek to checksums");
    let mut checksum_bytes = vec![0u8; CHECKSUMS_SIZE];
    file.read_exact(&mut checksum_bytes).expect("Failed to read checksums");
    
    // Extract checksums
    // Format: [original_hash] [compressed_hash] [aes_key_hash]
    let mut offset = 0;
    let mut original_hash = [0u8; BLAKE3_SIZE];
    original_hash.copy_from_slice(&checksum_bytes[offset..offset+BLAKE3_SIZE]);
    offset += BLAKE3_SIZE;
    
    let mut compressed_hash = [0u8; BLAKE3_SIZE];
    compressed_hash.copy_from_slice(&checksum_bytes[offset..offset+BLAKE3_SIZE]);
    offset += BLAKE3_SIZE;
    
    let mut aes_key_hash = [0u8; BLAKE3_SIZE];
    aes_key_hash.copy_from_slice(&checksum_bytes[offset..offset+BLAKE3_SIZE]);
    offset += BLAKE3_SIZE;

    // Calculate offsets for data sections
    // Format: [STUB] [encrypted_data] [a1] [a2] [white_data] [sizes] [checksums] [final_hash]
    let metadata_size = (SIZE_FIELDS_SIZE + CHECKSUMS_SIZE + BLAKE3_SIZE) as u64;
    let start_white_data = total_size - metadata_size - size_white_data;
    let start_a2 = start_white_data - size_a2;
    let start_a1 = start_a2 - size_a1;
    let start_encrypted_payload = start_a1 - size_encrypted_payload;
    
    // Check offsets
    if start_encrypted_payload >= total_size || start_a1 >= total_size || 
       start_a2 >= total_size || start_white_data >= total_size {
        bait();
    }

    // Read sections
    file.seek(SeekFrom::Start(start_encrypted_payload)).expect("Failed to seek to encrypted payload");
    let mut encrypted_payload = vec![0u8; size_encrypted_payload as usize];
    file.read_exact(&mut encrypted_payload).expect("Failed to read encrypted payload");

    file.seek(SeekFrom::Start(start_a1)).expect("Failed to seek to a1");
    let mut serialized_a1 = vec![0u8; size_a1 as usize];
    file.read_exact(&mut serialized_a1).expect("Failed to read serialized a1");

    file.seek(SeekFrom::Start(start_a2)).expect("Failed to seek to a2");
    let mut serialized_a2 = vec![0u8; size_a2 as usize];
    file.read_exact(&mut serialized_a2).expect("Failed to read serialized a2");

    file.seek(SeekFrom::Start(start_white_data)).expect("Failed to seek to white data");
    let mut serialized_white_data = vec![0u8; size_white_data as usize];
    file.read_exact(&mut serialized_white_data).expect("Failed to read serialized WhiteData");

    // Deserialize
    let white_data: WhiteData = bincode::deserialize(&serialized_white_data)
        .expect("Failed to deserialize WhiteData");
    let a1: NTRUVector = bincode::deserialize(&serialized_a1)
        .expect("Failed to deserialize a1");
    let a2: NTRUVector = bincode::deserialize(&serialized_a2)
        .expect("Failed to deserialize a2");

    // NTRUVector checksums
    if !a1.verify_checksum() || !a2.verify_checksum() {
        eprintln!("ERROR: NTRUVector checksum verification failed!");
        bait();
    }

    // Decrypt the AES key
    let decrypted_bits = decrypt_message(&white_data, &a1, &a2, a1.degree, a1.modulus);
    let mut aes_key = [0u8; 16];
    for i in 0..16 {
        for j in 0..8 {
            let bit = decrypted_bits[i * 8 + j] as u8;
            aes_key[i] |= bit << j;
        }
    }

    // AES key checksum
    if !validate_blake3(&aes_key, &aes_key_hash) {
        eprintln!("ERROR: AES key verification failed!");
        bait();
    }

    // Decrypt packed binary
    let aes = AES128::new(&aes_key);
    let padded_compressed_data = (aes.decrypt)(&aes, &encrypted_payload);
    let compressed_data = aes::unpad_pkcs7(&padded_compressed_data).expect("Invalid padding");

    // Compressed data checksum
    if !validate_blake3(&compressed_data, &compressed_hash) {
        eprintln!("ERROR: Compressed data verification failed!");
        bait();
    }

    // Decompress
    let decompressed_data = decompress(&compressed_data, decompressed_size as usize)
        .expect("Failed to decompress");

    // Decompressed binary checksum
    if !validate_blake3(&decompressed_data, &original_hash) {
        eprintln!("ERROR: Original binary verification failed!");
        bait();
    }

    let timecheck_start = Instant::now();
    let name = CString::new("meow").unwrap();
    let fd = unsafe { memfd_create(name.as_ptr(), 0) };
    if fd < 0 {
        eprintln!("Error in memfd_create");
        process::exit(1);
    }

    let mut memfd_file = unsafe { File::from_raw_fd(fd) };
    memfd_file.write_all(&decompressed_data).expect("Failed to write to memfd");
    timecheck!(timecheck_start, Duration::from_millis(100));

    let prog_path = format!("/proc/self/fd/{}", fd);
    let prog_name = CString::new(prog_path).unwrap();
    let argv: [*const c_char; 2] = [prog_name.as_ptr(), std::ptr::null()];
    let envp: [*const c_char; 1] = [std::ptr::null()];

    unsafe {
        fexecve(fd, argv.as_ptr(), envp.as_ptr());
    }

    eprintln!("Failed to execute fexecve");
    process::exit(1);
}
