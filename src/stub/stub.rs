#![no_std]
#![no_main]

extern crate panic_halt;

use core::arch::asm;


// TODO
/*
#[no_mangle]
pub extern "C" fn _start() {
    let old_entry: u64;
    unsafe {
        asm!(
            "mov {0}, [rip + old_entry]",
            out(reg) old_entry
        );
        
        let enc_data_ptr: *mut u8 = 0x601000 as *mut u8;
        let enc_data_len: usize = 128;
        
        let key: [u8; 16] = *b"0123456789ABCDEF";
        let iv: [u8; 16] = *b"INIT_VECTOR_1234";
        
        decrypt_aes128(enc_data_ptr, enc_data_len, &key, &iv);
        
        decompress_lz77_in_place(enc_data_ptr, enc_data_len);
        
        asm!(
            "jmp {0}",
            in(reg) old_entry
        );
    }
}
*/

/*
fn decrypt_aes128(data_ptr: *mut u8, length: usize, key: &[u8; 16], iv: &[u8; 16]) {
    unsafe {
        let data_slice = core::slice::from_raw_parts_mut(data_ptr, length);
        
    }
}
*/

fn decompress_lz77_in_place(data_ptr: *mut u8, length: usize) {
    unsafe {
        let compressed_data = core::slice::from_raw_parts(data_ptr, length);
        let mut output_buffer = [0u8; 256];

        let mut read_pos = 0;
        let mut write_pos = 0;

        while read_pos < length {
            let byte = compressed_data[read_pos];

            if byte == 0 {
                break;
            } else if byte & 0x80 != 0 {
                let offset = (byte & 0x7F) as usize;
                let length = compressed_data[read_pos + 1] as usize;
                read_pos += 2;

                for i in 0..length {
                    output_buffer[write_pos] = output_buffer[write_pos - offset];
                    write_pos += 1;
                }
            } else {
                output_buffer[write_pos] = byte;
                write_pos += 1;
                read_pos += 1;
            }
        }
        
        let decompressed_data = &output_buffer[..write_pos];
        let output_slice = core::slice::from_raw_parts_mut(data_ptr, write_pos);
        output_slice.copy_from_slice(decompressed_data);
    }
}

#[link_section = ".bss"]
static mut old_entry: u64 = 0;
