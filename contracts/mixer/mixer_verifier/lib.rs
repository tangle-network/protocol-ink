#![cfg_attr(not(feature = "std"), no_std)]

use ink_env::call::FromAccountId;
use ink_lang as ink;
use ink_storage::traits::SpreadAllocate;

pub use self::mixer_verifier::{MixerVerifier, MixerVerifierRef};

impl SpreadAllocate for MixerVerifierRef {
    fn allocate_spread(_ptr: &mut ink_primitives::KeyPtr) -> Self {
        FromAccountId::from_account_id([0; 32].into())
    }
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
        Ok(res)
    }

    impl<E: PairingEngine> ArkworksVerifierGroth16<E> {
        pub fn verify(
            public_inp_bytes: &[u8],
            proof_bytes: &[u8],
            vk_bytes: &[u8],
        ) -> Result<bool, Error> {
            let public_input_field_elts = to_field_elements::<E::Fr>(public_inp_bytes)?;
            let vk = VerifyingKey::<E>::deserialize(vk_bytes)?;
            let proof = Proof::<E>::deserialize(proof_bytes)?;
            let res = verify_groth16::<E>(&vk, &public_input_field_elts, &proof)?;
            Ok(res)
        }
    }

    use ark_bn254::Bn254;
    pub type ArkworksVerifierBn254 = ArkworksVerifierGroth16<Bn254>;
}

#[ink::contract]
pub mod mixer_verifier {
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
                    149, 229, 101, 88, 100, 4, 250, 96, 247, 71, 232, 13, 37, 96, 25, 0, 58, 59,
                    170, 192, 153, 186, 201, 53, 52, 68, 134, 78, 216, 35, 197, 5, 83, 19, 241,
                    225, 63, 142, 39, 92, 3, 1, 25, 4, 252, 1, 130, 162, 250, 160, 179, 215, 46,
                    105, 48, 133, 16, 66, 135, 144, 211, 190, 87, 32, 51, 149, 196, 231, 25, 186,
                    151, 253, 9, 153, 116, 162, 73, 52, 228, 245, 181, 64, 90, 183, 2, 84, 109,
                    174, 68, 148, 3, 146, 249, 112, 249, 34, 29, 149, 51, 170, 157, 9, 62, 225, 20,
                    28, 134, 177, 247, 178, 245, 104, 43, 44, 177, 89, 167, 231, 183, 14, 21, 169,
                    179, 87, 1, 255, 63, 14, 237, 227, 197, 61, 95, 71, 149, 217, 230, 213, 181,
                    111, 234, 166, 87, 61, 245, 106, 111, 134, 68, 3, 180, 133, 254, 201, 163, 43,
                    68, 58, 39, 44, 71, 248, 1, 42, 219, 209, 87, 147, 116, 246, 182, 164, 180,
                    211, 14, 100, 171, 231, 14, 161, 184, 148, 0, 176, 13, 91, 112, 103, 22, 18,
                    131, 30, 57, 146, 21, 163, 120, 223, 167, 249, 0, 105, 63, 18, 204, 58, 237,
                    109, 67, 102, 105, 102, 235, 19, 205, 179, 178, 146, 35, 239, 156, 232, 55, 14,
                    229, 217, 74, 39, 17, 110, 192, 137, 75, 61, 159, 182, 249, 237, 135, 107, 124,
                    199, 128, 20, 174, 96, 237, 164, 104, 248, 43, 67, 159, 124, 246, 28, 25, 235,
                    183, 117, 7, 123, 96, 228, 98, 133, 216, 77, 209, 229, 81, 145, 157, 233, 229,
                    58, 82, 40, 77, 22, 85, 95, 45, 155, 95, 23, 133, 0, 56, 67, 98, 184, 27, 76,
                    162, 216, 21, 110, 153, 49, 224, 182, 85, 129, 189, 185, 211, 201, 2, 105, 50,
                    152, 168, 138, 71, 220, 31, 165, 62, 14, 111, 160, 204, 128, 219, 117, 243,
                    122, 1, 177, 55, 171, 50, 74, 244, 108, 124, 11, 154, 255, 77, 58, 171, 123,
                    21, 186, 226, 119, 76, 244, 149, 21, 254, 213, 233, 157, 62, 58, 130, 65, 212,
                    236, 44, 0, 127, 92, 213, 50, 14, 78, 80, 78, 223, 114, 211, 214, 59, 85, 19,
                    203, 10, 165, 115, 46, 61, 185, 12, 17, 236, 168, 75, 158, 30, 65, 164, 46,
                    130, 86, 63, 188, 241, 206, 81, 110, 247, 214, 121, 189, 108, 120, 88, 24, 213,
                    78, 235, 44, 47, 221, 204, 216, 160, 138, 85, 104, 81, 178, 152, 209, 238, 25,
                    36, 18, 187, 122, 196, 203, 214, 113, 45, 186, 54, 225, 177, 231, 49, 154, 45,
                    38, 7, 0, 0, 0, 0, 0, 0, 0, 189, 115, 140, 254, 246, 241, 56, 164, 153, 131, 0,
                    157, 196, 69, 233, 245, 187, 215, 48, 113, 146, 18, 137, 55, 228, 134, 82, 236,
                    155, 193, 22, 18, 125, 83, 73, 78, 190, 117, 151, 107, 190, 138, 174, 202, 238,
                    40, 121, 75, 194, 184, 175, 217, 104, 151, 96, 138, 166, 53, 227, 222, 154,
                    106, 173, 11, 110, 146, 86, 167, 202, 211, 174, 113, 97, 214, 82, 164, 235, 12,
                    145, 99, 43, 193, 164, 196, 140, 5, 2, 243, 238, 126, 154, 87, 249, 187, 229,
                    34, 25, 197, 127, 48, 247, 198, 168, 186, 110, 195, 149, 218, 87, 101, 191,
                    118, 160, 65, 205, 29, 232, 190, 241, 65, 204, 11, 223, 229, 17, 36, 166, 24,
                    117, 18, 255, 192, 165, 58, 91, 163, 129, 20, 9, 219, 244, 188, 225, 123, 193,
                    119, 2, 27, 182, 162, 144, 140, 98, 90, 51, 2, 53, 35, 94, 37, 248, 101, 162,
                    238, 114, 174, 12, 38, 202, 61, 35, 14, 50, 238, 155, 44, 67, 213, 164, 67,
                    140, 68, 122, 210, 2, 140, 155, 219, 208, 199, 249, 43, 84, 42, 135, 78, 99,
                    107, 45, 48, 118, 108, 100, 75, 161, 27, 216, 5, 126, 49, 225, 149, 105, 71,
                    71, 13, 36, 120, 170, 38, 75, 76, 190, 21, 189, 156, 114, 84, 130, 220, 164,
                    58, 84, 16, 151, 142, 184, 179, 128, 243, 52, 207, 50, 241, 91, 166, 96, 134,
                    66, 95, 81, 91, 53, 104, 93, 18, 109, 95, 39, 83, 146, 120, 226, 215, 1, 77, 8,
                    38, 230, 252, 76, 234, 128, 63, 21, 150, 249, 42, 234, 75, 136, 163, 54, 135,
                    53, 60, 83, 12, 115, 55, 76, 63, 95, 77, 50, 231, 110, 79, 90, 48, 250, 191,
                    228, 101, 99, 22, 106, 186, 110, 150, 125, 31, 151, 198, 122, 31, 187, 74, 133,
                    4, 103, 91, 134, 233, 39, 37, 156, 65, 237, 116, 240, 194, 64, 248, 250, 128,
                    243, 112, 60, 164, 67, 153, 225, 58, 70, 112, 202, 102, 104, 88, 40, 7, 185,
                    61, 237, 6, 69, 37, 192, 243, 77, 202, 26, 106, 168, 194, 199, 140, 203, 241,
                    113, 238, 199, 231, 104, 111, 7, 151, 117, 170, 140, 226, 220, 22, 140, 248,
                    33, 221, 197, 68, 134, 214, 15, 210, 122, 80, 174, 65, 33, 37, 191, 6, 1, 29,
                    92, 10, 177, 86, 202, 16, 112, 253, 201, 158, 14, 34, 247, 60, 137, 217, 46,
                    185, 190, 140, 203, 220, 144, 89, 191, 127, 170, 190, 155, 145, 64, 176, 148,
                    68, 44, 76, 42, 125, 222, 197, 136, 199, 118, 47,
                ]
                .to_vec(),
            }
        }

        /// A message that can be called on instantiated contracts.
        /// This one flips the value of the stored `bool` from `true`
        /// to `false` and vice versa.
        #[ink(message)]
        pub fn verify(&self, public_inp_bytes: Vec<u8>, proof_bytes: Vec<u8>) -> Result<bool> {
            ArkworksVerifierBn254::verify(&public_inp_bytes, &proof_bytes, &self.vk_bytes)
                .map_err(|_| Error::VerifierError)
        }
    }
}
