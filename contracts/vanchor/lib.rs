#![cfg_attr(not(feature = "std"), no_std)]

//mod merkle_tree;
mod linkable_merkle_tree;
pub mod zeroes;
mod merkle_tree;

use ink_lang as ink;

#[ink::contract]
mod vanchor {
    use crate::linkable_merkle_tree::LinkableMerkleTree;
    use crate::merkle_tree::MerkleTree;

    /// The vanchor result type.
    pub type Result<T> = core::result::Result<T, Error>;


    #[ink(storage)]
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
        /// Constructor that initializes the `bool` value to the given `init_value`.
        #[ink(constructor)]
        pub fn new(chain_id: u64, creator: AccountId, merkle_tree: MerkleTree,  linkable_tree: LinkableMerkleTree, tokenwrapper_addr: AccountId,
                   max_deposit_amt: u128, min_withdraw_amt: u128, max_ext_amt: u128, max_fee: u128) -> Self {
            Self { chain_id, creator,  merkle_tree, linkable_tree, tokenwrapper_addr, max_deposit_amt, min_withdraw_amt, max_ext_amt, max_fee}
        }

        #[ink(message)]
        pub fn do_nothing_yet(&self)  {
        }

    }

}
