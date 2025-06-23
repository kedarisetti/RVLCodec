//! RVL Codec - A Rust implementation of the RVL (Run-Length Variable-Length) codec
//! 
//! This library provides lossless compression for depth image data using the RVL algorithm
//! as described in "Fast Lossless Depth Image Compression" by Andrew D. Wilson.
//! 
//! # Example
//! 
//! ```rust
//! use rvlcodec::RVLCodec;
//! 
//! let mut codec = RVLCodec::new();
//! let input = vec![0, 0, 1, 2, 0, 0, 3, 4, 5, 0, 0, 0, 6];
//! let mut compressed = Vec::new();
//! let mut decompressed = Vec::new();
//! 
//! // Compress
//! let compressed_size = codec.compress_rvl(&input, &mut compressed);
//! 
//! // Decompress
//! codec.decompress_rvl(&compressed, &mut decompressed, input.len());
//! 
//! // Verify
//! assert_eq!(input, decompressed);
//! ```

use pyo3::prelude::*;

/// RVL Codec for depth image compression
/// 
/// This struct implements the RVL (Run-Length Variable-Length) codec algorithm
/// for lossless compression of depth image data.
#[derive(Debug, Clone, Copy, Default)]
pub struct RVLCodec {
    p_buffer: usize,
    word: i32,
    nibbles_written: i32,
}

impl RVLCodec {
    /// Creates a new RVL codec instance
    pub fn new() -> Self {
        Self::default()
    }

    /// Compresses depth image data using the RVL algorithm
    /// 
    /// # Arguments
    /// 
    /// * `input` - Input depth image data as u16 values
    /// * `output` - Output buffer for compressed data
    /// 
    /// # Returns
    /// 
    /// The size of the compressed data in bytes
    pub fn compress_rvl(&mut self, input: &[u16], output: &mut Vec<u8>) -> usize {
        // Clear and prepare output buffer
        output.clear();
        let output_len = input.len() * 2; // Allocate enough space
        output.resize(output_len, 0);
        
        // Set up buffer pointers like C++ code
        let output_ptr = output.as_mut_ptr() as *mut i32;
        self.p_buffer = 0;
        self.word = 0;
        self.nibbles_written = 0;
        
        let mut input_index = 0;
        let mut previous: u16 = 0;
        
        while input_index < input.len() {
            // Count zeros
            let mut zeros = 0;
            while input_index < input.len() && input[input_index] == 0 {
                zeros += 1;
                input_index += 1;
            }
            self.encode_vle(zeros, output_ptr);
            
            // Count non-zeros
            let mut nonzeros = 0;
            let start_nonzero = input_index;
            while input_index < input.len() && input[input_index] != 0 {
                nonzeros += 1;
                input_index += 1;
            }
            self.encode_vle(nonzeros, output_ptr);
            
            // Encode non-zero values
            for i in 0..nonzeros {
                let current = input[start_nonzero + i as usize];
                let delta = current as i32 - previous as i32;
                let positive = (delta << 1) ^ (delta >> 31);
                self.encode_vle(positive, output_ptr);
                previous = current;
            }
        }
        
        // Write remaining nibbles
        if self.nibbles_written != 0 {
            unsafe {
                *output_ptr.offset(self.p_buffer as isize) = self.word << (4 * (8 - self.nibbles_written));
            }
            self.p_buffer += 1;
        }
        
        // Convert back to bytes
        let compressed_size = self.p_buffer * 4;
        output.truncate(compressed_size);
        
        compressed_size
    }

    /// Decompresses data back to depth image format
    /// 
    /// # Arguments
    /// 
    /// * `input` - Compressed data
    /// * `output` - Output buffer for decompressed u16 values
    /// * `num_pixels` - Number of pixels to decompress
    pub fn decompress_rvl(&mut self, input: &[u8], output: &mut Vec<u16>, num_pixels: usize) {
        output.clear();
        output.resize(num_pixels, 0);
        
        // Set up buffer pointers like C++ code
        let input_ptr = input.as_ptr() as *const i32;
        self.p_buffer = 0;
        self.word = 0;
        self.nibbles_written = 0;
        
        let mut output_index = 0;
        let mut previous: u16 = 0;
        
        while output_index < num_pixels {
            // Decode zeros
            let zeros = self.decode_vle(input_ptr);
            for _ in 0..zeros {
                if output_index < num_pixels {
                    output[output_index] = 0;
                    output_index += 1;
                }
            }
            
            // Decode non-zeros
            let nonzeros = self.decode_vle(input_ptr);
            for _ in 0..nonzeros {
                if output_index < num_pixels {
                    let positive = self.decode_vle(input_ptr);
                    let delta = (positive >> 1) ^ -(positive & 1);
                    let current = previous.wrapping_add(delta as u16);
                    output[output_index] = current;
                    output_index += 1;
                    previous = current;
                }
            }
        }
    }

    fn encode_vle(&mut self, mut value: i32, output_ptr: *mut i32) {
        loop {
            let mut nibble = value & 0x7; // lower 3 bits
            value >>= 3;
            if value != 0 {
                nibble |= 0x8; // more to come
            }
            
            self.word <<= 4;
            self.word |= nibble;
            self.nibbles_written += 1;
            
            if self.nibbles_written == 8 {
                unsafe {
                    *output_ptr.offset(self.p_buffer as isize) = self.word;
                }
                self.p_buffer += 1;
                self.nibbles_written = 0;
                self.word = 0;
            }
            
            if value == 0 {
                break;
            }
        }
    }

    fn decode_vle(&mut self, input_ptr: *const i32) -> i32 {
        let mut value = 0;
        let mut bits = 29;
        
        loop {
            if self.nibbles_written == 0 {
                unsafe {
                    self.word = *input_ptr.offset(self.p_buffer as isize);
                }
                self.p_buffer += 1;
                self.nibbles_written = 8;
            }
            
            let nibble = self.word as u32 & 0xf0000000;
            value |= ((nibble << 1) >> bits) as i32;
            self.word <<= 4;
            self.nibbles_written -= 1;
            bits -= 3;
            
            if (nibble & 0x80000000) == 0 {
                break;
            }
        }
        
        value
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_compress_decompress_rvl() {
        let mut codec = RVLCodec::new();
        let input = vec![0, 0, 1, 2, 0, 0, 3, 4, 5, 0, 0, 0, 6];
        let mut compressed = Vec::new();
        let mut decompressed = Vec::new();

        let _compressed_size = codec.compress_rvl(&input, &mut compressed);
        codec.decompress_rvl(&compressed, &mut decompressed, input.len());

        assert_eq!(input, decompressed);
    }

    #[test]
    fn test_compress_decompress_rvl_with_zeros() {
        let mut codec = RVLCodec::new();
        let input = vec![0, 0, 0, 0, 0, 0, 0, 0, 0, 0];
        let mut compressed = Vec::new();
        let mut decompressed = Vec::new();

        let _compressed_size = codec.compress_rvl(&input, &mut compressed);
        codec.decompress_rvl(&compressed, &mut decompressed, input.len());

        assert_eq!(input, decompressed);
    }

    #[test]
    fn test_compress_decompress_rvl_with_nonzeros() {
        let mut codec = RVLCodec::new();
        let input = vec![1, 2, 3, 4, 5, 6, 7, 8, 9, 10];
        let mut compressed = Vec::new();
        let mut decompressed = Vec::new();

        let _compressed_size = codec.compress_rvl(&input, &mut compressed);
        codec.decompress_rvl(&compressed, &mut decompressed, input.len());

        assert_eq!(input, decompressed);
    }

    #[test]
    fn test_compress_decompress_rvl_mixed() {
        let mut codec = RVLCodec::new();
        let input = vec![0, 1, 0, 2, 0, 3, 0, 4, 0, 5];
        let mut compressed = Vec::new();
        let mut decompressed = Vec::new();

        let _compressed_size = codec.compress_rvl(&input, &mut compressed);
        codec.decompress_rvl(&compressed, &mut decompressed, input.len());

        assert_eq!(input, decompressed);
    }

    #[test]
    fn test_default_impl() {
        let codec1 = RVLCodec::new();
        let codec2 = RVLCodec::default();
        assert_eq!(codec1.p_buffer, codec2.p_buffer);
        assert_eq!(codec1.word, codec2.word);
        assert_eq!(codec1.nibbles_written, codec2.nibbles_written);
    }
}

#[pyfunction]
fn compress_rvl(input: Vec<u16>) -> PyResult<Vec<u8>> {
    let mut codec = RVLCodec::new();
    let mut output = Vec::new();
    codec.compress_rvl(&input, &mut output);
    Ok(output)
}

#[pyfunction]
fn decompress_rvl(input: Vec<u8>, num_pixels: usize) -> PyResult<Vec<u16>> {
    let mut codec = RVLCodec::new();
    let mut output = Vec::new();
    codec.decompress_rvl(&input, &mut output, num_pixels);
    Ok(output)
}

#[pymodule]
fn rvlcodec(m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_function(wrap_pyfunction!(compress_rvl, m)?)?;
    m.add_function(wrap_pyfunction!(decompress_rvl, m)?)?;
    Ok(())
}