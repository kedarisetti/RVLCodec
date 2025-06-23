use rvlcodec::RVLCodec;

fn main() {
    println!("RVL Codec - Basic Usage Example");
    println!("===============================");
    
    // Create a new codec instance
    let mut codec = RVLCodec::new();
    
    // Example depth image data (similar to the C++ test)
    let input = vec![0, 0, 1, 2, 0, 0, 3, 4, 5, 0, 0, 0, 6];
    let mut compressed = Vec::new();
    let mut decompressed = Vec::new();
    
    println!("Original data: {:?}", input);
    println!("Original size: {} bytes", input.len() * 2);
    
    // Compress the data
    let compressed_size = codec.compress_rvl(&input, &mut compressed);
    println!("Compressed size: {} bytes", compressed_size);
    println!("Compression ratio: {:.1}%", (compressed_size as f64 / (input.len() * 2) as f64) * 100.0);
    println!("Compressed data: {:?}", compressed);
    
    // Decompress the data
    codec.decompress_rvl(&compressed, &mut decompressed, input.len());
    println!("Decompressed data: {:?}", decompressed);
    
    // Verify the result
    if input == decompressed {
        println!("✅ SUCCESS: Decompressed data matches original!");
    } else {
        println!("❌ FAILURE: Decompressed data does not match original!");
    }
    
    // Test with all zeros
    println!("\nTesting with all zeros:");
    let zeros_input = vec![0; 10];
    let mut zeros_compressed = Vec::new();
    let mut zeros_decompressed = Vec::new();
    
    let zeros_compressed_size = codec.compress_rvl(&zeros_input, &mut zeros_compressed);
    codec.decompress_rvl(&zeros_compressed, &mut zeros_decompressed, zeros_input.len());
    
    println!("All zeros compressed size: {} bytes", zeros_compressed_size);
    println!("All zeros test: {}", if zeros_input == zeros_decompressed { "✅ PASSED" } else { "❌ FAILED" });
    
    // Test with all non-zeros
    println!("\nTesting with all non-zeros:");
    let nonzeros_input = vec![1, 2, 3, 4, 5, 6, 7, 8, 9, 10];
    let mut nonzeros_compressed = Vec::new();
    let mut nonzeros_decompressed = Vec::new();
    
    let nonzeros_compressed_size = codec.compress_rvl(&nonzeros_input, &mut nonzeros_compressed);
    codec.decompress_rvl(&nonzeros_compressed, &mut nonzeros_decompressed, nonzeros_input.len());
    
    println!("All non-zeros compressed size: {} bytes", nonzeros_compressed_size);
    println!("All non-zeros test: {}", if nonzeros_input == nonzeros_decompressed { "✅ PASSED" } else { "❌ FAILED" });
} 