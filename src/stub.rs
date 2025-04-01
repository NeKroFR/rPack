use libc::{memfd_create, fexecve, c_char, fcntl, F_SETFD, FD_CLOEXEC};
use std::env;
use std::ffi::CString;
use std::fs::File;
use std::io::{Read, Seek, SeekFrom, Write};
use std::os::unix::io::{FromRawFd, AsRawFd};
use std::process;
use lz4_flex::decompress;  // Import LZ4 decompression function

fn main() {
    // Get the path to the current executable
    let current_exe = env::current_exe().expect("Failed to get current executable path");

    // Open the current executable and set FD_CLOEXEC to close it after execution
    let mut file = File::open(&current_exe).expect("Failed to open current executable");
    let fd = file.as_raw_fd();
    unsafe {
        fcntl(fd, F_SETFD, FD_CLOEXEC);
    }

    // Get the total size of the file
    let total_size = file.metadata().expect("Failed to get file metadata").len();

    // Read the decompressed size from the last 8 bytes
    file.seek(SeekFrom::Start(total_size - 8)).expect("Failed to seek");
    let mut size_bytes = [0u8; 8];
    file.read_exact(&mut size_bytes).expect("Failed to read decompressed size");
    let decompressed_size = u64::from_le_bytes(size_bytes);

    // Read the compressed size from the 8 bytes before that
    file.seek(SeekFrom::Start(total_size - 16)).expect("Failed to seek");
    file.read_exact(&mut size_bytes).expect("Failed to read compressed size");
    let compressed_size = u64::from_le_bytes(size_bytes);

    // Seek to the start of the compressed payload and read it
    file.seek(SeekFrom::Start(total_size - 16 - compressed_size)).expect("Failed to seek");
    let mut compressed_payload = vec![0u8; compressed_size as usize];
    file.read_exact(&mut compressed_payload).expect("Failed to read compressed payload");

    // Decompress the payload
    let decompressed_data = decompress(&compressed_payload, decompressed_size as usize)
        .expect("Failed to decompress");

    // Create an anonymous memory file descriptor (memfd)
    let name = CString::new("meow").unwrap();
    let fd = unsafe { memfd_create(name.as_ptr(), 0) };
    if fd < 0 {
        eprintln!("Error in memfd_create");
        process::exit(1);
    }

    // Write the decompressed data to the memfd
    let mut memfd_file = unsafe { File::from_raw_fd(fd) };
    memfd_file.write_all(&decompressed_data).expect("Failed to write to memfd");

    // Set up argv and envp for execution
    let prog_path = format!("/proc/self/fd/{}", fd);
    let prog_name = CString::new(prog_path).unwrap();
    let argv: [*const c_char; 2] = [prog_name.as_ptr(), std::ptr::null()];
    let envp: [*const c_char; 1] = [std::ptr::null()];

    // Execute the binary from memfd
    unsafe {
        fexecve(fd, argv.as_ptr(), envp.as_ptr());
    }

    eprintln!("Failed to execute fexecve");
    process::exit(1);
}
