use blake3::Hasher;

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


#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_compute_blake3() {
        let empty_data = b"";
        let empty_hash = [
            0xaf, 0x13, 0x49, 0xb9, 0xf5, 0xf9, 0xa1, 0xa6,
            0xa0, 0x40, 0x4d, 0xea, 0x36, 0xdc, 0xc9, 0x49,
            0x9b, 0xcb, 0x25, 0xc9, 0xad, 0xc1, 0x12, 0xb7,
            0xcc, 0x9a, 0x93, 0xca, 0xe4, 0x1f, 0x32, 0x62,
        ];
        assert_eq!(compute_blake3(empty_data), empty_hash);

        let data = b"abc";
        assert_eq!(compute_blake3(data).len(), 32);
    }

    #[test]
    fn test_validate_blake3_match() {
        let data = b"hello";
        let hash = compute_blake3(data);
        assert!(validate_blake3(data, &hash));
    }

    #[test]
    fn test_validate_blake3_mismatch() {
        let data = b"hello";
        let wrong_hash = [0; 32];
        assert!(!validate_blake3(data, &wrong_hash));
    }

    #[test]
    fn test_integer_sequence_checksum() {
        let values = [1, 2, 3];
        let expected_bytes = &[1, 0, 0, 0, 0, 0, 0, 0, 2, 0, 0, 0, 0, 0, 0, 0, 3, 0, 0, 0, 0, 0, 0, 0];
        let expected_hash = compute_blake3(expected_bytes);
        assert_eq!(integer_sequence_checksum(&values), expected_hash);
        assert_eq!(compute_crt_checksum(&values), expected_hash);
    }

    #[test]
    fn test_rolling_checksum_small_window() {
        let data = b"abc";
        let window_size = 2;
        let checksums = rolling_checksum(data, window_size);
        assert_eq!(checksums.len(), 2); // "ab", "bc"
        assert_eq!(checksums[0], compute_blake3(b"ab"));
        assert_eq!(checksums[1], compute_blake3(b"bc"));
    }

    #[test]
    fn test_rolling_checksum_window_equal_data() {
        let data = b"abc";
        let window_size = 3;
        let checksums = rolling_checksum(data, window_size);
        assert_eq!(checksums.len(), 1);
        assert_eq!(checksums[0], compute_blake3(data));
    }

    #[test]
    fn test_rolling_checksum_window_larger_than_data() {
        let data = b"abc";
        let window_size = 5;
        let checksums = rolling_checksum(data, window_size);
        assert_eq!(checksums.len(), 1);
        assert_eq!(checksums[0], compute_blake3(data));
    }

    #[test]
    fn test_crt_roundtrip() {
        let base = [2, 3, 5];
        let x = 7;
        let crt = goto_crt(x, &base);
        assert_eq!(crt, vec![1, 1, 2]); // 7 % 2 = 1, 7 % 3 = 1, 7 % 5 = 2
        let reconstructed = goback_crt(&crt, &base);
        assert_eq!(reconstructed, x);
    }

    #[test]
    fn test_crt_tampered() {
        let base = [2, 3, 5];
        let x = 7;
        let mut crt = goto_crt(x, &base); // [1, 1, 2]
        crt[0] = 0; // Tamper
        let reconstructed = goback_crt(&crt, &base);
        assert_ne!(reconstructed, x);
    }

    #[test]
    fn test_verify_crt_operation_correct() {
        let original = 7;
        let base = [2, 3, 5];
        let expected_crt = vec![1, 1, 2];
        assert!(verify_crt_operation(original, &base, &expected_crt));
    }

    #[test]
    fn test_verify_crt_operation_incorrect() {
        let original = 7;
        let base = [2, 3, 5];
        let incorrect_crt = vec![0, 1, 2];
        assert!(!verify_crt_operation(original, &base, &incorrect_crt));
    }
}
