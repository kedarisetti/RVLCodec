import rvlcodec
import numpy as np

def test_encode_decode():
    # Test data matching the C++ test
    input_data = np.array([0, 0, 1, 2, 0, 0, 3, 4, 5, 0, 0, 0, 6], dtype=np.uint16)
    
    # Convert to list of u16 values
    input_list = input_data.tolist()
    
    # Compress using the new API
    encoded_data = rvlcodec.compress_rvl(input_list)
    
    # Decompress using the new API
    decoded_data = rvlcodec.decompress_rvl(encoded_data, len(input_list))
    
    print("Original input:", input_list)
    print("Compressed size:", len(encoded_data), "bytes")
    print("Compressed data:", list(encoded_data))
    print("Decoded output:", decoded_data)
    
    # Convert back to numpy for comparison
    decoded_array = np.array(decoded_data, dtype=np.uint16)
    
    if np.array_equal(input_data, decoded_array):
        print("SUCCESS: Decompressed data matches original input!")
    else:
        print("FAILURE: Decompressed data does not match original input!")
        print("Expected:", input_data)
        print("Got:", decoded_array)

def test_all_zeros():
    input_data = np.array([0, 0, 0, 0, 0, 0, 0, 0, 0, 0], dtype=np.uint16)
    input_list = input_data.tolist()
    
    encoded_data = rvlcodec.compress_rvl(input_list)
    decoded_data = rvlcodec.decompress_rvl(encoded_data, len(input_list))
    
    decoded_array = np.array(decoded_data, dtype=np.uint16)
    
    if np.array_equal(input_data, decoded_array):
        print("SUCCESS: All zeros test passed!")
    else:
        print("FAILURE: All zeros test failed!")

def test_all_nonzeros():
    input_data = np.array([1, 2, 3, 4, 5, 6, 7, 8, 9, 10], dtype=np.uint16)
    input_list = input_data.tolist()
    
    encoded_data = rvlcodec.compress_rvl(input_list)
    decoded_data = rvlcodec.decompress_rvl(encoded_data, len(input_list))
    
    decoded_array = np.array(decoded_data, dtype=np.uint16)
    
    if np.array_equal(input_data, decoded_array):
        print("SUCCESS: All non-zeros test passed!")
    else:
        print("FAILURE: All non-zeros test failed!")

if __name__ == "__main__":
    test_encode_decode()
    test_all_zeros()
    test_all_nonzeros()
    print("All tests completed!")