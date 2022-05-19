#![cfg_attr(not(feature = "std"), no_std)]

mod linkable_merkle_tree;
mod merkle_tree;
pub mod zeroes;
mod keccak;
mod field_ops;

use ink_lang as ink;

#[ink::contract]
mod vanchor {
    use poseidon::poseidon::{PoseidonRef};
    use crate::linkable_merkle_tree::{Edge, LinkableMerkleTree};
    use crate::merkle_tree::MerkleTree;
    use verifier::vanchor_verifier::VAnchorVerifier;
    use crate::zeroes;
    use ink_prelude::vec::Vec;
    use ink_storage::{Mapping, traits::SpreadAllocate};
    use crate::keccak::Keccak256;
    use ink_storage::traits::{PackedLayout, SpreadLayout, StorageLayout};
    use crate::field_ops::{ArkworksIntoFieldBn254, IntoPrimeField};
    use ink_prelude::string::String;

    /// The vanchor result type.
    pub type Result<T> = core::result::Result<T, Error>;
    pub const INK_CHAIN_TYPE: [u8; 2] = [4, 0];
    pub const ERROR_MSG: &'static str =
        "requested transfer failed. this can be the case if the contract does not\
    have sufficient free funds or if the transfer would have brought the\
    contract's balance below minimum balance.";


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

        /// used nullifiers
        pub used_nullifiers: Mapping<[u8; 32], bool>,

        pub poseidon: PoseidonRef,
        pub verifier_2_2: VAnchorVerifier,
        pub verifier_16_2: VAnchorVerifier
    }

    #[derive(Default, Debug,  scale::Encode, scale::Decode, Clone, SpreadLayout, PackedLayout)]
    #[cfg_attr(feature = "std", derive(StorageLayout, scale_info::TypeInfo))]
    pub struct ExtData {
        pub recipient: AccountId,
        pub relayer: AccountId,
        pub ext_amount: String, // Still `String` since represents `i128` value
        pub fee: u128,
        pub encrypted_output1: [u8; 32],
        pub encrypted_output2: [u8; 32],
    }

    #[derive(Default, Debug,  scale::Encode, scale::Decode, Clone, SpreadLayout, PackedLayout)]
    #[cfg_attr(feature = "std", derive(StorageLayout, scale_info::TypeInfo))]
    pub struct ProofData {
        pub proof: Vec<u8>,
        pub public_amount: [u8; 32],
        pub roots: Vec<[u8; 32]>,
        pub input_nullifiers: Vec<[u8; 32]>,
        pub output_commitments: Vec<[u8; 32]>,
        pub ext_data_hash: [u8; 32],
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
        /// Unmatched edges
        UnmatchedEdges,
        /// Unknown root
        UnknownRoot,
        /// InvalidMerkleRoot
        InvalidMerkleRoots,
        /// Already revealed nullifier
        AlreadyRevealedNullfier,
        /// Invalid externalities data
        InvalidExtData,
        /// Invalid fee amount
        InvalidFeeAmount,
        /// Invalid ext amount
        InvalidExtAmount,
        /// Invalid public amount
        InvalidPublicAmount,
        /// Invalid transaction proof
        InvalidTxProof,
        /// Unauthorized
        Unauthorized,
        /// Invalid execution entry,
        InvalidExecutionEntry,
        /// Invalid deposit amount
        InvalidDepositAmount,
        /// Insufficient funds
        InsufficientFunds,


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
            version: u32,
            poseidon_contract_hash: Hash,
            verifier_contract_hash: Hash,
        ) -> Self {
            let salt = version.to_le_bytes();
            let poseidon = PoseidonRef::new()
                .endowment(0)
                .code_hash(poseidon_contract_hash)
                .salt_bytes(salt)
                .instantiate()
                .unwrap_or_else(|error| {
                    ink_env::debug_print!("contract error in poseidon init {:?}", error);
                    panic!("failed at instantiating the Poseidon contract: {:?}", error)
                });

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
        pub fn update_vanchor_config(&mut self, max_ext_amt: u128, max_fee: u128,) -> Result<()> {
            if self.creator !=  Self::env().caller() {
                return Err(Error::UnknownRoot);
            }


            self.max_ext_amt = max_ext_amt;
            self.max_fee = max_fee;

            Ok(())
        }

        #[ink(message)]
        pub fn update_edge(&mut self, src_chain_id: u64, root: [u8; 32],
                           latest_leaf_index: u32, target: [u8; 32]) -> Result<()> {
            let edge = Edge {
                chain_id: src_chain_id,
                root,
                latest_leaf_index,
                target
            };

            self.linkable_tree.update_edge(edge);

            Ok(())
        }

        #[ink(message)]
        pub fn transact_deposit(&mut self, proof_data: ProofData, ext_data: ExtData,
                                recv_token_addr: AccountId, recv_token_amt: u128) -> Result<()> {
            if self.tokenwrapper_addr != recv_token_addr {
                return Err(Error::Unauthorized);
            }

            self.validate_proof(proof_data.clone(), ext_data.clone());


            let ext_data_fee: u128 = ext_data.fee.clone();
            let ext_amt: i128 = ext_data.ext_amount.parse().expect("Invalid ext_amount");
            let abs_ext_amt = ext_amt.unsigned_abs();

            let is_withdraw = ext_amt.is_negative();

            if is_withdraw {
                return Err(Error::InvalidExecutionEntry);
            } else {
                if abs_ext_amt > self.max_deposit_amt {
                    return Err(Error::InvalidDepositAmount);
                };
                if abs_ext_amt != recv_token_amt {
                    return Err(Error::InsufficientFunds);
                };
            }

            let fee_exists = ext_data_fee != 0;

            if fee_exists {
                if self
                    .env()
                    .transfer(ext_data.relayer.clone(), ext_data_fee)
                    .is_err()
                {
                    panic!("{}", ERROR_MSG);
                }
            }

            self.execute_insertions(proof_data.clone());

            Ok(())
        }

        fn validate_proof(&mut self, proof_data: ProofData, ext_data: ExtData) -> Result<()> {
            let ext_data_fee: u128 = ext_data.fee;
            let ext_amt: i128 = ext_data.ext_amount.parse().expect("Invalid ext_amount");

            // Validation 1. Double check the number of roots.
            if self.linkable_tree.max_edges != proof_data.roots.len() as u32 {
                return Err(Error::UnmatchedEdges)
            }

            if !self
                .merkle_tree
                .is_known_root(proof_data.roots[0])
            {
                return Err(Error::UnknownRoot)
            }

            if !self.linkable_tree.is_valid_neighbor_roots(&proof_data.roots[1..]) {
                return Err(Error::InvalidMerkleRoots);
            }


            for nullifier in &proof_data.input_nullifiers {
                if self.is_known_nullifier(*nullifier) {
                    return Err(Error::AlreadyRevealedNullfier);
                }
            }

            let element_encoder = |v: &[u8]| {
                let mut output = [0u8; 32];
                output.iter_mut().zip(v).for_each(|(b1, b2)| *b1 = *b2);
                output
            };


            // Compute hash of abi encoded ext_data, reduced into field from config
            // Ensure that the passed external data hash matches the computed one
            let mut ext_data_args = Vec::new();
            let recipient_bytes = element_encoder(ext_data.recipient.as_ref());
            let relayer_bytes = element_encoder(ext_data.relayer.as_ref());
            let fee_bytes = element_encoder(&ext_data_fee.to_le_bytes());
            let ext_amt_bytes = element_encoder(&ext_amt.to_le_bytes());
            ext_data_args.extend_from_slice(&recipient_bytes);
            ext_data_args.extend_from_slice(&relayer_bytes);
            ext_data_args.extend_from_slice(&ext_amt_bytes);
            ext_data_args.extend_from_slice(&fee_bytes);
            ext_data_args.extend_from_slice(&ext_data.encrypted_output1);
            ext_data_args.extend_from_slice(&ext_data.encrypted_output2);

            let computed_ext_data_hash =
                Keccak256::hash(&ext_data_args).map_err(|_| Error::HashError)?;
            if computed_ext_data_hash != proof_data.ext_data_hash {
                return Err(Error::InvalidExtData);
            }

            let abs_ext_amt = ext_amt.unsigned_abs();
            // Making sure that public amount and fee are correct
            if ext_data_fee > self.max_fee {
                return Err(Error::InvalidFeeAmount);
            }

            if abs_ext_amt > self.max_ext_amt {
                return Err(Error::InvalidExtAmount);
            }

            // Public amounnt can also be negative, in which
            // case it would wrap around the field, so we should check if FIELD_SIZE -
            // public_amount == proof_data.public_amount, in case of a negative ext_amount
            let calc_public_amt = ext_amt - ext_data_fee as i128;
            let calc_public_amt_bytes =
                element_encoder(&ArkworksIntoFieldBn254::into_field(calc_public_amt));
            if calc_public_amt_bytes != proof_data.public_amount {
                return Err(Error::InvalidPublicAmount);
            }

            // Construct public inputs
            let chain_id_type_bytes =
                element_encoder(&self.compute_chain_id_type(self.chain_id, &INK_CHAIN_TYPE).to_le_bytes());

            let mut bytes = Vec::new();
            bytes.extend_from_slice(&proof_data.public_amount);
            bytes.extend_from_slice(&proof_data.ext_data_hash);
            for null in &proof_data.input_nullifiers {
                bytes.extend_from_slice(null);
            }
            for comm in &proof_data.output_commitments {
                bytes.extend_from_slice(comm);
            }
            bytes.extend_from_slice(&element_encoder(&chain_id_type_bytes));
            for root in &proof_data.roots {
                bytes.extend_from_slice(root);
            }

            let result = match (
                proof_data.input_nullifiers.len(),
                proof_data.output_commitments.len(),
            ) {
                (2, 2) => self.verifier_2_2.verify(bytes, proof_data.proof),
                (16, 2) => self.verifier_16_2.verify( bytes, proof_data.proof),
                _ => Ok(false),
            };

            if !result.unwrap() {
                return Err(Error::InvalidTxProof);
            }

            // Set used nullifiers to true
            for nullifier in &proof_data.input_nullifiers {
                self.used_nullifiers.insert(nullifier, &true);
            }

            Ok(())
        }

        fn execute_insertions(&mut self, proof_data: ProofData) -> Result<()> {

            for comm in &proof_data.output_commitments {
                self.merkle_tree.insert(self.poseidon.clone(), *comm);
            }

            Ok(())

        }

        fn is_known_nullifier(&self, nullifier: [u8; 32]) -> bool {
            self.used_nullifiers.get(&nullifier).is_some()
        }

        // Computes the combination bytes of "chain_type" and "chain_id".
        // Combination rule: 8 bytes array(00 * 2 bytes + [chain_type] 2 bytes + [chain_id] 4 bytes)
        // Example:
        //  chain_type - 0x0401, chain_id - 0x00000001 (big endian)
        //  Result - [00, 00, 04, 01, 00, 00, 00, 01]
        fn compute_chain_id_type(&self, chain_id: u64, chain_type: &[u8]) -> u64 {
            let chain_id_value: u32 = chain_id.try_into().unwrap_or_default();
            let mut buf = [0u8; 8];
            buf[2..4].copy_from_slice(&chain_type);
            buf[4..8].copy_from_slice(&chain_id_value.to_le_bytes());
            u64::from_be_bytes(buf)
        }
    }
}
