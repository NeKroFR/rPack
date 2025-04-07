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

// Placeholder for the AES key
const AES_KEY: [u8; 16] = [
    0xDE, 0xAD, 0xBE, 0xEF, 0xDE, 0xAD, 0xBE, 0xEF,
    0xDE, 0xAD, 0xBE, 0xEF, 0xDE, 0xAD, 0xBE, 0xEF
];

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
    // BigMONKE
    println!("{}", r#"
            ,.-" "-.,
           /   ===   \
          /  =======  \
       __|  (o)   (0)  |__
      / _|    .---.    |_ \
     | /.----/ O O \----.\ |
      \/     |     |     \/
      |                   |
      |                   |
      |                   |
      _\   -.,_____,.-   /_
  ,.-"  "-.,_________,.-"  "-.,
 /         |   BIG   |         \
|          l. MONKE .l          |
    "#);
    std::process::exit(1);    
}

macro_rules! is_traced {
    () => {
        if is_being_traced() {
            bait();
        }
    };
}

// Time check  (also call the is_traced macro so we also check for debuggers)
// No arguments -> check if the time since the last call is greater than 100ms
// 2 arguments (beg, delay) -> check if the time since beg is greater than delay
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
    // Make the process undumpable
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

    // Read decompressed size (last 8 bytes)
    file.seek(SeekFrom::Start(total_size - 8)).expect("Failed to seek");
    let mut size_bytes = [0u8; 8];
    file.read_exact(&mut size_bytes).expect("Failed to read decompressed size");
    let decompressed_size = u64::from_le_bytes(size_bytes);

    // Read encrypted size (8 bytes before decompressed size)
    file.seek(SeekFrom::Start(total_size - 16)).expect("Failed to seek");
    file.read_exact(&mut size_bytes).expect("Failed to read encrypted size");
    let encrypted_size = u64::from_le_bytes(size_bytes);

    // Read the encrypted payload
    file.seek(SeekFrom::Start(total_size - 16 - encrypted_size)).expect("Failed to seek");
    let mut encrypted_payload = vec![0u8; encrypted_size as usize];
    file.read_exact(&mut encrypted_payload).expect("Failed to read encrypted payload");

    // Decrypt the payload using the embedded AES key
    let aes = AES128::new(&AES_KEY);
    let padded_compressed_data = (aes.decrypt)(&aes, &encrypted_payload);
    let compressed_data = aes::unpad_pkcs7(&padded_compressed_data).expect("Invalid padding");

    // Decompress the data
    let decompressed_data = decompress(&compressed_data, decompressed_size as usize)
        .expect("Failed to decompress");

    let timecheck_start = Instant::now();
    // Create an in-memory file descriptor
    let name = CString::new("meow").unwrap();
    let fd = unsafe { memfd_create(name.as_ptr(), 0) };
    if fd < 0 {
        eprintln!("Error in memfd_create");
        process::exit(1);
    }

    let mut memfd_file = unsafe { File::from_raw_fd(fd) };
    memfd_file.write_all(&decompressed_data).expect("Failed to write to memfd");
    timecheck!(timecheck_start, Duration::from_millis(100));

    // Run the decompressed binary
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
