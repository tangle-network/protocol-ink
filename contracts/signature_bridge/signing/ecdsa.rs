use ink_env::hash::{HashOutput, Keccak256};
use ink_env::Error;
use ink_prelude::vec::Vec;

pub const SIGNATURE_LENGTH: usize = 65;

pub fn validate_ecdsa_signature(data: &[u8], signature: &[u8]) -> bool {
    if signature.len() == SIGNATURE_LENGTH {
        let mut sig = [0u8; SIGNATURE_LENGTH];
        sig[..SIGNATURE_LENGTH].copy_from_slice(&signature);

        let mut hash = <Keccak256 as HashOutput>::Type::default();
        let result = ink_env::hash_bytes::<Keccak256>(data, &mut hash);

        let mut output = [0; 33];
        return ink_env::ecdsa_recover(&sig, &hash, &mut output).is_ok();
    } else {
        return false;
    }
}

pub fn recover_ecdsa_pub_key(data: &[u8], signature: &[u8]) -> Result<Vec<u8>, Error> {
    if signature.len() == SIGNATURE_LENGTH {
        let mut sig = [0u8; SIGNATURE_LENGTH];
        sig[..SIGNATURE_LENGTH].copy_from_slice(&signature);

        let mut hash = <Keccak256 as HashOutput>::Type::default();
        let result = ink_env::hash_bytes::<Keccak256>(data, &mut hash);

        let mut output = [0; 33];
        let result = ink_env::ecdsa_recover(&sig, &hash, &mut output);
        if result.is_err() {
            return Err(Error::EcdsaRecoveryFailed);
        }
        return Ok(output.to_vec());
    }
    Err(Error::EcdsaRecoveryFailed)
}
