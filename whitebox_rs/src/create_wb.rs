use std::fs::File;
use std::io::Write;
use std::process::Command;

use crate::consts::{
    CREATE_WB_HEXA,
    DEFAULT_PRIVATE_DATA,
    DEFAULT_PUB_ENC_DATA,
    DEFAULT_WB_DEC_DATA,
};

fn is_python_installed() -> bool {
    Command::new("python3")
        .arg("--version")
        .output()
        .is_ok()
}

pub fn create_wb(dir: &str) {
    if !is_python_installed() {
        eprintln!("Failed to create the whitebox: python3 is not installed.");
        return handle_failure(dir);
    }
    
    let create_wb_bytes = match hex::decode(CREATE_WB_HEXA) {
        Ok(bytes) => bytes,
        Err(e) => {
            eprintln!("Failed to decode hex: {:?}", e);
            return handle_failure(dir);
        }
    };

    let create_wb_code = String::from_utf8_lossy(&create_wb_bytes);

    let mut file = match File::create(format!("{}/create_wb.py", dir)) {
        Ok(file) => file,
        Err(e) => {
            eprintln!("Failed to create the whitebox: {:?}", e);
            return handle_failure(dir);
        }
    };

    if let Err(e) = file.write_all(create_wb_code.as_bytes()) {
        eprintln!("Failed to create the whitebox: {:?}", e);
        return handle_failure(dir);
    }

    let status = Command::new("python3")
        .arg("create_wb.py")
        .current_dir(dir)
        .status();

    match status {
        Ok(status) => {
            if !status.success() {
                eprintln!("Failed to create the whitebox: {:?}", status);
                return handle_failure(dir);
            }
        }
        Err(e) => {
            eprintln!("Failed to create the whitebox: {:?}", e);
            return handle_failure(dir);
        }
    }
}

fn handle_failure(dir: &str) {
    let mut input = String::new();
    println!("Do you want to use default values? (Y/N)");
    std::io::stdin()
        .read_line(&mut input)
        .expect("Failed to read line");
    let input = input.trim().to_uppercase();
    
    if input == "Y" {
        let default_files = [
            ("private_data.json", DEFAULT_PRIVATE_DATA),
            ("pub_enc_data.json", DEFAULT_PUB_ENC_DATA),
            ("wb_dec_data.json", DEFAULT_WB_DEC_DATA),
        ];

        for (filename, hex_content) in default_files.iter() {
            let bytes = match hex::decode(hex_content) {
                Ok(bytes) => bytes,
                Err(e) => {
                    eprintln!("Failed to decode default hex for {}: {:?}", filename, e);
                    std::process::exit(1);
                }
            };

            let mut file = match File::create(format!("{}/{}", dir, filename)) {
                Ok(file) => file,
                Err(e) => {
                    eprintln!("Failed to create {}: {:?}", filename, e);
                    std::process::exit(1);
                }
            };

            if let Err(e) = file.write_all(&bytes) {
                eprintln!("Failed to write to {}: {:?}", filename, e);
                std::process::exit(1);
            }
        }
        println!("Default files created successfully in {}", dir);
    }
    else {
        if let Err(e) = std::fs::remove_dir_all(dir) {
            eprintln!("Failed to remove directory {}: {:?}", dir, e);
        }
        std::process::exit(1);
    }
}