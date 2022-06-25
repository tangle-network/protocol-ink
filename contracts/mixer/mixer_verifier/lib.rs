#![cfg_attr(not(feature = "std"), no_std)]

use ink_env::call::FromAccountId;
use ink_env::Environment;
use ink_lang as ink;
use ink_prelude::vec::Vec;
use ink_storage::traits::SpreadAllocate;

pub use self::mixer_verifier::{MixerVerifier, MixerVerifierRef};

impl SpreadAllocate for MixerVerifierRef {
    fn allocate_spread(_ptr: &mut ink_primitives::KeyPtr) -> Self {
        FromAccountId::from_account_id([0; 32].into())
    }
}

#[ink::chain_extension]
pub trait VerifyProof {
    type ErrorCode = VerifyProofErr;

    #[ink(extension = 1101, returns_result = false)]
    fn verify_proof(public_inp_bytes: (Vec<u8>, Vec<u8>)) -> bool;
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, scale::Encode, scale::Decode)]
#[cfg_attr(feature = "std", derive(scale_info::TypeInfo))]
pub enum VerifyProofErr {
    FailGetVerifyProof,
}

impl ink_env::chain_extension::FromStatusCode for VerifyProofErr {
    fn from_status_code(status_code: u32) -> Result<(), Self> {
        match status_code {
            0 => Ok(()),
            1 => Err(Self::FailGetVerifyProof),
            _ => panic!("encountered unknown status code"),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "std", derive(scale_info::TypeInfo))]
pub enum CustomEnvironment {}

impl Environment for CustomEnvironment {
    const MAX_EVENT_TOPICS: usize = <ink_env::DefaultEnvironment as Environment>::MAX_EVENT_TOPICS;

    type AccountId = <ink_env::DefaultEnvironment as Environment>::AccountId;
    type Balance = <ink_env::DefaultEnvironment as Environment>::Balance;
    type Hash = <ink_env::DefaultEnvironment as Environment>::Hash;
    type BlockNumber = <ink_env::DefaultEnvironment as Environment>::BlockNumber;
    type Timestamp = <ink_env::DefaultEnvironment as Environment>::Timestamp;

    type ChainExtension = VerifyProof;
}

mod verifier {
    use ark_crypto_primitives::{Error, SNARK};
    use ark_ec::PairingEngine;
    use ark_groth16::{Groth16, Proof, VerifyingKey};
    use ark_serialize::CanonicalDeserialize;
    use ark_std::marker::PhantomData;
    use arkworks_native_gadgets::to_field_elements;
    pub struct ArkworksVerifierGroth16<E: PairingEngine>(PhantomData<E>);

    pub fn verify_groth16<E: PairingEngine>(
        vk: &VerifyingKey<E>,
        public_inputs: &[E::Fr],
        proof: &Proof<E>,
    ) -> Result<bool, Error> {
        let res = Groth16::<E>::verify(vk, public_inputs, proof)?;
        let message = ink_prelude::format!("finished verifying groth16");
        ink_env::debug_println!("{}", &message);
        Ok(res)
    }

    impl<E: PairingEngine> ArkworksVerifierGroth16<E> {
        pub fn verify(
            public_inp_bytes: &[u8],
            proof_bytes: &[u8],
            vk_bytes: &[u8],
        ) -> Result<bool, Error> {
            let public_input_field_elts = to_field_elements::<E::Fr>(public_inp_bytes)?;
            let proof = Proof::<E>::deserialize(proof_bytes)?;

            let message = ink_prelude::format!("vk_bytes {:?}", vk_bytes);
            ink_env::debug_println!("{}", &message);

            let vk = VerifyingKey::<E>::deserialize(vk_bytes)?;

            ink_env::debug_println!("{}", &message);
            let message = ink_prelude::format!("verifying groth16");
            ink_env::debug_println!("{}", &message);
            let res = verify_groth16::<E>(&vk, &public_input_field_elts, &proof).unwrap();
            let message = ink_prelude::format!("res is {:?}", res);
            ink_env::debug_println!("{}", &message);
            Ok(res)
        }
    }

    use ark_bn254::Bn254;
    pub type ArkworksVerifierBn254 = ArkworksVerifierGroth16<Bn254>;
}

#[ink::contract(env = crate::CustomEnvironment)]
pub mod mixer_verifier {
    use super::VerifyProofErr;
    use crate::verifier::ArkworksVerifierBn254;
    use ink_prelude::vec::Vec;
    use ink_storage::traits::SpreadAllocate;

    #[ink(storage)]
    #[derive(SpreadAllocate)]
    pub struct MixerVerifier {
        vk_bytes: Vec<u8>,
    }

    /// The verifier error types.
    #[derive(Debug, PartialEq, Eq, scale::Encode, scale::Decode)]
    #[cfg_attr(feature = "std", derive(scale_info::TypeInfo))]
    pub enum Error {
        /// Returned if error verifying
        VerifierError,
    }

    /// The verifier result type.
    pub type Result<T> = core::result::Result<T, Error>;

    impl MixerVerifier {
        /// Constructor that initializes the `bool` value to the given `init_value`.
        #[ink(constructor)]
        pub fn new() -> Self {
            // Self { vk_bytes }
            Self {
                vk_bytes: [
                    168, 210, 158, 164, 6, 41, 190, 118, 47, 42, 18, 189, 167, 204, 69, 185, 152,
                    163, 76, 67, 169, 108, 76, 103, 68, 200, 199, 169, 0, 232, 248, 10, 94, 236,
                    230, 250, 87, 113, 72, 156, 176, 48, 111, 73, 154, 217, 26, 51, 208, 21, 159,
                    41, 120, 99, 50, 215, 130, 219, 135, 10, 9, 128, 68, 10, 161, 34, 90, 210, 60,
                    14, 71, 108, 125, 54, 183, 150, 230, 169, 181, 2, 64, 132, 27, 75, 233, 85,
                    209, 62, 165, 77, 216, 218, 1, 18, 142, 12, 249, 32, 148, 180, 89, 255, 136,
                    39, 128, 171, 223, 62, 23, 132, 192, 109, 246, 200, 92, 0, 6, 253, 127, 37,
                    151, 227, 233, 5, 45, 146, 21, 39, 79, 8, 252, 79, 148, 220, 129, 41, 242, 154,
                    87, 141, 193, 127, 94, 166, 14, 168, 93, 44, 136, 167, 130, 148, 183, 146, 219,
                    242, 250, 141, 48, 151, 59, 128, 202, 107, 86, 116, 99, 182, 144, 184, 179,
                    168, 247, 14, 182, 70, 130, 39, 53, 142, 95, 49, 110, 184, 21, 10, 146, 21, 43,
                    117, 53, 25, 196, 202, 130, 127, 236, 23, 247, 40, 61, 21, 34, 135, 103, 181,
                    107, 115, 110, 201, 73, 143, 57, 254, 91, 81, 26, 138, 245, 3, 182, 58, 25,
                    151, 4, 0, 0, 0, 0, 0, 0, 0, 185, 200, 107, 190, 62, 94, 243, 73, 13, 93, 180,
                    120, 190, 10, 121, 51, 147, 78, 75, 90, 20, 142, 44, 1, 96, 43, 71, 92, 131,
                    226, 80, 8, 145, 220, 68, 166, 254, 47, 51, 29, 169, 182, 111, 53, 98, 57, 138,
                    118, 38, 119, 247, 183, 201, 93, 58, 130, 169, 105, 142, 12, 246, 33, 33, 40,
                    214, 235, 39, 163, 241, 26, 187, 82, 169, 138, 80, 155, 241, 80, 47, 9, 71,
                    233, 171, 156, 29, 114, 160, 134, 20, 12, 13, 49, 104, 102, 214, 36, 194, 42,
                    105, 248, 223, 181, 149, 126, 53, 216, 211, 211, 80, 179, 200, 62, 233, 94,
                    136, 151, 199, 183, 109, 65, 136, 22, 131, 210, 86, 28, 217, 25,
                ]
                .to_vec(),
            }
        }

        /// A message that can be called on instantiated contracts.
        /// This one flips the value of the stored `bool` from `true`
        /// to `false` and vice versa.
        #[ink(message)]
        pub fn verify(&self, public_inp_bytes: Vec<u8>, proof_bytes: Vec<u8>) -> Result<bool> {
            ink_env::debug_println!("sending chain extension verification");
            let tuple: (Vec<u8>, Vec<u8>) = (public_inp_bytes.clone(), proof_bytes.clone());
            // Get the on-chain proof verification result
            let proof_result = self.env().extension().verify_proof(tuple).unwrap();
            let message = ink_prelude::format!("result is {:?}", proof_result);
            ink_env::debug_println!("{}", &message);

            Ok(proof_result)
            /*ArkworksVerifierBn254::verify(&public_inp_bytes, &proof_bytes, &self.vk_bytes)
            .map_err(|_| Error::VerifierError)*/
        }
    }
}
