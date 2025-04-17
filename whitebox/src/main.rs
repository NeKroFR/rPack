mod decrypt;
mod encrypt;
mod lattice;
mod create_wb;

use crate::encrypt::encrypt_func;
use crate::decrypt::decrypt_message;
use numpy::ndarray::Array1;
use numpy::ndarray::s;

fn binary_to_text(binary_str: String) -> String {
    let mut binary_str_mut = binary_str;
    if binary_str_mut.len() % 8 != 0 {
        binary_str_mut = format!("{:0<width$}", binary_str_mut, width = binary_str_mut.len() + (8 - binary_str_mut.len() % 8));
    }

    let binary_values: Vec<&str> = (0..binary_str_mut.len())
        .step_by(8)
        .map(|i| &binary_str_mut[i..i + 8])
        .collect();

    let ascii_chars: Vec<char> = binary_values
        .iter()
        .filter_map(|bv| {
            let val = u8::from_str_radix(bv, 2).unwrap_or(0);
            Some(val as char)
        })
        .collect();

    ascii_chars.into_iter().collect()
}

fn main() {
    let (pub_enc_data, white_data) = create_wb::create_whitebox();

    let message_to_encrypt = "MEOW MEOW MEOW!!";
    println!("Message to encrypt: {}", message_to_encrypt);

    // --- Encryption Section ---
    let message_bytes = message_to_encrypt.as_bytes();
    let mut message_bits_vec = Vec::new();
    for byte in message_bytes {
        for i in 0..8 {
            message_bits_vec.push((byte >> (7 - i)) & 1);
        }
    }
    let message_bits = Array1::from_vec(message_bits_vec.iter().map(|&x| x as i64).collect::<Vec<i64>>());

    if message_bits.len() > pub_enc_data.degree {
        println!("Error: Message too long for encryption.");
        std::process::exit(1);
    }

    let mut message_padded = Array1::zeros(pub_enc_data.degree);
    message_padded.slice_mut(s![..message_bits.len()]).assign(&message_bits);

    let (a1, a2) = encrypt_func(&message_padded, &pub_enc_data.pka, &pub_enc_data.pkb, pub_enc_data.degree, pub_enc_data.modulus);

    // --- Decryption Section ---
    let decrypted_message = decrypt_message(&white_data, &a1, &a2, pub_enc_data.degree, pub_enc_data.modulus);

    let decrypted_message_str = decrypted_message.iter().map(|&x| x.to_string()).collect::<String>();
    let mut recovered_message = binary_to_text(decrypted_message_str);
    recovered_message.truncate(16);
    println!("Decrypted message: {}", recovered_message);
    assert_eq!(recovered_message, message_to_encrypt);
}
