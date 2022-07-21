use ink_prelude::vec::Vec;
use ink_env::AccountId;
use ink_lang_ir::Selector;
use ink_lang_ir::Selector::;


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

/// Transforms a u8 array to a fixed size array of 1 byte
///
/// * `v` -  u8 array to transform
pub fn element_encoder_for_one_byte(v: &[u8]) -> [u8; 1] {
    let mut output = [0u8; 1];
    output.iter_mut().zip(v).for_each(|(b1, b2)| *b1 = *b2);
    output
}

/// Determines if an account is zero token address
///
/// * `account_id` - an address to determine,
pub fn is_account_id_zero(account_id: AccountId) -> bool {
    account_id == ZERO_ADDRESS.into()
}

/// Hashes
pub fn hash_function(input: &[u8]) -> [u8; 4] {
    Selector::compute(input).to_bytes();
}
