#![cfg_attr(not(feature = "std"), no_std)]

use ink_lang as ink;
use ink_storage::traits::SpreadAllocate;

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
mod vanchor_verifier {
    use ink_storage::traits::SpreadAllocate;
    use ink_prelude::vec::Vec;
    use crate::verifier::ArkworksVerifierBn254;

    #[ink(storage)]
    #[derive(SpreadAllocate)]
    pub struct VAnchorVerifier {
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


    impl VAnchorVerifier {
        /// Constructor that initializes the `bool` value to the given `init_value`.
        #[ink(constructor)]
        pub fn new(max_edges: u32, ins: u32, outs: u32) -> Self {
            let vk_bytes: &[u8] = match (max_edges, ins, outs) {
                (2, 2, 2) => include_bytes!("/Users/Damilare/Documents/Webb/protocol-substrate/protocol-substrate-fixtures/vanchor/bn254/x5/2-2-2/verifying_key.bin"),
                (2, 16, 2) => include_bytes!("/Users/Damilare/Documents/Webb/protocol-substrate/protocol-substrate-fixtures/vanchor/bn254/x5/2-16-2/verifying_key.bin"),
                (32, 2, 2) => include_bytes!("/Users/Damilare/Documents/Webb/protocol-substrate/protocol-substrate-fixtures/vanchor/bn254/x5/32-2-2/verifying_key.bin"),
                (32, 16, 2) => include_bytes!("/Users/Damilare/Documents/Webb/protocol-substrate/protocol-substrate-fixtures/vanchor/bn254/x5/32-16-2/verifying_key.bin"),

                _ => [0 as u8].as_slice(),
            };
            Self {
                vk_bytes: vk_bytes.to_vec()
            }
        }


        #[ink(message)]
        pub fn verify(&self, public_inp_bytes: Vec<u8>, proof_bytes: Vec<u8>) -> Result<bool> {
            ArkworksVerifierBn254::verify(&public_inp_bytes, &proof_bytes, &self.vk_bytes)
                .map_err(|_| Error::VerifierError)
        }


    }

}
