#![cfg_attr(not(feature = "std"), no_std)]

mod linkable_tree;

use ink_lang as ink;

#[ink::contract]
mod anchor {
    use super::*;
    use ink_prelude::vec::Vec;
    use ink_storage::collections::HashMap;
    use linkable_tree::LinkableMerkleTree;
    use mixer::{merkle_tree::MerkleTree, zeroes::zeroes};
    use poseidon::poseidon::{Poseidon, PoseidonRef};
    use verifier::anchor_verifier::{AnchorVerifier, AnchorVerifierRef};

    pub const ROOT_HISTORY_SIZE: u32 = 100;
    pub const ERROR_MSG: &'static str =
        "requested transfer failed. this can be the case if the contract does not\
    have sufficient free funds or if the transfer would have brought the\
    contract's balance below minimum balance.";

    // TODO: Anchor should have an ERC20 attached
    #[ink(storage)]
    pub struct Anchor {
        initialized: bool,
        chain_id: u64,
        deposit_size: Balance,
        merkle_tree: MerkleTree,
        linkable_tree: LinkableMerkleTree,
        used_nullifiers: HashMap<[u8; 32], bool>,
        poseidon: PoseidonRef,
        verifier: AnchorVerifierRef,
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
        /// Returned if the Anchor is not initialized
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
        roots: Vec<[u8; 32]>,
        nullifier_hash: [u8; 32],
        recipient: AccountId,
        relayer: AccountId,
        fee: Balance,
        refund: Balance,
    }

    impl Anchor {
        #[ink(constructor)]
        pub fn new(
            max_edges: u32,
            chain_id: u64,
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
                    panic!("failed at instantiating the Poseidon contract: {:?}", error)
                });
            let verifier = AnchorVerifierRef::new()
                .endowment(0)
                .code_hash(verifier_contract_hash)
                .salt_bytes(b"verifier")
                .instantiate()
                .unwrap_or_else(|error| {
                    panic!(
                        "failed at instantiating the AnchorVerifier contract: {:?}",
                        error
                    )
                });
            Self {
                chain_id,
                deposit_size,
                poseidon,
                verifier,
                initialized: false,
                linkable_tree: LinkableMerkleTree {
                    max_edges,
                    edges: HashMap::new(),
                    curr_neighbor_root_index: HashMap::new(),
                    neighbor_roots: HashMap::new(),
                },
                merkle_tree: MerkleTree {
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
                self.merkle_tree.filled_subtrees[&(i as u32)] = zeroes(i);
            }

            self.merkle_tree.roots[&0] = zeroes(self.merkle_tree.levels);
            self.initialized = true;
            Ok(())
        }

        #[ink(message)]
        pub fn deposit(&mut self, commitment: [u8; 32]) -> Result<u32> {
            assert!(self.initialized, "Anchor is not initialized");

            assert!(
                self.env().transferred_value() == self.deposit_size,
                "Deposit size is not correct"
            );

            let res = self
                .merkle_tree
                .insert(self.poseidon.clone(), commitment)
                .map_err(|_| Error::MerkleTreeIsFull)?;
            Ok(res)
        }

        #[ink(message)]
        pub fn withdraw(&mut self, withdraw_params: WithdrawParams) -> Result<()> {
            assert!(self.initialized, "Anchor is not initialized");
            assert!(
                self.merkle_tree.is_known_root(withdraw_params.roots[0]),
                "Root is not known"
            );
            assert!(
                self.linkable_tree
                    .is_valid_neighbor_roots(&withdraw_params.roots[1..]),
                "Neighbor roots are not valid"
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
            let recipient_bytes =
                mixer::mixer::truncate_and_pad(withdraw_params.recipient.as_ref());
            let relayer_bytes = mixer::mixer::truncate_and_pad(withdraw_params.relayer.as_ref());
            let fee_bytes = element_encoder(&withdraw_params.fee.to_be_bytes());
            let refund_bytes = element_encoder(&withdraw_params.refund.to_be_bytes());
            // Join the public input bytes
            let mut bytes = Vec::new();
            bytes.extend_from_slice(&withdraw_params.nullifier_hash);
            // TODO: Update with proper anchor public input encoding
            bytes.extend_from_slice(&withdraw_params.roots[0]);
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
            // TODO: Support ERC20 tokens
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
}
