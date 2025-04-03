use libc::{memfd_create, fexecve, c_char, fcntl, F_SETFD, FD_CLOEXEC};
use std::env;
use std::ffi::CString;
use std::fs::File;
use std::io::{Read, Seek, SeekFrom, Write};
use std::os::unix::io::{FromRawFd, AsRawFd};
use std::process;
use lz4_flex::decompress;

fn main() {
    let current_exe = env::current_exe().expect("Failed to get current executable path");
    let mut file = File::open(current_exe).expect("Failed to open current executable");
    let fd = file.as_raw_fd();
    unsafe {
        fcntl(fd, F_SETFD, FD_CLOEXEC);
    }

    let total_size = file.metadata().expect("Failed to get file metadata").len();
    file.seek(SeekFrom::Start(total_size - 8)).expect("Failed to seek");
    let mut size_bytes = [0u8; 8];
    file.read_exact(&mut size_bytes).expect("Failed to read decompressed size");
    let decompressed_size = u64::from_le_bytes(size_bytes);

    file.seek(SeekFrom::Start(total_size - 16)).expect("Failed to seek");
    file.read_exact(&mut size_bytes).expect("Failed to read compressed size");
    let compressed_size = u64::from_le_bytes(size_bytes);

    file.seek(SeekFrom::Start(total_size - 16 - compressed_size)).expect("Failed to seek");
    let mut compressed_payload = vec![0u8; compressed_size as usize];
    file.read_exact(&mut compressed_payload).expect("Failed to read compressed payload");

    let decompressed_data = decompress(&compressed_payload, decompressed_size as usize)
        .expect("Failed to decompress");

    let name = CString::new("meow").unwrap();
    let fd = unsafe { memfd_create(name.as_ptr(), 0) };
    if fd < 0 {
        eprintln!("Error in memfd_create");
        process::exit(1);
    }

    let mut memfd_file = unsafe { File::from_raw_fd(fd) };
    memfd_file.write_all(&decompressed_data).expect("Failed to write to memfd");

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

