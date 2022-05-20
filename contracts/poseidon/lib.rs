#![cfg_attr(not(feature = "std"), no_std)]
#![feature(min_specialization)]

use ink_env::call::FromAccountId;
use ink_lang as ink;
use ink_storage::traits::SpreadAllocate;

pub use self::poseidon::{Poseidon, PoseidonRef};

impl SpreadAllocate for PoseidonRef {
    fn allocate_spread(_ptr: &mut ink_primitives::KeyPtr) -> Self {
        FromAccountId::from_account_id([0; 32].into())
    }
}

mod hasher {
    use ark_crypto_primitives::Error;
    use ark_ff::{BigInteger, PrimeField};
    use ark_std::{marker::PhantomData, vec::Vec};
    use arkworks_native_gadgets::poseidon::{FieldHasher, Poseidon, PoseidonParameters};
    use arkworks_native_gadgets::to_field_elements;
    pub struct ArkworksPoseidonHasher<F: PrimeField>(PhantomData<F>);

    impl<F: PrimeField> ArkworksPoseidonHasher<F> {
        pub fn hash(input: &[u8], param_bytes: &[u8]) -> Result<Vec<u8>, Error> {
            let params = PoseidonParameters::<F>::from_bytes(param_bytes)?;
            let poseidon = Poseidon::new(params);
            let f_ins = to_field_elements(input)?;
            let output: F = poseidon.hash(&f_ins)?;
            let value = output.into_repr().to_bytes_le();
            Ok(value)
        }
    }

    use ark_bn254::Fr as Bn254;
    pub type ArkworksPoseidonHasherBn254 = ArkworksPoseidonHasher<Bn254>;
}

#[ink::contract]
pub mod poseidon {
    use crate::hasher::ArkworksPoseidonHasherBn254;
    use ark_bn254::Fr as Bn254Fr;
    use arkworks_setups::common::setup_params;
    use arkworks_setups::Curve;
    use ink_prelude::vec::Vec;
    use ink_storage::traits::SpreadAllocate;

    /// Defines the storage of your contract.
    /// Add new fields to the below struct in order
    /// to add new static storage fields to your contract.
    #[ink(storage)]
    #[derive(SpreadAllocate)]
    pub struct Poseidon {
        hasher_params_width_3_bytes: Vec<u8>,
    }

    /// The hash error types.
    #[derive(Debug, PartialEq, Eq, scale::Encode, scale::Decode)]
    #[cfg_attr(feature = "std", derive(scale_info::TypeInfo))]
    pub enum Error {
        /// Returned if there is an error hashing
        HashError,
        /// Invalid hash width
        InvalidHashInputWidth,
    }

    /// The Hash result type.
    pub type Result<T> = core::result::Result<T, Error>;

    impl Poseidon {
        /// Constructor that initializes the `bool` value to the given `init_value`.
        #[ink(constructor)]
        pub fn new() -> Self {
            Self {
                hasher_params_width_3_bytes: setup_params::<Bn254Fr>(Curve::Bn254, 5, 3).to_bytes(),
            }
        }

        #[ink(message)]
        pub fn hash(&self, inputs: Vec<[u8; 32]>) -> Result<[u8; 32]> {
            let num_inputs = inputs.len();
            let mut packed_inputs = Vec::new();
            for inp in inputs {
                packed_inputs.extend_from_slice(&inp);
            }

            let hash_result = match num_inputs {
                2 => ArkworksPoseidonHasherBn254::hash(
                    &packed_inputs,
                    &self.hasher_params_width_3_bytes,
                ),
                _ => return Err(Error::InvalidHashInputWidth),
            };

            hash_result
                .map(|h| {
                    let mut hash_result = [0u8; 32];
                    hash_result.copy_from_slice(&h);
                    hash_result
                })
                .map_err(|_| Error::HashError)
        }
    }
}
