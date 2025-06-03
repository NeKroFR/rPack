# rPack

rPack is an ELF packer written in Rust.

## Features

- **Compression**: Uses lz4 for efficient compression of the input binary.
- **Encryption**: Encrypts the compressed binary with AES-128 ECB.
- **Whitebox Cryptography**: Encrypt the AES key using a lattice based whitebox ([BVWhiteBox](https://github.com/quarkslab/BVWhiteBox)).
- **Anti-Debugging**: Uses multiple anti-debugging techniques such as `ptrace` or `prctl`
- **Anti-VM**: Uses multiple method to detect if the binary is runned in a virtualized environment.
- **Integrity Checks**: Uses blake3 to perform multiple checksums.

## Building

To build rPack you need to use [Cargo](https://doc.rust-lang.org/cargo/getting-started/installation.html) :

```sh
git clone git@github.com:NeKroFR/rPack.git
cd rPack/rpack
cargo build 
```
This command will build all crates in the workspace, including the stub and rpack binaries. The `rpack` binary (the packer) will be located at `target/debug/rpack`.
The build process uses a custom `build.rs` script to compile and strip the stub binary, embedding it into the packer.

## Usage

```sh
./target/debug/rpack <input_binary> <output_packed_binary>
```

- `<input_binary>`: Path to the ELF binary you want to pack (e.g., `/bin/ls`).
- `<output_packed_binary>`: Path where the packed binary will be saved (e.g., `ls.packed`).

The command then would be:

```sh
./target/debug/rpack /bin/ls ls.packed
```
