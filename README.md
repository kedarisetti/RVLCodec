# RVL Codec

A pure Rust implementation of the RVL (Run-Length Variable-Length) codec for depth image compression, with optional Python bindings.

## Overview

This package implements the RVL codec algorithm as described in the paper "Fast Lossless Depth Image Compression" by Andrew D. Wilson. The implementation is written entirely in Rust and provides both a native Rust library and Python bindings for easy integration.

## Features

- **Pure Rust implementation** for maximum performance and safety
- **Lossless compression** of depth image data
- **Efficient encoding** using run-length and variable-length encoding
- **Optional Python bindings** for easy integration with Python projects
- **Zero dependencies** (except for Python bindings when used)

## Installation

### Rust Library

Add to your `Cargo.toml`:

```toml
[dependencies]
rvlcodec = "0.1.0"
```

### Python Bindings (Optional)

#### Prerequisites

- Rust (latest stable version)
- Python 3.7+
- maturin (for building Python bindings)

#### Building the Python Module

```bash
# Install maturin if you haven't already
pip install maturin

# Build and install the Python module
maturin develop
```

## Usage

### Rust

```rust
use rvlcodec::RVLCodec;

fn main() {
    let mut codec = RVLCodec::new();
    let input = vec![0, 0, 1, 2, 0, 0, 3, 4, 5, 0, 0, 0, 6];
    let mut compressed = Vec::new();
    let mut decompressed = Vec::new();

    // Compress
    let compressed_size = codec.compress_rvl(&input, &mut compressed);

    // Decompress
    codec.decompress_rvl(&compressed, &mut decompressed, input.len());

    // Verify
    assert_eq!(input, decompressed);
}
```

### Python

```python
import rvlcodec
import numpy as np

# Example depth image data (u16 values)
input_data = np.array([0, 0, 1, 2, 0, 0, 3, 4, 5, 0, 0, 0, 6], dtype=np.uint16)
input_list = input_data.tolist()

# Compress the data
compressed_data = rvlcodec.compress_rvl(input_list)

# Decompress the data
decompressed_data = rvlcodec.decompress_rvl(compressed_data, len(input_list))

# Verify the result
assert input_list == decompressed_data
```

## Examples

Run the basic usage example:

```bash
cargo run --example basic_usage
```

## API Reference

### Rust API

- `RVLCodec::new() -> RVLCodec`: Creates a new codec instance
- `compress_rvl(&self, input: &[u16], output: &mut Vec<u8>) -> usize`: Compresses input data
- `decompress_rvl(&self, input: &[u8], output: &mut Vec<u16>, num_pixels: usize)`: Decompresses data

### Python API

- `compress_rvl(input: List[int]) -> bytes`: Compresses a list of u16 values
- `decompress_rvl(input: bytes, num_pixels: int) -> List[int]`: Decompresses data back to u16 values

## Algorithm

The RVL codec works by:

1. **Run-length encoding** of zero values
2. **Variable-length encoding** of non-zero values using delta encoding
3. **Efficient bit packing** using 3-bit nibbles with continuation bits

The algorithm is particularly effective for depth images which often contain large regions of zero values (background) and smooth transitions in depth values.

## Testing

Run the tests using:

```bash
# Rust tests
cargo test

# Python tests (if Python bindings are installed)
python src/test_rvlcodec.py
```

## Performance

The Rust implementation provides excellent performance:

- **Fast compression**: Optimized for depth image data patterns
- **Memory efficient**: Minimal memory overhead during compression/decompression
- **Zero-copy operations**: Where possible, avoids unnecessary data copying

## License

This project is licensed under the MIT License - see the LICENSE file for details.

## Acknowledgments

Based on the algorithm described by Andrew D. Wilson in "Fast Lossless Depth Image Compression". 