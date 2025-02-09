use std::io::{self, Write, Read};

#[derive(Debug)]
pub struct LZ77Token {
    offset: u16,  // How far back the match is
    length: u8,   // How long the match is
    next_byte: u8, // The next unmatched byte
}

/// Compresses input data using LZ77 and outputs binary tokens
pub fn lz77_compress(input: &[u8], window_size: usize) -> Vec<LZ77Token> {
    let mut tokens = Vec::new();
    let mut pos = 0;

    while pos < input.len() {
        let mut best_match = (0, 0); // (offset, length)
        
        // Search for the longest match within the sliding window
        for back in 1..=usize::min(pos, window_size) {
            let mut length = 0;
            while length < 255 &&
                  pos + length < input.len() &&
                  input[pos - back + length] == input[pos + length] {
                length += 1;
            }

            if length > best_match.1 {
                best_match = (back, length);
            }
        }

        let (offset, length) = best_match;
        let next_byte = if pos + length < input.len() {
            input[pos + length]
        } else {
            0 // End of input
        };

        tokens.push(LZ77Token { offset: offset as u16, length: length as u8, next_byte });
        pos += length + 1;
    }

    tokens
}

/// Decompresses binary LZ77 tokens back into the original byte stream
pub fn lz77_decompress(tokens: &[LZ77Token]) -> Vec<u8> {
    let mut output = Vec::new();

    for token in tokens {
        let start = output.len().saturating_sub(token.offset as usize);
        for i in 0..token.length {
            output.push(output[start + i as usize]);
        }
        output.push(token.next_byte);
    }

    output
}

/// Writes the compressed data as binary output
pub fn write_compressed_data<W: Write>(tokens: &[LZ77Token], writer: &mut W) -> io::Result<()> {
    for token in tokens {
        writer.write_all(&token.offset.to_le_bytes())?;
        writer.write_all(&[token.length])?;
        writer.write_all(&[token.next_byte])?;
    }
    Ok(())
}

/// Reads compressed binary data and reconstructs tokens
pub fn read_compressed_data<R: Read>(reader: &mut R) -> io::Result<Vec<LZ77Token>> {
    let mut tokens = Vec::new();
    let mut buffer = [0; 3];
    while reader.read_exact(&mut buffer).is_ok() {
        let offset = u16::from_le_bytes([buffer[0], buffer[1]]);
        let length = buffer[2];
        let mut next_byte = [0; 1];
        reader.read_exact(&mut next_byte)?;
        tokens.push(LZ77Token { offset, length, next_byte: next_byte[0] });
    }

    Ok(tokens)
}


#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_compress_decompress() {
        let input_data = b"ABABABAABACBAB\0";
        println!("Original Data: {:?}", input_data);

        let compressed = lz77_compress(input_data, 8);
        println!("Compressed Tokens: {:?}", compressed);

        let mut compressed_binary = Vec::new();
        write_compressed_data(&compressed, &mut compressed_binary).unwrap();
        println!("Binary Compressed Data: {:?}", compressed_binary);

        let decompressed_tokens = read_compressed_data(&mut compressed_binary.as_slice()).unwrap();
        let decompressed = lz77_decompress(&decompressed_tokens);
        println!("Decompressed Data: {:?}", decompressed);

        assert_eq!(input_data.to_vec(), decompressed);
    }
}
