use blake3::Hasher;
use std::convert::TryInto;

/// Compute Blake3 hash of data
pub fn compute_blake3(data: &[u8]) -> [u8; 32] {
    let mut hasher = Hasher::new();
    hasher.update(data);
    *hasher.finalize().as_bytes()
}

/// Validate Blake3 hash
pub fn validate_blake3(data: &[u8], expected: &[u8; 32]) -> bool {
    let computed = compute_blake3(data);
    computed == *expected
}

/// CRT-specific checksum for vector verification using Blake3
pub fn compute_crt_checksum(values: &[i64]) -> [u8; 32] {
    let bytes: Vec<u8> = values.iter()
        .flat_map(|&x| x.to_le_bytes().to_vec())
        .collect();
    compute_blake3(&bytes)
}

/// Compute rolling checksum for data integrity
pub fn rolling_checksum(data: &[u8], window_size: usize) -> Vec<[u8; 32]> {
    let mut checksums = Vec::new();
    for i in 0..data.len().saturating_sub(window_size) + 1 {
        let window = &data[i..i+window_size.min(data.len() - i)];
        checksums.push(compute_blake3(window));
    }
    checksums
}

/// Verify a sequence of values using Blake3
pub fn integer_sequence_checksum(values: &[i64]) -> [u8; 32] {
    let bytes: Vec<u8> = values.iter()
        .flat_map(|&x| x.to_le_bytes().to_vec())
        .collect();
    compute_blake3(&bytes)
}

/// Verify CRT operations by checking if values correctly map back
pub fn verify_crt_operation(original: i64, base: &[i64], expected_crt: &[i64]) -> bool {
    let crt_values = goto_crt(original, base);
    let reconstructed = goback_crt(&crt_values, base);
    
    // Also verify with Blake3 hash
    let orig_hash = compute_blake3(&original.to_le_bytes());
    let reconst_hash = compute_blake3(&reconstructed.to_le_bytes());
    
    crt_values == expected_crt && reconstructed == original && orig_hash == reconst_hash
}

/// CRT conversion
pub fn goto_crt(x: i64, base: &[i64]) -> Vec<i64> {
    base.iter().map(|&b| x.rem_euclid(b)).collect()
}

/// CRT reconstruction
pub fn goback_crt(x_b: &[i64], base: &[i64]) -> i64 {
    let mut x = 0;
    let b_prod: i64 = base.iter().product();
    for i in 0..base.len() {
        let b_i = b_prod / base[i];
        let (_, mi, _) = xgcd(b_i, base[i]);
        x = (x + (x_b[i] * b_i % b_prod).rem_euclid(b_prod) * mi % b_prod).rem_euclid(b_prod);
    }
    x.rem_euclid(b_prod)
}

/// Extended Euclidean algorithm for CRT
fn xgcd(mut b: i64, mut n: i64) -> (i64, i64, i64) {
    let mut x0 = 1;
    let mut x1 = 0;
    let mut y0 = 0;
    let mut y1 = 1;
    while n != 0 {
        let q = b / n;
        let tmp_n = n;
        n = b % n;
        b = tmp_n;

        let tmp_x1 = x1;
        x1 = x0 - q * x1;
        x0 = tmp_x1;

        let tmp_y1 = y1;
        y1 = y0 - q * y1;
        y0 = tmp_y1;
    }
    (b, x0, y0)
}

/// Format Blake3 hash as hex string
pub fn hash_to_hex(hash: &[u8; 32]) -> String {
    hash.iter()
        .map(|b| format!("{:02x}", b))
        .collect::<Vec<String>>()
        .join("")
}
