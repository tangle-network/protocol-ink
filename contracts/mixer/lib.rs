#![cfg_attr(not(feature = "std"), no_std)]
#![feature(min_specialization)]

pub mod merkle_tree;
pub mod zeroes;

use ink_lang as ink;

#[ink::contract]
pub mod mixer {
    use super::*;
    use crate::zeroes;
    use ink_prelude::vec::Vec;
    use ink_storage::collections::HashMap;
    use poseidon::poseidon::PoseidonRef;
    use verifier::mixer_verifier::MixerVerifierRef;

    pub const ROOT_HISTORY_SIZE: u32 = 100;
    pub const ERROR_MSG: &'static str =
        "requested transfer failed. this can be the case if the contract does not\
    have sufficient free funds or if the transfer would have brought the\
    contract's balance below minimum balance.";

    /// Defines the storage of your contract.
    /// Add new fields to the below struct in order
    /// to add new static storage fields to your contract.
    #[ink(storage)]
    pub struct Mixer {
        initialized: bool,
        deposit_size: Balance,
        merkle_tree: merkle_tree::MerkleTree,
        used_nullifiers: HashMap<[u8; 32], bool>,
        poseidon: PoseidonRef,
        verifier: MixerVerifierRef,
    }

    #[ink(event)]
    pub struct Deposit {
        #[ink(topic)]
        from: Option<AccountId>,
        #[ink(topic)]
        commitment: Option<[u8; 32]>,
        value: Balance,
    }

    /// The mixer error types.
    #[derive(Debug, PartialEq, Eq, scale::Encode, scale::Decode)]
    #[cfg_attr(feature = "std", derive(scale_info::TypeInfo))]
    pub enum Error {
        /// Returned if the mixer is not initialized
        NotInitialized,
        /// Returned if the mixer is already initialized
        AlreadyInitialized,
        /// Returned if the merkle tree is full.
        MerkleTreeIsFull,
        /// Hash error
        HashError,
        /// Verify error
        VerifyError,
    }

    /// The mixer result type.
    pub type Result<T> = core::result::Result<T, Error>;

    #[derive(Debug, PartialEq, Eq, scale::Encode, scale::Decode)]
    #[cfg_attr(feature = "std", derive(scale_info::TypeInfo))]
    pub struct WithdrawParams {
        proof_bytes: Vec<u8>,
        root: [u8; 32],
        nullifier_hash: [u8; 32],
        recipient: AccountId,
        relayer: AccountId,
        fee: Balance,
        refund: Balance,
    }

    impl Mixer {
        #[ink(constructor)]
        pub fn new(
            levels: u8,
            deposit_size: Balance,
            poseidon_contract_hash: Hash,
            verifier_contract_hash: Hash,
        ) -> Self {
            let poseidon = PoseidonRef::new()
                .endowment(0)
                .code_hash(poseidon_contract_hash)
                .salt_bytes(b"poseidon")
                .instantiate()
                .unwrap_or_else(|error| {
                    // ink_env::debug_println!(
                    //     "{}",
                    //     &format!("contract error in poseidon init{:?}", error)
                    // );
                    panic!("failed at instantiating the Poseidon contract: {:?}", error)
                });
            let verifier = MixerVerifierRef::new()
                .endowment(0)
                .code_hash(verifier_contract_hash)
                .salt_bytes(b"verifier")
                .instantiate()
                .unwrap_or_else(|error| {
                    panic!(
                        "failed at instantiating the MixerVerifier contract: {:?}",
                        error
                    )
                });
            Self {
                deposit_size,
                poseidon,
                verifier,
                initialized: false,
                merkle_tree: merkle_tree::MerkleTree {
                    levels,
                    current_root_index: 0,
                    next_index: 1,
                    filled_subtrees: HashMap::new(),
                    roots: HashMap::new(),
                },
                used_nullifiers: HashMap::new(),
            }
        }

        #[ink(message)]
        pub fn initialize(&mut self) -> Result<()> {
            assert!(!self.initialized, "Mixer already initialized");

            for i in 0..self.merkle_tree.levels {
                self.merkle_tree.filled_subtrees[&(i as u32)] = zeroes::zeroes(i);
            }

            self.merkle_tree.roots[&0] = zeroes::zeroes(self.merkle_tree.levels);
            self.initialized = true;
            Ok(())
        }

        #[ink(message)]
        pub fn deposit_size(&self) -> Result<Balance> {
            assert!(self.initialized, "Mixer not initialized");
            Ok(self.deposit_size)
        }

        #[ink(message)]
        pub fn deposit(&mut self, commitment: [u8; 32]) -> Result<u32> {
            assert!(self.initialized, "Mixer is not initialized");

            assert!(
                self.env().transferred_value() == self.deposit_size,
                "Deposit size is not correct"
            );

            self.merkle_tree.insert(self.poseidon.clone(), commitment)
        }

        #[ink(message)]
        pub fn withdraw(&mut self, withdraw_params: WithdrawParams) -> Result<()> {
            assert!(self.initialized, "Mixer is not initialized");
            assert!(
                self.merkle_tree.is_known_root(withdraw_params.root),
                "Root is not known"
            );
            assert!(
                !self.is_known_nullifier(withdraw_params.nullifier_hash),
                "Nullifier is known"
            );
            let element_encoder = |v: &[u8]| {
                let mut output = [0u8; 32];
                output.iter_mut().zip(v).for_each(|(b1, b2)| *b1 = *b2);
                output
            };
            // Format the public input bytes
            let recipient_bytes = truncate_and_pad(withdraw_params.recipient.as_ref());
            let relayer_bytes = truncate_and_pad(withdraw_params.relayer.as_ref());
            let fee_bytes = element_encoder(&withdraw_params.fee.to_be_bytes());
            let refund_bytes = element_encoder(&withdraw_params.refund.to_be_bytes());
            // Join the public input bytes
            let mut bytes = Vec::new();
            bytes.extend_from_slice(&withdraw_params.nullifier_hash);
            bytes.extend_from_slice(&withdraw_params.root);
            bytes.extend_from_slice(&recipient_bytes);
            bytes.extend_from_slice(&relayer_bytes);
            bytes.extend_from_slice(&fee_bytes);
            bytes.extend_from_slice(&refund_bytes);
            // Verify the proof
            let result = self.verify(bytes, withdraw_params.proof_bytes)?;
            assert!(result, "Invalid withdraw proof");
            // Set used nullifier to true after successfuly verification
            self.used_nullifiers[&withdraw_params.nullifier_hash] = true;
            // Send the funds
            // TODO: Support "ERC20"-like tokens
            if self
                .env()
                .transfer(
                    withdraw_params.recipient,
                    self.deposit_size - withdraw_params.fee,
                )
                .is_err()
            {
                panic!("{}", ERROR_MSG);
            }

            if self
                .env()
                .transfer(withdraw_params.relayer, withdraw_params.fee)
                .is_err()
            {
                panic!("{}", ERROR_MSG);
            }

            if withdraw_params.refund > 0 {
                if self
                    .env()
                    .transfer(withdraw_params.recipient, withdraw_params.refund)
                    .is_err()
                {
                    panic!("{}", ERROR_MSG);
                }
            }

            Ok(())
        }

        fn verify(&self, public_input: Vec<u8>, proof_bytes: Vec<u8>) -> Result<bool> {
            self.verifier
                .verify(public_input, proof_bytes)
                .map_err(|_| Error::VerifyError)
        }

        fn is_known_nullifier(&self, nullifier: [u8; 32]) -> bool {
            self.used_nullifiers.contains_key(&nullifier)
        }
    }

    pub fn truncate_and_pad(t: &[u8]) -> Vec<u8> {
        let mut truncated_bytes = t[..20].to_vec();
        truncated_bytes.extend_from_slice(&[0u8; 12]);
        truncated_bytes
    }
}
//  -- > poseidon: 0xccef3ab7b72033ca14fa6d6ef82159b998656fba6cf6da0d06f865817b96a8ac
// --> verifier: 0x9e4556c4661757959c7afdd546b81cf5546f841e9c104198f2b2f50cb1bf539f
