use std::io::{self, Write, Read};

#[derive(Debug)]
pub struct LZ77Token {
    offset: u8,  // How far back the match is
    length: u8,   // How long the match is
    next_byte: u8, // The next unmatched byte
}

/// Finds the longest match within the search window
fn find_longest_match(data: &[u8], pos: usize) -> (u8, u8) {
    let mut best_offset = 0u8;
    let mut best_len = 0u8;
    let window_size = 255;
    let start = if pos > window_size { pos - window_size } else { 0 };

    for offset in start..pos {
        let len = matching(data, offset, pos);
        if len > best_len {
            best_offset = (pos - offset) as u8;
            best_len = len;
        }
    }
    (best_offset, best_len)
}

fn matching(data: &[u8], offset: usize, end: usize) -> u8 {
    let mut offset = offset;
    let mut pos = end;
    let mut len = 0u8;

    while offset < pos && pos < data.len() && data[offset] == data[pos] && len < 255 {
        offset += 1;
        pos += 1;
        len += 1;
    }
    len
}

/// LZ77 Compression
pub fn lz77_compress(input: &[u8]) -> Vec<LZ77Token> {
    let mut tokens = Vec::new();
    let mut pos = 0;

    while pos < input.len() {
        let (offset, length) = find_longest_match(input, pos);
        
        let next_byte = input.get(pos + length as usize).copied().unwrap_or(0);
        
        tokens.push(LZ77Token {
            offset,
            length,
            next_byte,
        });

        pos += length as usize + 1;
    }
    tokens
}

/// LZ77 Decompression
pub fn lz77_decompress(tokens: &[LZ77Token]) -> Vec<u8> {
    let mut output = Vec::new();
    
    for token in tokens {
        if token.offset > 0 {
            let start = output.len().saturating_sub(token.offset as usize);
            for i in 0..token.length {
                let copied_byte = output.get(start + i as usize).copied().unwrap_or(0);
                output.push(copied_byte);
            }
        }
        output.push(token.next_byte);
    }
    output
}


#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_compress_decompress() {
        let input_data = b"ABABABAABACBAB\0";
        
        println!("Test: {:?}", input_data);
        let compressed = lz77_compress(input_data);
        let decompressed = lz77_decompress(&compressed);
        assert_eq!(input_data.to_vec(), decompressed);
    }

    #[test]
    fn test_empty_input() {
        let input_data = b"\0";
        
        println!("Empty: {:?}", input_data);
        let compressed = lz77_compress(input_data);
        let decompressed = lz77_decompress(&compressed);
        assert_eq!(input_data.to_vec(), decompressed);
    }

    #[test]
    fn test_single_character() {
        let input_data = b"A\0";
        
        println!("Single character: {:?}\0", input_data);
        let compressed = lz77_compress(input_data);
        let decompressed = lz77_decompress(&compressed);
        assert_eq!(input_data.to_vec(), decompressed);
    }

    #[test]
    fn test_repeated_patterns() {
        let input_data = b"AAABBBCCCABC\0";
        println!("Repeated: {:?}", input_data);
        let compressed = lz77_compress(input_data);
        let decompressed = lz77_decompress(&compressed);
        assert_eq!(input_data.to_vec(), decompressed);
    }
}
