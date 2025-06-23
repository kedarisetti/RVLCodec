# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Added
- Initial implementation of RVL codec
- Rust library with Python bindings
- Comprehensive test suite
- GitHub Actions CI/CD pipeline
- Documentation and examples

### Changed
- Converted from C++ to pure Rust implementation
- Improved API design with idiomatic Rust patterns

## [0.1.0] - 2024-01-XX

### Added
- RVL codec implementation for depth image compression
- Rust library API with `RVLCodec` struct
- Python bindings via PyO3
- Lossless compression/decompression of u16 depth data
- Run-length encoding for zero values
- Variable-length encoding for non-zero deltas
- 3-bit nibble packing with continuation bits
- Comprehensive test coverage
- Documentation and usage examples
- GitHub Actions workflows for CI/CD
- Crate publishing configuration 