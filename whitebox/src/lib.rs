pub mod lattice;
pub mod create_wb;
pub mod encrypt;
pub mod decrypt;

pub use crate::lattice::{PubEncData, WhiteData, NTRUVector};
pub use crate::decrypt::decrypt_message;
pub use crate::encrypt::encrypt_func;

use numpy::ndarray::Array1;

/// Creates a whitebox configuration, generating public encryption data and whitebox data.
///
/// # Returns
/// A tuple containing:
/// - `PubEncData`: Public encryption parameters (degree, modulus, public keys).
/// - `WhiteData`: Whitebox data for decryption (root, unroot, ninv, beta, beta_p, etc.).
pub fn create_whitebox() -> (PubEncData, WhiteData) {
    create_wb::create_whitebox()
}

/// Encrypts a message using the provided public encryption data.
///
/// # Arguments
/// - `message`: The message as a string to be encrypted.
/// - `pub_enc_data`: The public encryption data containing degree, modulus, and public keys.
///
/// # Returns
/// A tuple of two `NTRUVector`s representing the ciphertext (a1, a2).
///
/// # Panics
/// Panics if the message is too long for the given degree.
pub fn encrypt_message(message: &str, pub_enc_data: &PubEncData) -> (NTRUVector, NTRUVector) {
    let message_bytes = message.as_bytes();
    let mut message_bits_vec = Vec::new();
    for byte in message_bytes {
        for i in 0..8 {
            message_bits_vec.push((byte >> (7 - i)) & 1);
        }
    }
    let message_bits = Array1::from_vec(message_bits_vec.iter().map(|&x| x as i64).collect::<Vec<i64>>());

    if message_bits.len() > pub_enc_data.degree {
        panic!("Error: Message too long for encryption.");
    }

    let mut message_padded = Array1::zeros(pub_enc_data.degree);
    message_padded.slice_mut(numpy::ndarray::s![..message_bits.len()]).assign(&message_bits);

    encrypt_func(&message_padded, &pub_enc_data.pka, &pub_enc_data.pkb, pub_enc_data.degree, pub_enc_data.modulus)
}

/// Decrypts a ciphertext using the provided whitebox data.
///
/// # Arguments
/// - `white_data`: The whitebox data required for decryption.
/// - `a1`: The first part of the ciphertext (NTRUVector).
/// - `a2`: The second part of the ciphertext (NTRUVector).
/// - `degree`: The degree of the NTRU polynomial.
/// - `modulus`: The modulus used in the NTRU scheme.
///
/// # Returns
/// The decrypted message as a string.
pub fn decrypt_to_text(white_data: &WhiteData, a1: &NTRUVector, a2: &NTRUVector, degree: usize, modulus: i64) -> String {
    let decrypted_message = decrypt_message(white_data, a1, a2, degree, modulus);
    let decrypted_message_str = decrypted_message.iter().map(|&x| x.to_string()).collect::<String>();
    binary_to_text(decrypted_message_str)
}

/// Converts a binary string to its ASCII text representation.
///
/// # Arguments
/// - `binary_str`: A string of binary digits (0s and 1s).
///
/// # Returns
/// The ASCII text decoded from the binary string.
pub fn binary_to_text(binary_str: String) -> String {
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
