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
    word_index: usize,
    word: u32,
    nibbles_written: u8,
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
        self.compress_rvl_checked(input, output).unwrap_or(0)
    }

    /// Compresses depth image data using the RVL algorithm, returning errors for invalid input
    pub fn compress_rvl_checked(
        &mut self,
        input: &[u16],
        output: &mut Vec<u8>,
    ) -> Result<usize, RvlError> {
        output.clear();
        output.reserve(input.len());

        self.word_index = 0;
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
            let zeros = u32::try_from(zeros).map_err(|_| RvlError::InputTooLarge)?;
            self.encode_vle(zeros, output);

            // Count non-zeros
            let mut nonzeros = 0;
            let start_nonzero = input_index;
            while input_index < input.len() && input[input_index] != 0 {
                nonzeros += 1;
                input_index += 1;
            }
            let nonzeros = u32::try_from(nonzeros).map_err(|_| RvlError::InputTooLarge)?;
            self.encode_vle(nonzeros, output);

            // Encode non-zero values
            for i in 0..nonzeros {
                let current = input[start_nonzero + i as usize];
                let delta = current as i32 - previous as i32;
                let positive = ((delta << 1) ^ (delta >> 31)) as u32;
                self.encode_vle(positive, output);
                previous = current;
            }
        }

        // Write remaining nibbles
        if self.nibbles_written != 0 {
            self.word <<= 4 * (8 - self.nibbles_written as u32);
            self.flush_word(output);
        }

        Ok(output.len())
    }

    /// Decompresses data back to depth image format
    ///
    /// # Arguments
    ///
    /// * `input` - Compressed data
    /// * `output` - Output buffer for decompressed u16 values
    /// * `num_pixels` - Number of pixels to decompress
    pub fn decompress_rvl(&mut self, input: &[u8], output: &mut Vec<u16>, num_pixels: usize) {
        if self
            .decompress_rvl_checked(input, output, num_pixels)
            .is_err()
        {
            output.clear();
            output.resize(num_pixels, 0);
        }
    }

    /// Decompresses data back to depth image format, returning errors for invalid input
    pub fn decompress_rvl_checked(
        &mut self,
        input: &[u8],
        output: &mut Vec<u16>,
        num_pixels: usize,
    ) -> Result<(), RvlError> {
        #[allow(unknown_lints, clippy::manual_is_multiple_of)]
        if input.len() % 4 != 0 {
            return Err(RvlError::InvalidInputLength);
        }

        let mut words = Vec::with_capacity(input.len() / 4);
        for chunk in input.chunks_exact(4) {
            words.push(u32::from_le_bytes([chunk[0], chunk[1], chunk[2], chunk[3]]));
        }

        output.clear();
        output.resize(num_pixels, 0);

        self.word_index = 0;
        self.word = 0;
        self.nibbles_written = 0;

        let mut output_index = 0;
        let mut previous: u16 = 0;

        while output_index < num_pixels {
            // Decode zeros
            let zeros = self.decode_vle(&words)? as usize;
            for _ in 0..zeros {
                if output_index < num_pixels {
                    output[output_index] = 0;
                    output_index += 1;
                }
            }

            // Decode non-zeros
            let nonzeros = self.decode_vle(&words)? as usize;
            for _ in 0..nonzeros {
                if output_index < num_pixels {
                    let positive = self.decode_vle(&words)?;
                    let delta = ((positive >> 1) as i32) ^ (-((positive & 1) as i32));
                    let current = previous.wrapping_add(delta as u16);
                    output[output_index] = current;
                    output_index += 1;
                    previous = current;
                }
            }
        }

        Ok(())
    }

    fn encode_vle(&mut self, mut value: u32, output: &mut Vec<u8>) {
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
                self.flush_word(output);
            }

            if value == 0 {
                break;
            }
        }
    }

    fn decode_vle(&mut self, input_words: &[u32]) -> Result<u32, RvlError> {
        let mut value = 0u32;
        let mut bits = 29u32;

        loop {
            if self.nibbles_written == 0 {
                self.word = *input_words
                    .get(self.word_index)
                    .ok_or(RvlError::UnexpectedEof)?;
                self.word_index += 1;
                self.nibbles_written = 8;
            }

            let nibble = self.word & 0xf0000000;
            value |= (nibble << 1) >> bits;
            self.word <<= 4;
            self.nibbles_written -= 1;

            if (nibble & 0x80000000) == 0 {
                break;
            }

            bits = bits.checked_sub(3).ok_or(RvlError::InvalidCodeword)?;
        }

        Ok(value)
    }

    fn flush_word(&mut self, output: &mut Vec<u8>) {
        output.extend_from_slice(&self.word.to_le_bytes());
        self.word_index += 1;
        self.nibbles_written = 0;
        self.word = 0;
    }
}

#[derive(Debug)]
pub enum RvlError {
    InputTooLarge,
    InvalidInputLength,
    UnexpectedEof,
    InvalidCodeword,
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
        assert_eq!(codec1.word_index, codec2.word_index);
        assert_eq!(codec1.word, codec2.word);
        assert_eq!(codec1.nibbles_written, codec2.nibbles_written);
    }
}

#[pyfunction]
fn compress_rvl(input: Vec<u16>) -> PyResult<Vec<u8>> {
    let mut codec = RVLCodec::new();
    let mut output = Vec::new();
    if let Err(err) = codec.compress_rvl_checked(&input, &mut output) {
        return Err(pyo3::exceptions::PyValueError::new_err(format!(
            "{err:?}"
        )));
    }
    Ok(output)
}

#[pyfunction]
fn decompress_rvl(input: Vec<u8>, num_pixels: usize) -> PyResult<Vec<u16>> {
    let mut codec = RVLCodec::new();
    let mut output = Vec::new();
    if let Err(err) = codec.decompress_rvl_checked(&input, &mut output, num_pixels) {
        return Err(pyo3::exceptions::PyValueError::new_err(format!(
            "{err:?}"
        )));
    }
    Ok(output)
}

#[pymodule]
fn rvlcodec(m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_function(wrap_pyfunction!(compress_rvl, m)?)?;
    m.add_function(wrap_pyfunction!(decompress_rvl, m)?)?;
    Ok(())
}
