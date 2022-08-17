use ink_env::AccountId;
use ink_prelude::vec::Vec;

pub const ZERO_ADDRESS: [u8; 32] = [0; 32];

pub fn truncate_and_pad(t: &[u8]) -> Vec<u8> {
    let mut truncated_bytes = t[..20].to_vec();
    truncated_bytes.extend_from_slice(&[0u8; 12]);
    truncated_bytes
}

pub fn element_encoder(v: &[u8]) -> [u8; 32] {
    let mut output = [0u8; 32];
    output.iter_mut().zip(v).for_each(|(b1, b2)| *b1 = *b2);
    output
}

/// Transforms a u8 array to a fixed size array of 4 bytes
///
/// * `v` -  u8 array to transform
pub fn element_encoder_for_four_bytes(v: &[u8]) -> [u8; 4] {
    let mut output = [0u8; 4];
    output.iter_mut().zip(v).for_each(|(b1, b2)| *b1 = *b2);
    output
}

/// Transforms a u8 array to a fixed size array of 8 bytes
///
/// * `v` -  u8 array to transform
pub fn element_encoder_for_eight_bytes(v: &[u8]) -> [u8; 8] {
    let mut output = [0u8; 8];
    output.iter_mut().zip(v).for_each(|(b1, b2)| *b1 = *b2);
    output
}

/// Transforms a u8 array to a fixed size array of 16 bytes
///
/// * `v` -  u8 array to transform
pub fn element_encoder_for_sixteen_bytes(v: &[u8]) -> [u8; 16] {
    let mut output = [0u8; 16];
    output.iter_mut().zip(v).for_each(|(b1, b2)| *b1 = *b2);
    output
}

/// Transforms a u8 array to a fixed size array of 1 byte
///
/// * `v` -  u8 array to transform
pub fn element_encoder_for_one_byte(v: &[u8]) -> [u8; 1] {
    let mut output = [0u8; 1];
    output.iter_mut().zip(v).for_each(|(b1, b2)| *b1 = *b2);
    output
}

/// Transforms a u8 array to a fixed size array of 2 bytes
///
/// * `v` -  u8 array to transform
pub fn element_encoder_for_two_bytes(v: &[u8]) -> [u8; 2] {
    let mut output = [0u8; 2];
    output.iter_mut().zip(v).for_each(|(b1, b2)| *b1 = *b2);
    output
}

/// Determines if an account is zero token address
///
/// * `account_id` - an address to determine,
pub fn is_account_id_zero(account_id: AccountId) -> bool {
    account_id == ZERO_ADDRESS.into()
}

pub fn transform_u32_to_array_of_u8(x: u32) -> [u8; 4] {
    let b1: u8 = ((x >> 24) & 0xff) as u8;
    let b2: u8 = ((x >> 16) & 0xff) as u8;
    let b3: u8 = ((x >> 8) & 0xff) as u8;
    let b4: u8 = (x & 0xff) as u8;
    return [b1, b2, b3, b4];
}
