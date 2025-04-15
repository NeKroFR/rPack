mod decrypt;
mod encrypt;
mod lattice;
mod create_wb;

use crate::lattice::NTRUVector;
use numpy::ndarray::Array1;
use numpy::ndarray::s;
use serde_json::json;
use serde_json::Result;
use std::fs::File;

use decrypt::decrypt_json;
fn load_public_key() -> serde_json::Result<(NTRUVector, NTRUVector, usize, i64)> {
    let pub_enc_data_file = File::open("pub_enc_data.json").map_err(serde_json::Error::io)?;
    let data: encrypt::PubEncData = serde_json::from_reader(pub_enc_data_file)?;

    let degree = data.degree;
    let modulus = data.modulus;
    let pka = NTRUVector {
        vector: Array1::from_vec(data.pka.clone()),
        degree,
        modulus,
        ntt: false,
    };
    let pkb = NTRUVector {
        vector: Array1::from_vec(data.pkb.clone()),
        degree,
        modulus,
        ntt: false,
    };

    Ok((pka, pkb, degree, modulus))
}

fn main() -> Result<()> {
    create_wb::create_wb();

    let message_to_encrypt = "MEOW MEOW MEOW!!";
    println!("Message to encrypt: {}", message_to_encrypt);

    // --- Encryption Section ---
    println!("\n---Encryption---");
    let (pka, pkb, degree, modulus) = load_public_key()?;
    let message_bytes = message_to_encrypt.as_bytes();
    let mut message_bits_vec = Vec::new();
    for byte in message_bytes {
        for i in 0..8 {
            message_bits_vec.push((byte >> (7 - i)) & 1);
        }
    }
    let message_bits = Array1::from_vec(message_bits_vec.iter().map(|&x| x as i64).collect::<Vec<i64>>()); // Removed mut

    if message_bits.len() > degree {
        println!("Error: Message too long for encryption.");
        std::process::exit(1);
    }

    let mut message_padded = Array1::zeros(degree);
    message_padded.slice_mut(s![..message_bits.len()]).assign(&message_bits);

    let (a1, a2) = encrypt::encrypt_func(&message_padded, &pka, &pkb, degree, modulus);

    let ciphertext_file = File::create("ciphertext.json").map_err(serde_json::Error::io)?;
    let ciphertext_data = json!({
        "a1": a1.vector.to_vec(),
        "a2": a2.vector.to_vec(),
    });

    serde_json::to_writer_pretty(ciphertext_file, &ciphertext_data)?;

    println!("Message encrypted and saved to ciphertext.json");

    // --- Decryption Section ---
    println!("\n---Decryption---");
    decrypt_json()?;

    // -- Remove generated json files --
    println!("\n---Cleaning---");
    std::fs::remove_file("ciphertext.json").unwrap_or_else(|_| {
        println!("Error removing ciphertext.json");
    });
    std::fs::remove_file("private_data.json").unwrap_or_else(|_| {
        println!("Error removing private_data.json");
    });
    std::fs::remove_file("pub_enc_data.json").unwrap_or_else(|_| {
        println!("Error removing pub_enc_data.json");
    });
    std::fs::remove_file("wb_dec_data.json").unwrap_or_else(|_| {
        println!("Error removing wb_dec_data.json");
    });

    Ok(())
}
