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

// Galois field multiplication
const fn galois_multiplication(a: u8, b: u8) -> u8 {
    let mut p = 0u8;
    let mut a = a;
    let mut b = b;
    let mut counter = 0;
    while counter < 8 {
        if (b & 1) != 0 {
            p ^= a;
        }
        let hi_bit_set = (a & 0x80) != 0;
        a <<= 1;
        if hi_bit_set {
            a ^= 0x1b;
        }
        b >>= 1;
        counter += 1;
    }
    p
}

// Generate multiplication tables for Galois field operations
const fn generate_mul_table(multiplier: u8) -> [u8; 256] {
    let mut table = [0u8; 256];
    let mut i = 0;
    while i < 256 {
        table[i] = galois_multiplication(i as u8, multiplier);
        i += 1;
    }
    table
}

static MUL2: [u8; 256] = generate_mul_table(2);
static MUL3: [u8; 256] = generate_mul_table(3);
static MUL9: [u8; 256] = generate_mul_table(9);
static MUL11: [u8; 256] = generate_mul_table(11);
static MUL13: [u8; 256] = generate_mul_table(13);
static MUL14: [u8; 256] = generate_mul_table(14);

pub struct AES128 {
    expanded_key: [[u8; 4]; 44],
    pub encrypt: fn(&AES128, &[u8]) -> Vec<u8>,
    pub decrypt: fn(&AES128, &[u8]) -> Vec<u8>,
    encrypt_block: fn(&AES128, &[u8; 16]) -> [u8; 16],
    decrypt_block: fn(&AES128, &[u8; 16]) -> [u8; 16],
}

impl AES128 {
    // Create a new AES-128 instance from a string key (must be 16 bytes).
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

    // Create a new AES-128 instance from a 16-byte key.
    pub fn new(key: &[u8; 16]) -> AES128 {
        AES128 {
            expanded_key: key_schedule_aes128(key),
            encrypt: encrypt_aes128,
            decrypt: decrypt_aes128,
            encrypt_block: encrypt_block_aes128,
            decrypt_block: decrypt_block_aes128,
        }
    }

    // Generate a random 16-byte key.
    pub fn generate_key() -> [u8; 16] {
        let mut rng = rand::thread_rng();
        let mut key = [0u8; 16];
        rng.fill(&mut key);
        key
    }

    // Encrypt data in CBC mode, returning IV || ciphertext.
    pub fn encrypt_cbc(&self, plaintext: &[u8]) -> Vec<u8> {
        let padded = pad_pkcs7(plaintext, 16);
        let mut rng = rand::thread_rng();
        let iv: [u8; 16] = rng.gen();
        let mut ciphertext = vec![0u8; 16 + padded.len()];
        ciphertext[0..16].copy_from_slice(&iv);
        let mut previous = iv;
        for (i, block) in padded.chunks(16).enumerate() {
            let mut xor_block = [0u8; 16];
            for j in 0..16 {
                xor_block[j] = block[j] ^ previous[j];
            }
            let encrypted = (self.encrypt_block)(self, &xor_block);
            ciphertext[16 + i * 16..16 + (i + 1) * 16].copy_from_slice(&encrypted);
            previous = encrypted;
        }
        ciphertext
    }

    // Decrypt data in CBC mode, assuming IV is the first 16 bytes of ciphertext.
    pub fn decrypt_cbc(&self, ciphertext: &[u8]) -> Option<Vec<u8>> {
        if ciphertext.len() < 16 || (ciphertext.len() - 16) % 16 != 0 {
            return None;
        }
        let iv = &ciphertext[0..16];
        let mut plaintext = Vec::with_capacity(ciphertext.len() - 16);
        let mut previous: [u8; 16] = iv.try_into().ok()?;
        for block in ciphertext[16..].chunks(16) {
            let decrypted = (self.decrypt_block)(self, &block.try_into().ok()?);
            for j in 0..16 {
                plaintext.push(decrypted[j] ^ previous[j]);
            }
            previous = block.try_into().ok()?;
        }
        unpad_pkcs7(&plaintext)
    }
}

// Clone a slice into an array.
fn clone_into_array<A, T>(slice: &[T]) -> A
where
    A: Default + AsMut<[T]>,
    T: Clone,
{
    let mut a = A::default();
    <A as AsMut<[T]>>::as_mut(&mut a).clone_from_slice(slice);
    a
}

// Generate the key schedule for AES-128.
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

// Substitute a byte using the S-box or inverse S-box.
fn substitute(byte: u8, encryption: bool) -> u8 {
    let upper_nibble = ((byte >> 4) & 0xF) as usize;
    let lower_nibble = (byte & 0xF) as usize;
    if encryption {
        AES_SBOX[upper_nibble][lower_nibble]
    } else {
        INVERSE_AES_SBOX[upper_nibble][lower_nibble]
    }
}

// Rotate a 4-byte word left by one byte.
fn rot_word(word: &[u8; 4]) -> [u8; 4] {
    [word[1], word[2], word[3], word[0]]
}

// Apply S-box substitution to a 4-byte word.
fn sub_word(word: &[u8; 4]) -> [u8; 4] {
    let mut res = [0u8; 4];
    for i in 0..4 {
        res[i] = substitute(word[i], true);
    }
    res
}

// XOR two 4-byte words.
fn xor_words(word1: &[u8; 4], word2: &[u8; 4]) -> [u8; 4] {
    let mut res = [0u8; 4];
    for i in 0..4 {
        res[i] = word1[i] ^ word2[i];
    }
    res
}

// Add the round key to the state (optimized to use slice).
fn add_round_key(state: &mut [[u8; 4]; 4], round_key: &[[u8; 4]]) {
    for i in 0..4 {
        for j in 0..4 {
            state[i][j] ^= round_key[j][i];
        }
    }
}

// Substitute all bytes in the state with the S-box.
fn sub_bytes(state: &mut [[u8; 4]; 4]) {
    for i in 0..4 {
        for j in 0..4 {
            state[i][j] = substitute(state[i][j], true);
        }
    }
}

// Substitute all bytes in the state with the inverse S-box.
fn inv_sub_bytes(state: &mut [[u8; 4]; 4]) {
    for i in 0..4 {
        for j in 0..4 {
            state[i][j] = substitute(state[i][j], false);
        }
    }
}

// Shift rows in the state.
fn shift_rows(state: &mut [[u8; 4]; 4]) {
    for i in 1..4 {
        let mut new_row = [0u8; 4];
        for j in 0..4 {
            new_row[j] = state[i][(j + i) % 4];
        }
        state[i] = new_row;
    }
}

// Inverse shift rows in the state.
fn inv_shift_rows(state: &mut [[u8; 4]; 4]) {
    for i in 1..4 {
        let mut new_row = [0u8; 4];
        for j in 0..4 {
            new_row[(j + i) % 4] = state[i][j];
        }
        state[i] = new_row;
    }
}

// Mix columns using precomputed tables.
fn mix_columns(state: &mut [[u8; 4]; 4]) {
    for i in 0..4 {
        let s0 = state[0][i];
        let s1 = state[1][i];
        let s2 = state[2][i];
        let s3 = state[3][i];
        state[0][i] = MUL2[s0 as usize] ^ MUL3[s1 as usize] ^ s2 ^ s3;
        state[1][i] = s0 ^ MUL2[s1 as usize] ^ MUL3[s2 as usize] ^ s3;
        state[2][i] = s0 ^ s1 ^ MUL2[s2 as usize] ^ MUL3[s3 as usize];
        state[3][i] = MUL3[s0 as usize] ^ s1 ^ s2 ^ MUL2[s3 as usize];
    }
}

// Inverse mix columns using precomputed tables.
fn inv_mix_columns(state: &mut [[u8; 4]; 4]) {
    for i in 0..4 {
        let s0 = state[0][i];
        let s1 = state[1][i];
        let s2 = state[2][i];
        let s3 = state[3][i];
        state[0][i] = MUL14[s0 as usize] ^ MUL11[s1 as usize] ^ MUL13[s2 as usize] ^ MUL9[s3 as usize];
        state[1][i] = MUL9[s0 as usize] ^ MUL14[s1 as usize] ^ MUL11[s2 as usize] ^ MUL13[s3 as usize];
        state[2][i] = MUL13[s0 as usize] ^ MUL9[s1 as usize] ^ MUL14[s2 as usize] ^ MUL11[s3 as usize];
        state[3][i] = MUL11[s0 as usize] ^ MUL13[s1 as usize] ^ MUL9[s2 as usize] ^ MUL14[s3 as usize];
    }
}

// Encrypt data in ECB mode (unchanged interface).
fn encrypt_aes128(aes: &AES128, bytes: &[u8]) -> Vec<u8> {
    if bytes.len() % 16 != 0 {
        panic!("Input must be multiple of 16 bytes");
    }
    let mut res = vec![0u8; bytes.len()];
    for (i, block) in bytes.chunks(16).enumerate() {
        let mut arr = [0u8; 16];
        arr.copy_from_slice(block);
        let encrypted = (aes.encrypt_block)(aes, &arr);
        res[i * 16..(i + 1) * 16].copy_from_slice(&encrypted);
    }
    res
}

// Encrypt a single block (optimized).
fn encrypt_block_aes128(aes: &AES128, bytes: &[u8; 16]) -> [u8; 16] {
    let mut state = [[0u8; 4]; 4];
    for i in 0..16 {
        state[i % 4][i / 4] = bytes[i];
    }

    add_round_key(&mut state, &aes.expanded_key[0..4]);

    for round in 1..10 {
        sub_bytes(&mut state);
        shift_rows(&mut state);
        mix_columns(&mut state);
        add_round_key(&mut state, &aes.expanded_key[round * 4..round * 4 + 4]);
    }

    sub_bytes(&mut state);
    shift_rows(&mut state);
    add_round_key(&mut state, &aes.expanded_key[40..44]);

    let mut res = [0u8; 16];
    for i in 0..4 {
        for j in 0..4 {
            res[j * 4 + i] = state[i][j];
        }
    }
    res
}

// Decrypt data in ECB mode (unchanged interface).
fn decrypt_aes128(aes: &AES128, bytes: &[u8]) -> Vec<u8> {
    if bytes.len() % 16 != 0 {
        panic!("Input must be multiple of 16 bytes");
    }
    let mut res = vec![0u8; bytes.len()];
    for (i, block) in bytes.chunks(16).enumerate() {
        let mut arr = [0u8; 16];
        arr.copy_from_slice(block);
        let decrypted = (aes.decrypt_block)(aes, &arr);
        res[i * 16..(i + 1) * 16].copy_from_slice(&decrypted);
    }
    res
}

// Decrypt a single block (optimized).
fn decrypt_block_aes128(aes: &AES128, bytes: &[u8; 16]) -> [u8; 16] {
    let mut state = [[0u8; 4]; 4];
    for i in 0..16 {
        state[i % 4][i / 4] = bytes[i];
    }

    add_round_key(&mut state, &aes.expanded_key[40..44]);

    for round in (1..10).rev() {
        inv_shift_rows(&mut state);
        inv_sub_bytes(&mut state);
        add_round_key(&mut state, &aes.expanded_key[round * 4..round * 4 + 4]);
        inv_mix_columns(&mut state);
    }

    inv_shift_rows(&mut state);
    inv_sub_bytes(&mut state);
    add_round_key(&mut state, &aes.expanded_key[0..4]);

    let mut res = [0u8; 16];
    for i in 0..4 {
        for j in 0..4 {
            res[j * 4 + i] = state[i][j];
        }
    }
    res
}

// Pad data using PKCS#7 padding.
pub fn pad_pkcs7(data: &[u8], block_size: usize) -> Vec<u8> {
    let padding_len = block_size - (data.len() % block_size);
    let mut padded = data.to_vec();
    padded.extend(vec![padding_len as u8; padding_len]);
    padded
}

// Remove PKCS#7 padding from data.
pub fn unpad_pkcs7(data: &[u8]) -> Option<Vec<u8>> {
    if data.is_empty() {
        return None;
    }
    let padding_len = *data.last().unwrap() as usize;
    if padding_len == 0 || padding_len > 16 || padding_len > data.len() {
        return None;
    }
    for &byte in &data[data.len() - padding_len..] {
        if byte != padding_len as u8 {
            return None;
        }
    }
    Some(data[..data.len() - padding_len].to_vec())
}


#[cfg(test)]
mod tests_aes {
    use super::*;

    #[test]
    fn test_aes_key_generation() {
        let key1 = AES128::generate_key();
        let key2 = AES128::generate_key();
        assert_ne!(key1, key2); // Les clés doivent être différentes
        assert_eq!(key1.len(), 16);
        assert_eq!(key2.len(), 16);
    }

    #[test]
    fn test_aes_new_from_str() {
        let key_str = "YELLOW SUBMARINE";
        let aes = AES128::new_from_str(key_str);
        // Test que l'instance se crée sans panic
        assert_eq!(key_str.len(), 16);
    }

    #[test]
    #[should_panic(expected = "Key needs to be 16 bytes long")]
    fn test_aes_invalid_key_length() {
        AES128::new_from_str("trop court");
    }

    #[test]
    fn test_ecb_encrypt_decrypt() {
        let key = "YELLOW SUBMARINE".as_bytes();
        let aes = AES128::new(&key.try_into().unwrap());
        
        // Test avec un bloc de 16 bytes
        let plaintext = b"Hello World!!!!!"; // 16 bytes exactement
        let ciphertext = (aes.encrypt)(&aes, plaintext);
        let decrypted = (aes.decrypt)(&aes, &ciphertext);
        
        assert_eq!(plaintext, &decrypted[..]);
        assert_ne!(plaintext.to_vec(), ciphertext);
    }

    #[test]
    fn test_ecb_multiple_blocks() {
        let key = AES128::generate_key();
        let aes = AES128::new(&key);
        
        // Test avec 32 bytes (2 blocs)
        let plaintext = b"This is a test message with 32by"; // 32 bytes
        let ciphertext = (aes.encrypt)(&aes, plaintext);
        let decrypted = (aes.decrypt)(&aes, &ciphertext);
        
        assert_eq!(plaintext, &decrypted[..]);
        assert_eq!(ciphertext.len(), 32);
    }

    #[test]
    #[should_panic(expected = "Input must be multiple of 16 bytes")]
    fn test_ecb_invalid_input_length() {
        let key = AES128::generate_key();
        let aes = AES128::new(&key);
        let plaintext = b"Invalid length"; // 14 bytes
        (aes.encrypt)(&aes, plaintext);
    }

    #[test]
    fn test_encrypt_block() {
        let key = "YELLOW SUBMARINE".as_bytes();
        let aes = AES128::new(&key.try_into().unwrap());
        
        let block = [0u8; 16];
        let encrypted = (aes.encrypt_block)(&aes, &block);
        let decrypted = (aes.decrypt_block)(&aes, &encrypted);
        
        assert_eq!(block, decrypted);
        assert_ne!(block, encrypted);
    }

    #[test]
    fn test_cbc_encrypt_decrypt() {
        let key = AES128::generate_key();
        let aes = AES128::new(&key);
        
        let plaintext = b"Hello World! This is a test message for CBC mode encryption.";
        let ciphertext = aes.encrypt_cbc(plaintext);
        let decrypted = aes.decrypt_cbc(&ciphertext).unwrap();
        
        assert_eq!(plaintext, &decrypted[..]);
        assert!(ciphertext.len() > plaintext.len()); // IV + padding
    }

    #[test]
    fn test_cbc_with_padding() {
        let key = AES128::generate_key();
        let aes = AES128::new(&key);
        
        // Test avec différentes tailles
        for size in 1..50 {
            let plaintext = vec![b'A'; size];
            let ciphertext = aes.encrypt_cbc(&plaintext);
            let decrypted = aes.decrypt_cbc(&ciphertext).unwrap();
            assert_eq!(plaintext, decrypted);
        }
    }

    #[test]
    fn test_cbc_invalid_ciphertext() {
        let key = AES128::generate_key();
        let aes = AES128::new(&key);
        
        // Ciphertext trop court
        assert!(aes.decrypt_cbc(&[0u8; 15]).is_none());
        
        // Taille invalide
        assert!(aes.decrypt_cbc(&[0u8; 20]).is_none());
    }

    #[test]
    fn test_pkcs7_padding() {
        // Test padding normal
        let data = b"Hello";
        let padded = pad_pkcs7(data, 16);
        assert_eq!(padded.len(), 16);
        assert_eq!(&padded[5..], &[11u8; 11]); // 11 bytes de padding avec valeur 11
        
        // Test unpadding
        let unpadded = unpad_pkcs7(&padded).unwrap();
        assert_eq!(unpadded, data);
    }

    #[test]
    fn test_pkcs7_full_block() {
        // Quand la taille est déjà un multiple du bloc
        let data = b"Exactly 16 bytes"; // 16 bytes
        let padded = pad_pkcs7(data, 16);
        assert_eq!(padded.len(), 32); // Un bloc entier de padding
        assert_eq!(&padded[16..], &[16u8; 16]);
        
        let unpadded = unpad_pkcs7(&padded).unwrap();
        assert_eq!(unpadded, data);
    }

    #[test]
    fn test_pkcs7_invalid_padding() {
        // Padding invalide
        assert!(unpad_pkcs7(&[]).is_none());
        assert!(unpad_pkcs7(&[0u8]).is_none());
        assert!(unpad_pkcs7(&[17u8]).is_none()); // > 16
        assert!(unpad_pkcs7(&[1u8, 2u8]).is_none()); // Inconsistent
    }

    #[test]
    fn test_galois_multiplication() {
        // Tests connus pour la multiplication dans GF(2^8)
        assert_eq!(galois_multiplication(0x02, 0x01), 0x02);
        assert_eq!(galois_multiplication(0x02, 0x02), 0x04);
        assert_eq!(galois_multiplication(0x02, 0x80), 0x1b);
        assert_eq!(galois_multiplication(0x03, 0x01), 0x03);
    }

    #[test]
    fn test_substitute() {
        // Test S-box
        assert_eq!(substitute(0x00, true), 0x63);
        assert_eq!(substitute(0x01, true), 0x7c);
        
        // Test inverse S-box
        assert_eq!(substitute(0x63, false), 0x00);
        assert_eq!(substitute(0x7c, false), 0x01);
        
        // Test round-trip
        for i in 0..256 {
            let byte = i as u8;
            let substituted = substitute(byte, true);
            let back = substitute(substituted, false);
            assert_eq!(byte, back);
        }
    }

    #[test]
    fn test_rot_word() {
        let word = [0x01, 0x02, 0x03, 0x04];
        let rotated = rot_word(&word);
        assert_eq!(rotated, [0x02, 0x03, 0x04, 0x01]);
    }

    #[test]
    fn test_xor_words() {
        let word1 = [0x01, 0x02, 0x03, 0x04];
        let word2 = [0x05, 0x06, 0x07, 0x08];
        let result = xor_words(&word1, &word2);
        assert_eq!(result, [0x04, 0x04, 0x04, 0x0c]);
    }

    #[test]
    fn test_key_schedule() {
        let key = [0u8; 16];
        let expanded = key_schedule_aes128(&key);
        
        // Les 4 premiers mots doivent être la clé originale
        for i in 0..4 {
            assert_eq!(expanded[i], [0u8; 4]);
        }
        
        // Vérifier que nous avons 44 mots (11 round keys * 4)
        assert_eq!(expanded.len(), 44);
    }

    #[test]
    fn test_aes_deterministic() {
        let key = "YELLOW SUBMARINE".as_bytes();
        let aes = AES128::new(&key.try_into().unwrap());
        
        let plaintext = b"Hello World!!!!!";
        let ciphertext1 = (aes.encrypt)(&aes, plaintext);
        let ciphertext2 = (aes.encrypt)(&aes, plaintext);
        
        // ECB doit être déterministe
        assert_eq!(ciphertext1, ciphertext2);
    }

    #[test]
    fn test_cbc_randomness() {
        let key = AES128::generate_key();
        let aes = AES128::new(&key);
        
        let plaintext = b"Hello World!";
        let ciphertext1 = aes.encrypt_cbc(plaintext);
        let ciphertext2 = aes.encrypt_cbc(plaintext);
        
        // CBC avec IV aléatoire doit produire des résultats différents
        assert_ne!(ciphertext1, ciphertext2);
        
        // Mais le déchiffrement doit donner le même plaintext
        assert_eq!(aes.decrypt_cbc(&ciphertext1).unwrap(), plaintext);
        assert_eq!(aes.decrypt_cbc(&ciphertext2).unwrap(), plaintext);
    }

    #[test]
    fn test_empty_input_cbc() {
        let key = AES128::generate_key();
        let aes = AES128::new(&key);
        
        let plaintext = b"";
        let ciphertext = aes.encrypt_cbc(plaintext);
        let decrypted = aes.decrypt_cbc(&ciphertext).unwrap();
        
        assert_eq!(decrypted, plaintext);
        assert_eq!(ciphertext.len(), 32); // IV (16) + un bloc de padding (16)
    }

    #[test]
    fn test_mul_tables() {
        // Vérifier quelques valeurs des tables de multiplication
        assert_eq!(MUL2[0x01], 0x02);
        assert_eq!(MUL3[0x01], 0x03);
        assert_eq!(MUL9[0x01], 0x09);
        assert_eq!(MUL11[0x01], 0x0b);
        assert_eq!(MUL13[0x01], 0x0d);
        assert_eq!(MUL14[0x01], 0x0e);
    }

    #[test]
    fn test_large_data() {
        let key = AES128::generate_key();
        let aes = AES128::new(&key);
        
        // Test avec des données plus importantes
        let plaintext = vec![0x42; 1024]; // 1KB de données
        let ciphertext = aes.encrypt_cbc(&plaintext);
        let decrypted = aes.decrypt_cbc(&ciphertext).unwrap();
        
        assert_eq!(plaintext, decrypted);
    }
}
