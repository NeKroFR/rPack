use std::convert::AsMut;
use rand::Rng;

static AES_SBOX: [[u8; 16]; 16] = [
    [0x63, 0x7c, 0x77, 0x7b, 0xf2, 0x6b, 0x6f, 0xc5, 0x30, 0x01, 0x67, 0x2b, 0xfe, 0xd7, 0xab, 0x76],
    [0xca, 0x82, 0xc9, 0x7d, 0xfa, 0x59, 0x47, 0xf0, 0xad, 0xd4, 0xa2, 0xaf, 0x9c, 0xa4, 0x72, 0xc0],
    [0xb7, 0xfd, 0x93, 0x26, 0x36, 0x3f, 0xf7, 0xcc, 0x34, 0xa5, 0xe5, 0xf1, 0x71, 0xd8, 0x31, 0x15],
    [0x04, 0xc7, 0x23, 0xc3, 0x18, 0x96, 0x05, 0x9a, 0x07, 0x12, 0x80, 0xe2, 0xeb, 0x27, 0xb2, 0x75],
    [0x09, 0x83, 0x2c, 0x1a, 0x1b, 0x6e, 0x5a, 0xa0, 0x52, 0x3b, 0xd6, 0xb3, 0x29, 0xe3, 0x2f, 0x84],
    [0x53, 0xd1, 0x00, 0xed, 0x20, 0xfc, 0xb1, 0x5b, 0x6a, 0xcb, 0xbe, 0x39, 0x4a, 0x4c, 0x58, 0xcf],
    [0xd0, 0xef, 0xaa, 0xfb, 0x43, 0x4d, 0x33, 0x85, 0x45, 0xf9, 0x02, 0x7f, 0x50, 0x3c, 0x9f, 0xa8],
    [0x51, 0xa3, 0x40, 0x8f, 0x92, 0x9d, 0x38, 0xf5, 0xbc, 0xb6, 0xda, 0x21, 0x10, 0xff, 0xf3, 0xd2],
    [0xcd, 0x0c, 0x13, 0xec, 0x5f, 0x97, 0x44, 0x17, 0xc4, 0xa7, 0x7e, 0x3d, 0x64, 0x5d, 0x19, 0x73],
    [0x60, 0x81, 0x4f, 0xdc, 0x22, 0x2a, 0x90, 0x88, 0x46, 0xee, 0xb8, 0x14, 0xde, 0x5e, 0x0b, 0xdb],
    [0xe0, 0x32, 0x3a, 0x0a, 0x49, 0x06, 0x24, 0x5c, 0xc2, 0xd3, 0xac, 0x62, 0x91, 0x95, 0xe4, 0x79],
    [0xe7, 0xc8, 0x37, 0x6d, 0x8d, 0xd5, 0x4e, 0xa9, 0x6c, 0x56, 0xf4, 0xea, 0x65, 0x7a, 0xae, 0x08],
    [0xba, 0x78, 0x25, 0x2e, 0x1c, 0xa6, 0xb4, 0xc6, 0xe8, 0xdd, 0x74, 0x1f, 0x4b, 0xbd, 0x8b, 0x8a],
    [0x70, 0x3e, 0xb5, 0x66, 0x48, 0x03, 0xf6, 0x0e, 0x61, 0x35, 0x57, 0xb9, 0x86, 0xc1, 0x1d, 0x9e],
    [0xe1, 0xf8, 0x98, 0x11, 0x69, 0xd9, 0x8e, 0x94, 0x9b, 0x1e, 0x87, 0xe9, 0xce, 0x55, 0x28, 0xdf],
    [0x8c, 0xa1, 0x89, 0x0d, 0xbf, 0xe6, 0x42, 0x68, 0x41, 0x99, 0x2d, 0x0f, 0xb0, 0x54, 0xbb, 0x16],
];

static INVERSE_AES_SBOX: [[u8; 16]; 16] = [
    [0x52, 0x09, 0x6a, 0xd5, 0x30, 0x36, 0xa5, 0x38, 0xbf, 0x40, 0xa3, 0x9e, 0x81, 0xf3, 0xd7, 0xfb],
    [0x7c, 0xe3, 0x39, 0x82, 0x9b, 0x2f, 0xff, 0x87, 0x34, 0x8e, 0x43, 0x44, 0xc4, 0xde, 0xe9, 0xcb],
    [0x54, 0x7b, 0x94, 0x32, 0xa6, 0xc2, 0x23, 0x3d, 0xee, 0x4c, 0x95, 0x0b, 0x42, 0xfa, 0xc3, 0x4e],
    [0x08, 0x2e, 0xa1, 0x66, 0x28, 0xd9, 0x24, 0xb2, 0x76, 0x5b, 0xa2, 0x49, 0x6d, 0x8b, 0xd1, 0x25],
    [0x72, 0xf8, 0xf6, 0x64, 0x86, 0x68, 0x98, 0x16, 0xd4, 0xa4, 0x5c, 0xcc, 0x5d, 0x65, 0xb6, 0x92],
    [0x6c, 0x70, 0x48, 0x50, 0xfd, 0xed, 0xb9, 0xda, 0x5e, 0x15, 0x46, 0x57, 0xa7, 0x8d, 0x9d, 0x84],
    [0x90, 0xd8, 0xab, 0x00, 0x8c, 0xbc, 0xd3, 0x0a, 0xf7, 0xe4, 0x58, 0x05, 0xb8, 0xb3, 0x45, 0x06],
    [0xd0, 0x2c, 0x1e, 0x8f, 0xca, 0x3f, 0x0f, 0x02, 0xc1, 0xaf, 0xbd, 0x03, 0x01, 0x13, 0x8a, 0x6b],
    [0x3a, 0x91, 0x11, 0x41, 0x4f, 0x67, 0xdc, 0xea, 0x97, 0xf2, 0xcf, 0xce, 0xf0, 0xb4, 0xe6, 0x73],
    [0x96, 0xac, 0x74, 0x22, 0xe7, 0xad, 0x35, 0x85, 0xe2, 0xf9, 0x37, 0xe8, 0x1c, 0x75, 0xdf, 0x6e],
    [0x47, 0xf1, 0x1a, 0x71, 0x1d, 0x29, 0xc5, 0x89, 0x6f, 0xb7, 0x62, 0x0e, 0xaa, 0x18, 0xbe, 0x1b],
    [0xfc, 0x56, 0x3e, 0x4b, 0xc6, 0xd2, 0x79, 0x20, 0x9a, 0xdb, 0xc0, 0xfe, 0x78, 0xcd, 0x5a, 0xf4],
    [0x1f, 0xdd, 0xa8, 0x33, 0x88, 0x07, 0xc7, 0x31, 0xb1, 0x12, 0x10, 0x59, 0x27, 0x80, 0xec, 0x5f],
    [0x60, 0x51, 0x7f, 0xa9, 0x19, 0xb5, 0x4a, 0x0d, 0x2d, 0xe5, 0x7a, 0x9f, 0x93, 0xc9, 0x9c, 0xef],
    [0xa0, 0xe0, 0x3b, 0x4d, 0xae, 0x2a, 0xf5, 0xb0, 0xc8, 0xeb, 0xbb, 0x3c, 0x83, 0x53, 0x99, 0x61],
    [0x17, 0x2b, 0x04, 0x7e, 0xba, 0x77, 0xd6, 0x26, 0xe1, 0x69, 0x14, 0x63, 0x55, 0x21, 0x0c, 0x7d],
];

static RC: [u8; 11] = [0x00, 0x01, 0x02, 0x04, 0x08, 0x10, 0x20, 0x40, 0x80, 0x1B, 0x36];

pub struct AES128 {
    expanded_key: [[u8; 4]; 44],
    pub encrypt: fn(&AES128, &[u8]) -> Vec<u8>,
    pub decrypt: fn(&AES128, &[u8]) -> Vec<u8>,
    encrypt_block: fn(&AES128, &[u8; 16]) -> [u8; 16],
    decrypt_block: fn(&AES128, &[u8; 16]) -> [u8; 16],
}

impl AES128 {
    pub fn new_from_str(key: &str) -> AES128 {
        let key_bytes = key.as_bytes();
        if key_bytes.len() != 16 {
            panic!("Key needs to be 16 bytes long");
        }

        AES128 {
            expanded_key: key_schedule_aes128(&clone_into_array(key_bytes)),
            encrypt: encrypt_aes128,
            decrypt: decrypt_aes128,
            encrypt_block: encrypt_block_aes128,
            decrypt_block: decrypt_block_aes128,
        }
    }

    pub fn new(key: &[u8; 16]) -> AES128 {
        AES128 {
            expanded_key: key_schedule_aes128(key),
            encrypt: encrypt_aes128,
            decrypt: decrypt_aes128,
            encrypt_block: encrypt_block_aes128,
            decrypt_block: decrypt_block_aes128,
        }
    }

    pub fn generate_key() -> [u8; 16] {
        let mut rng = rand::thread_rng();
        let mut key = [0u8; 16];
        rng.fill(&mut key);
        key
    }
}

fn clone_into_array<A, T>(slice: &[T]) -> A
where
    A: Default + AsMut<[T]>,
    T: Clone,
{
    let mut a = A::default();
    <A as AsMut<[T]>>::as_mut(&mut a).clone_from_slice(slice);
    a
}

fn key_schedule_aes128(key_bytes: &[u8; 16]) -> [[u8; 4]; 44] {
    let mut original_key = [[0u8; 4]; 4];
    let mut expanded_key = [[0u8; 4]; 44];
    let n = 4;

    for i in 0..16 {
        original_key[i / 4][i % 4] = key_bytes[i];
    }

    for i in 0..44 {
        if i < n {
            expanded_key[i] = original_key[i];
        } else if i >= n && i % n == 0 {
            let mut rcon = [0u8; 4];
            rcon[0] = RC[i / n];
            expanded_key[i] = xor_words(
                &xor_words(&expanded_key[i - n], &sub_word(&rot_word(&expanded_key[i - 1]))),
                &rcon,
            );
        } else {
            expanded_key[i] = xor_words(&expanded_key[i - n], &expanded_key[i - 1]);
        }
    }

    expanded_key
}

fn substitute(byte: u8, encryption: bool) -> u8 {
    let upper_nibble = ((byte >> 4) & 0xF) as usize;
    let lower_nibble = (byte & 0xF) as usize;
    if encryption {
        AES_SBOX[upper_nibble][lower_nibble]
    } else {
        INVERSE_AES_SBOX[upper_nibble][lower_nibble]
    }
}

fn rot_word(word: &[u8; 4]) -> [u8; 4] {
    [word[1], word[2], word[3], word[0]]
}

fn sub_word(word: &[u8; 4]) -> [u8; 4] {
    let mut result = [0u8; 4];
    for i in 0..4 {
        result[i] = substitute(word[i], true);
    }
    result
}

fn xor_words(word1: &[u8; 4], word2: &[u8; 4]) -> [u8; 4] {
    let mut result = [0u8; 4];
    for i in 0..4 {
        result[i] = word1[i] ^ word2[i];
    }
    result
}

fn add_round_key(state: &mut [[u8; 4]; 4], key: &[[u8; 4]; 4]) {
    for i in 0..4 {
        for j in 0..4 {
            state[i][j] ^= key[j][i];
        }
    }
}

fn sub_bytes(state: &mut [[u8; 4]; 4]) {
    for i in 0..4 {
        for j in 0..4 {
            state[i][j] = substitute(state[i][j], true);
        }
    }
}

fn inv_sub_bytes(state: &mut [[u8; 4]; 4]) {
    for i in 0..4 {
        for j in 0..4 {
            state[i][j] = substitute(state[i][j], false);
        }
    }
}

fn shift_rows(state: &mut [[u8; 4]; 4]) {
    for i in 1..4 {
        let mut new_row = [0u8; 4];
        for j in 0..4 {
            new_row[j] = state[i][(j + i) % 4];
        }
        state[i] = new_row;
    }
}

fn inv_shift_rows(state: &mut [[u8; 4]; 4]) {
    for i in 1..4 {
        let mut new_row = [0u8; 4];
        for j in 0..4 {
            new_row[(j + i) % 4] = state[i][j];
        }
        state[i] = new_row;
    }
}

fn galois_multiplication(a: u8, b: u8) -> u8 {
    let mut p = 0u8;
    let mut a = a;
    let mut b = b;
    for _ in 0..8 {
        if (b & 1) != 0 {
            p ^= a;
        }
        let hi_bit_set = (a & 0x80) != 0;
        a <<= 1;
        if hi_bit_set {
            a ^= 0x1b;
        }
        b >>= 1;
    }
    p
}

fn mix_columns(state: &mut [[u8; 4]; 4]) {
    for i in 0..4 {
        let s0 = state[0][i];
        let s1 = state[1][i];
        let s2 = state[2][i];
        let s3 = state[3][i];

        state[0][i] = galois_multiplication(s0, 2)
            ^ galois_multiplication(s1, 3)
            ^ s2
            ^ s3;
        state[1][i] = s0
            ^ galois_multiplication(s1, 2)
            ^ galois_multiplication(s2, 3)
            ^ s3;
        state[2][i] = s0
            ^ s1
            ^ galois_multiplication(s2, 2)
            ^ galois_multiplication(s3, 3);
        state[3][i] = galois_multiplication(s0, 3)
            ^ s1
            ^ s2
            ^ galois_multiplication(s3, 2);
    }
}

fn inv_mix_columns(state: &mut [[u8; 4]; 4]) {
    for i in 0..4 {
        let s0 = state[0][i];
        let s1 = state[1][i];
        let s2 = state[2][i];
        let s3 = state[3][i];

        state[0][i] = galois_multiplication(s0, 0x0e)
            ^ galois_multiplication(s1, 0x0b)
            ^ galois_multiplication(s2, 0x0d)
            ^ galois_multiplication(s3, 0x09);
        state[1][i] = galois_multiplication(s0, 0x09)
            ^ galois_multiplication(s1, 0x0e)
            ^ galois_multiplication(s2, 0x0b)
            ^ galois_multiplication(s3, 0x0d);
        state[2][i] = galois_multiplication(s0, 0x0d)
            ^ galois_multiplication(s1, 0x09)
            ^ galois_multiplication(s2, 0x0e)
            ^ galois_multiplication(s3, 0x0b);
        state[3][i] = galois_multiplication(s0, 0x0b)
            ^ galois_multiplication(s1, 0x0d)
            ^ galois_multiplication(s2, 0x09)
            ^ galois_multiplication(s3, 0x0e);
    }
}

fn encrypt_aes128(aes: &AES128, bytes: &[u8]) -> Vec<u8> {
    if bytes.len() % 16 != 0 {
        panic!("Input must be multiple of 16 bytes");
    }

    let mut result = vec![0u8; bytes.len()];
    for (i, block) in bytes.chunks(16).enumerate() {
        let mut arr = [0u8; 16];
        arr.copy_from_slice(block);
        let encrypted = (aes.encrypt_block)(aes, &arr);
        result[i * 16..(i + 1) * 16].copy_from_slice(&encrypted);
    }
    result
}

fn encrypt_block_aes128(aes: &AES128, bytes: &[u8; 16]) -> [u8; 16] {
    let mut state = [[0u8; 4]; 4];
    for i in 0..16 {
        state[i % 4][i / 4] = bytes[i];
    }

    add_round_key(&mut state, &clone_into_array(&aes.expanded_key[0..4]));

    for round in 1..10 {
        sub_bytes(&mut state);
        shift_rows(&mut state);
        mix_columns(&mut state);
        add_round_key(
            &mut state,
            &clone_into_array(&aes.expanded_key[round * 4..(round + 1) * 4]),
        );
    }

    sub_bytes(&mut state);
    shift_rows(&mut state);
    add_round_key(&mut state, &clone_into_array(&aes.expanded_key[40..44]));

    let mut result = [0u8; 16];
    for i in 0..4 {
        for j in 0..4 {
            result[j * 4 + i] = state[i][j];
        }
    }
    result
}

fn decrypt_aes128(aes: &AES128, bytes: &[u8]) -> Vec<u8> {
    if bytes.len() % 16 != 0 {
        panic!("Input must be multiple of 16 bytes");
    }

    let mut result = vec![0u8; bytes.len()];
    for (i, block) in bytes.chunks(16).enumerate() {
        let mut arr = [0u8; 16];
        arr.copy_from_slice(block);
        let decrypted = (aes.decrypt_block)(aes, &arr);
        result[i * 16..(i + 1) * 16].copy_from_slice(&decrypted);
    }
    result
}

fn decrypt_block_aes128(aes: &AES128, bytes: &[u8; 16]) -> [u8; 16] {
    let mut state = [[0u8; 4]; 4];
    for i in 0..16 {
        state[i % 4][i / 4] = bytes[i];
    }

    add_round_key(&mut state, &clone_into_array(&aes.expanded_key[40..44]));

    for round in (1..10).rev() {
        inv_shift_rows(&mut state);
        inv_sub_bytes(&mut state);
        add_round_key(
            &mut state,
            &clone_into_array(&aes.expanded_key[round * 4..(round + 1) * 4]),
        );
        inv_mix_columns(&mut state);
    }

    inv_shift_rows(&mut state);
    inv_sub_bytes(&mut state);
    add_round_key(&mut state, &clone_into_array(&aes.expanded_key[0..4]));

    let mut result = [0u8; 16];
    for i in 0..4 {
        for j in 0..4 {
            result[j * 4 + i] = state[i][j];
        }
    }
    result
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_encrypt_decrypt() {
        let key = b"Meow Meow Meow!";
        let aes = AES128::new(key);
        let plaintext = [b'A'; 16];
        let ciphertext = encrypt_aes128(&aes, &plaintext);
        let decrypted = decrypt_aes128(&aes, &ciphertext);
        assert_eq!(plaintext.to_vec(), decrypted);
    }
}
