#![cfg_attr(not(feature = "std"), no_std)]

mod linkable_merkle_tree;
mod merkle_tree;
pub mod zeroes;

use ink_lang as ink;

#[ink::contract]
mod vanchor {
    use poseidon::Poseidon;
    use crate::linkable_merkle_tree::{Edge, LinkableMerkleTree};
    use crate::merkle_tree::MerkleTree;
    use verifier::vanchor_verifier::VAnchorVerifier;
    use crate::zeroes;
    use ink_storage::traits::SpreadAllocate;
    use ink_prelude::vec::Vec;


    /// The vanchor result type.
    pub type Result<T> = core::result::Result<T, Error>;

    #[ink(storage)]
    #[derive(SpreadAllocate)]
    pub struct VAnchor {
        /// chain id
        pub chain_id: u64,
        /// ERC20 token address
        pub creator: AccountId,
        /// The merkle tree
        pub merkle_tree: MerkleTree,
        /// The linkable merkle tree
        pub linkable_tree: LinkableMerkleTree,
        /// ERC20 token wrapper address
        pub tokenwrapper_addr: AccountId,
        /// maximum deposit amount
        pub max_deposit_amt: u128,
        /// minimum withdrawal amount
        pub min_withdraw_amt: u128,
        /// maximum externalities amount
        pub max_ext_amt: u128,
        /// maximum fee
        pub max_fee: u128,

        pub poseidon: Poseidon,
        pub verifier_2_2: VAnchorVerifier,
        pub verifier_16_2: VAnchorVerifier
    }

    /// The vanchor error types.
    #[derive(Debug, PartialEq, Eq, scale::Encode, scale::Decode)]
    #[cfg_attr(feature = "std", derive(scale_info::TypeInfo))]
    pub enum Error {
        /// Returned if a mapping item is not found
        ItemNotFound,
        /// Returned if the merkle tree is full.
        MerkleTreeIsFull,
        /// Hash error
        HashError,
        /// Verify error
        VerifyError,
        /// Edege already exists
        EdgeAlreadyExists,
        ///  Too many edges
        TooManyEdges,
    }

    impl VAnchor {
        #[ink(constructor)]
        pub fn new(
            max_edges: u32,
            chain_id: u64,
            levels: u32,
            max_deposit_amt: u128,
            min_withdraw_amt: u128,
            max_ext_amt: u128,
            max_fee: u128,
            tokenwrapper_addr: AccountId,
            poseidon_contract_hash: Hash,
            verifier_contract_hash: Hash,
        ) -> Self {
            let poseidon = Poseidon::new();

            let verifier_2_2 = VAnchorVerifier::new(max_edges,2, 2);
            let verifier_16_2 = VAnchorVerifier::new(max_edges,16, 16);

            ink_lang::utils::initialize_contract(|contract: &mut VAnchor| {
                contract.chain_id = chain_id;
                contract.creator = Self::env().caller();
                contract.max_ext_amt = max_ext_amt;
                contract.min_withdraw_amt = min_withdraw_amt;
                contract.max_deposit_amt = max_deposit_amt;
                contract.max_fee = max_fee;
                contract.tokenwrapper_addr = tokenwrapper_addr;

                contract.linkable_tree.max_edges = max_edges;
                contract.linkable_tree.chain_id_list = Vec::new();

                contract.merkle_tree.levels = levels;
                contract.merkle_tree.current_root_index = 0;
                contract.merkle_tree.next_index = 0;

                contract.poseidon = poseidon;
                contract.verifier_2_2 = verifier_2_2;
                contract.verifier_16_2 = verifier_16_2;

                for i in 0..levels {
                    contract.merkle_tree.filled_subtrees.insert(i, &zeroes::zeroes(i));
                }

                contract.merkle_tree.roots.insert(0, &zeroes::zeroes(levels));
            })
        }

        #[ink(message)]
        pub fn update_vanchor_config(&mut self, max_ext_amt: u128, max_fee: u128,) {
            assert!(
                self.creator ==  Self::env().caller(),
                "Root is not known"
            );

            self.max_ext_amt = max_ext_amt;
            self.max_fee = max_fee;
        }

        #[ink(message)]
        pub fn update_edge(&mut self, src_chain_id: u64, root: [u8; 32],
                           latest_leaf_index: u32, target: [u8; 32]) {
            let edge = Edge {
                chain_id: src_chain_id,
                root,
                latest_leaf_index,
                target
            };

            self.linkable_tree.update_edge(edge);
        }

        fn validate_proof(&mut self) {

        }
    }
}
