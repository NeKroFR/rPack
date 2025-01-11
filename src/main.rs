use std::env;
use std::fs::File;
use std::io::{self, Read, Write};
use std::process::{self, Command};

fn print_help() {
    eprintln!("Usage: <program> <input_file>");
    eprintln!("The input file must be a 64-bit ELF executable.");
}

fn get_filename_from_args() -> String {
    let args: Vec<String> = env::args().collect();

    match args.len() {
        2 => args[1].clone(),
        _ => {
            print_help();
            process::exit(1);
        }
    }
}

fn is_elf_file(filename: &str) -> bool {
    let mut f = match File::open(filename) {
        Ok(file) => file,
        Err(_) => return false,
    };

    let mut magic = [0u8; 4];
    if let Err(_) = f.read_exact(&mut magic) {
        return false;
    }
    magic == [0x7f, 0x45, 0x4c, 0x46]
}

fn open_file(filename: &str) -> Result<Vec<u8>, io::Error> {
    let mut f = File::open(filename)?;
    let mut content = Vec::new();
    f.read_to_end(&mut content)?;
    Ok(content)
}

fn save_file(content: &[u8], filename: &str) -> io::Result<()> {
    let mut f = File::create(filename)?;
    f.write_all(content)?;
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

    let out_filename = format!("{}.packed", filename);
    save_file(&content, &out_filename).unwrap();
	Command::new("chmod")
		.arg("+x")
		.arg(&out_filename)
		.output()
		.expect("Failed to run chmod.");
}
