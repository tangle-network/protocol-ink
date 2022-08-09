use ink_env::hash::{Blake2x256, CryptoHash};

/// Returns the 4 byte BLAKE-2b 256-bit hash of an input
pub fn blake2b_256_4_bytes_output(input: &[u8]) -> [u8; 4] {
    let mut output = [0; 32];

    Blake2x256::hash(input, &mut output);
    [output[0], output[1], output[2], output[3]]
}

pub fn blake2b_256_32_bytes_output(input: &[u8]) -> [u8; 32] {
    let mut output = [0; 32];

    Blake2x256::hash(input, &mut output);

    output
}
