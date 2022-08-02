use ink_env::Error;
use ink_prelude::vec::Vec;
use protocol_ink_lib::keccak::Keccak256;

pub const SIGNATURE_LENGTH: usize = 65;

pub fn validate_ecdsa_signature(data: &[u8], signature: &[u8]) -> bool {
    if signature.len() == SIGNATURE_LENGTH {
        let mut sig = [0u8; SIGNATURE_LENGTH];
        sig[..SIGNATURE_LENGTH].copy_from_slice(&signature);

        let hash = Keccak256::hash(&data)
            .unwrap_or_else(|error| panic!("could not hash data: {:?}", error));
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

        let message = ink_prelude::format!("sig in recover  is {:?}", sig);
        ink_env::debug_println!("{}",message);

        let hash = Keccak256::hash(&data)
            .unwrap_or_else(|error| panic!("could not hash data: {:?}", error));
        let message = ink_prelude::format!("hash data  is {:?}", hash);
        ink_env::debug_println!("{}",message);
        let mut output = [0; 33];
        let result = ink_env::ecdsa_recover(&sig, &hash, &mut output);
        if result.is_err() {
            ink_env::debug_println!("result is error");
            return Err(Error::EcdsaRecoveryFailed);
        }
        let message = ink_prelude::format!("output  is {:?}", output);
        ink_env::debug_println!("{}",message);
        return Ok(output.to_vec());
    }
    ink_env::debug_println!("signature length is not 65");
    Err(Error::EcdsaRecoveryFailed)
}
