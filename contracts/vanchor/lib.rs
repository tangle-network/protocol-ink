#![cfg_attr(not(feature = "std"), no_std)]

mod merkle_tree;
mod linkable_merkle_tree;

use ink_lang as ink;

#[ink::contract]
mod vanchor {
    use linkable_tree::LinkableMerkleTree;
    use merkle_tree::MerkleTree;



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
        pub max_deposit_amt: Uint128,
        /// minimum withdrawal amount
        pub min_withdraw_amt: Uint128,
        /// maximum externalities amount
        pub max_ext_amt: Uint128,
        /// maximum fee
        pub max_fee: Uint128,
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

    impl Vanchor {
        /// Constructor that initializes the `bool` value to the given `init_value`.
        #[ink(constructor)]
        pub fn new(init_value: bool) -> Self {
            Self { value: init_value }
        }

        /// Constructor that initializes the `bool` value to `false`.
        ///
        /// Constructors can delegate to other constructors.
        #[ink(constructor)]
        pub fn default() -> Self {
            Self::new(Default::default())
        }

        /// A message that can be called on instantiated contracts.
        /// This one flips the value of the stored `bool` from `true`
        /// to `false` and vice versa.
        #[ink(message)]
        pub fn flip(&mut self) {
            self.value = !self.value;
        }

        /// Simply returns the current value of our `bool`.
        #[ink(message)]
        pub fn get(&self) -> bool {
            self.value
        }
    }

}
