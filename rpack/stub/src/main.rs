use libc::{c_char, c_long};
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
use checksum::validate_blake3;
use ctor::ctor;

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
#[cfg(not(test))]
fn bait() {
    let name = match CString::new("meow") {
        Ok(name) => name,
        Err(_) => {
            // eprintln!("Error creating CString for memfd_create");
            process::exit(1);
        }
    };
    let fd = unsafe { libc::syscall(319, name.as_ptr(), 0) as i32 }; // 319 is SYS_memfd_create
    if fd < 0 {
        // eprintln!("Error in memfd_create");
        process::exit(1);
    }

    let mut memfd_file = unsafe { File::from_raw_fd(fd) };
    if memfd_file.write_all(BIGMONKE_BYTES).is_err() {
        // eprintln!("Error writing to memfd");
        process::exit(1);
    }

    let prog_path = format!("/proc/self/fd/{}", fd);
    let prog_name = match CString::new(prog_path) {
        Ok(name) => name,
        Err(_) => {
            // eprintln!("Error creating CString for execve");
            process::exit(1);
        }
    };
    let argv: [*const c_char; 2] = [prog_name.as_ptr(), std::ptr::null()];
    const envp: [*const c_char; 1] = [std::ptr::null()];

    unsafe {
        libc::syscall(59, prog_name.as_ptr(), argv.as_ptr(), envp.as_ptr()); // 59 is SYS_execve
    }

    // eprintln!("Failed to execute execve");
    process::exit(1);
}
#[cfg(not(test))]
macro_rules! is_traced {
    () => {
        if is_being_traced() {
            // eprintln!("Tracing detected");
            bait();
        }
    };
}
#[cfg(not(test))]
macro_rules! timecheck {
    () => {
        is_traced!();
        let t1 = Instant::now();
        let t2 = Instant::now();
        if t2.duration_since(t1) > Duration::from_millis(500) {
            // eprintln!("Timing check failed");
            bait();
        }
    };
    ($beg:expr, $delay:expr) => {
        is_traced!();
        if Instant::now().duration_since($beg) > $delay {
            // eprintln!("Timing check with delay failed");
            bait();
        }
    };
}
#[cfg(not(test))]
#[ctor]
fn init_checksum_validation() {
    let current_exe = match env::current_exe() {
        Ok(path) => path,
        Err(_) => {
            // eprintln!("Failed to get current executable path");
            bait();
            return;
        }
    };
    let mut file = match File::open(&current_exe) {
        Ok(f) => f,
        Err(_) => {
            // eprintln!("Failed to open current executable");
            bait();
            return;
        }
    };
    let total_size = match file.metadata() {
        Ok(metadata) => metadata.len(),
        Err(_) => {
            // eprintln!("Failed to get file metadata");
            bait();
            return;
        }
    };

    // Read the final checksum (last 32 bytes for Blake3)
    if file.seek(SeekFrom::End(-(BLAKE3_SIZE as i64))).is_err() {
        // eprintln!("Failed to seek to final hash");
        bait();
        return;
    }
    let mut final_hash = [0u8; BLAKE3_SIZE];
    if file.read_exact(&mut final_hash).is_err() {
        // eprintln!("Failed to read final hash");
        bait();
        return;
    }

    // Check binary integrity (excluding the final hash)
    let binary_size = total_size - (BLAKE3_SIZE as u64);
    if file.seek(SeekFrom::Start(0)).is_err() {
        // eprintln!("Failed to seek to start of binary");
        bait();
        return;
    }
    let mut binary_data = vec![0u8; binary_size as usize];
    if file.read_exact(&mut binary_data).is_err() {
        // eprintln!("Failed to read binary data");
        bait();
        return;
    }

    if !validate_blake3(&binary_data, &final_hash) {
        // eprintln!("ERROR: Binary integrity check failed");
        bait();
    }
}

#[cfg(not(test))]
fn main() {
    timecheck!();

    unsafe {
        // prctl: SYS_prctl = 157, PR_SET_DUMPABLE = 4
        let ret = libc::syscall(157, 4 as c_long, 0, 0, 0, 0);
        if ret == -1 {
            // eprintln!("Failed to disable PR_SET_DUMPABLE");
            bait();
            return;
        }
    }

    let current_exe = match env::current_exe() {
        Ok(path) => path,
        Err(_) => {
            // eprintln!("Failed to get current executable path");
            bait();
            return;
        }
    };
    let mut file = match File::open(&current_exe) {
        Ok(f) => f,
        Err(_) => {
            // eprintln!("Failed to open current executable");
            bait();
            return;
        }
    };
    let fd = file.as_raw_fd();
    unsafe {
        // fcntl: SYS_fcntl = 72, F_SETFD = 2, FD_CLOEXEC = 1
        let ret = libc::syscall(72, fd as c_long, 2 as c_long, 1 as c_long);
        if ret == -1 {
            // eprintln!("Failed to set FD_CLOEXEC");
            bait();
            return;
        }
    }

    let total_size = match file.metadata() {
        Ok(metadata) => metadata.len(),
        Err(_) => {
            // eprintln!("Failed to get file metadata");
            bait();
            return;
        }
    };

    // Size of all hashes at the end:
    // Blake3 (3): original, compressed, aes_key
    const CHECKSUMS_SIZE: usize = 3 * BLAKE3_SIZE;

    // Size of all size fields:
    // 8 (encrypted_size) + 8 (a1_size) + 8 (a2_size) + 8 (white_data_size) + 8 (decompressed_size)
    const SIZE_FIELDS_SIZE: usize = 40;

    // Read all sizes in one go (they're before the checksums)
    if file.seek(SeekFrom::End(-((BLAKE3_SIZE + CHECKSUMS_SIZE + SIZE_FIELDS_SIZE) as i64))).is_err() {
        // eprintln!("Failed to seek to sizes");
        bait();
        return;
    }
    let mut sizes_bytes = [0u8; SIZE_FIELDS_SIZE];
    if file.read_exact(&mut sizes_bytes).is_err() {
        // eprintln!("Failed to read sizes");
        bait();
        return;
    }

    let size_encrypted_payload = u64::from_le_bytes(sizes_bytes[0..8].try_into().unwrap());
    let size_a1 = u64::from_le_bytes(sizes_bytes[8..16].try_into().unwrap());
    let size_a2 = u64::from_le_bytes(sizes_bytes[16..24].try_into().unwrap());
    let size_white_data = u64::from_le_bytes(sizes_bytes[24..32].try_into().unwrap());
    let decompressed_size = u64::from_le_bytes(sizes_bytes[32..40].try_into().unwrap());

    // Read all checksums
    if file.seek(SeekFrom::End(-((BLAKE3_SIZE + CHECKSUMS_SIZE) as i64))).is_err() {
        // eprintln!("Failed to seek to checksums");
        bait();
        return;
    }
    let mut checksum_bytes = vec![0u8; CHECKSUMS_SIZE];
    if file.read_exact(&mut checksum_bytes).is_err() {
        // eprintln!("Failed to read checksums");
        bait();
        return;
    }

    // Extract checksums
    // Format: [original_hash] [compressed_hash] [aes_key_hash]
    let mut offset = 0;
    let mut original_hash = [0u8; BLAKE3_SIZE];
    original_hash.copy_from_slice(&checksum_bytes[offset..offset + BLAKE3_SIZE]);
    offset += BLAKE3_SIZE;

    let mut compressed_hash = [0u8; BLAKE3_SIZE];
    compressed_hash.copy_from_slice(&checksum_bytes[offset..offset + BLAKE3_SIZE]);
    offset += BLAKE3_SIZE;

    let mut aes_key_hash = [0u8; BLAKE3_SIZE];
    aes_key_hash.copy_from_slice(&checksum_bytes[offset..offset + BLAKE3_SIZE]);

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
        // eprintln!("Invalid offsets");
        bait();
        return;
    }

    // Read sections
    if file.seek(SeekFrom::Start(start_encrypted_payload)).is_err() {
        // eprintln!("Failed to seek to encrypted payload");
        bait();
        return;
    }
    let mut encrypted_payload = vec![0u8; size_encrypted_payload as usize];
    if file.read_exact(&mut encrypted_payload).is_err() {
        // eprintln!("Failed to read encrypted payload");
        bait();
        return;
    }

    if file.seek(SeekFrom::Start(start_a1)).is_err() {
        // eprintln!("Failed to seek to a1");
        bait();
        return;
    }
    let mut serialized_a1 = vec![0u8; size_a1 as usize];
    if file.read_exact(&mut serialized_a1).is_err() {
        // eprintln!("Failed to read serialized a1");
        bait();
        return;
    }

    if file.seek(SeekFrom::Start(start_a2)).is_err() {
        // eprintln!("Failed to seek to a2");
        bait();
        return;
    }
    let mut serialized_a2 = vec![0u8; size_a2 as usize];
    if file.read_exact(&mut serialized_a2).is_err() {
        // eprintln!("Failed to read serialized a2");
        bait();
        return;
    }

    if file.seek(SeekFrom::Start(start_white_data)).is_err() {
        // eprintln!("Failed to seek to white data");
        bait();
        return;
    }
    let mut serialized_white_data = vec![0u8; size_white_data as usize];
    if file.read_exact(&mut serialized_white_data).is_err() {
        // eprintln!("Failed to read serialized WhiteData");
        bait();
        return;
    }

    // Deserialize
    let white_data: WhiteData = match bincode::deserialize(&serialized_white_data) {
        Ok(data) => data,
        Err(_) => {
            // eprintln!("Failed to deserialize WhiteData");
            bait();
            return;
        }
    };
    let a1: NTRUVector = match bincode::deserialize(&serialized_a1) {
        Ok(data) => data,
        Err(_) => {
            // eprintln!("Failed to deserialize a1");
            bait();
            return;
        }
    };
    let a2: NTRUVector = match bincode::deserialize(&serialized_a2) {
        Ok(data) => data,
        Err(_) => {
            // eprintln!("Failed to deserialize a2");
            bait();
            return;
        }
    };

    // NTRUVector checksums
    if !a1.verify_checksum() || !a2.verify_checksum() {
        // eprintln!("ERROR: NTRUVector checksum verification failed");
        bait();
        return;
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
        // eprintln!("ERROR: AES key verification failed");
        bait();
        return;
    }

    // Decrypt packed binary
    let aes = AES128::new(&aes_key);
    let padded_compressed_data = (aes.decrypt)(&aes, &encrypted_payload);
    let compressed_data = match aes::unpad_pkcs7(&padded_compressed_data) {
        Some(data) => data,
        None => {
            // eprintln!("Invalid padding in AES decryption");
            bait();
            return;
        }
    };

    // Compressed data checksum
    if !validate_blake3(&compressed_data, &compressed_hash) {
        // eprintln!("ERROR: Compressed data verification failed");
        bait();
        return;
    }

    // Decompress
    let decompressed_data = match decompress(&compressed_data, decompressed_size as usize) {
        Ok(data) => data,
        Err(_) => {
            // eprintln!("Failed to decompress");
            bait();
            return;
        }
    };

    // Decompressed binary checksum
    if !validate_blake3(&decompressed_data, &original_hash) {
        // eprintln!("ERROR: Original binary verification failed");
        bait();
        return;
    }

    let timecheck_start = Instant::now();
    let name = match CString::new("meow") {
        Ok(name) => name,
        Err(_) => {
            // eprintln!("Error creating CString for memfd_create");
            bait();
            return;
        }
    };
    let fd = unsafe { libc::syscall(319, name.as_ptr(), 0) as i32 }; // 319 is SYS_memfd_create
    if fd < 0 {
        // eprintln!("Error in memfd_create");
        bait();
        return;
    }

    let mut memfd_file = unsafe { File::from_raw_fd(fd) };
    if memfd_file.write_all(&decompressed_data).is_err() {
        // eprintln!("Failed to write to memfd");
        bait();
        return;
    }
    timecheck!(timecheck_start, Duration::from_millis(500));

    let prog_path = format!("/proc/self/fd/{}", fd);
    let prog_name = match CString::new(prog_path) {
        Ok(name) => name,
        Err(_) => {
            // eprintln!("Error creating CString for execve");
            bait();
            return;
        }
    };
    let argv: [*const c_char; 2] = [prog_name.as_ptr(), std::ptr::null()];
    const envp: [*const c_char; 1] = [std::ptr::null()];

    unsafe {
        libc::syscall(59, prog_name.as_ptr(), argv.as_ptr(), envp.as_ptr()); // 59 is SYS_execve
    }

    // eprintln!("Failed to execute execve");
    bait();
}

#[cfg(test)]
mod tests {
    use super::*;
   
    #[test]
    fn test_not_traced() {
        assert_eq!(is_being_traced(), false);
    }
    
}
