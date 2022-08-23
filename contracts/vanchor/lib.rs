#![cfg_attr(not(feature = "std"), no_std)]
#![feature(min_specialization)]

mod linkable_merkle_tree;
mod merkle_tree;
mod test_util;

use ink_env::call::FromAccountId;
use ink_storage::traits::SpreadAllocate;
use ink_env::Environment;
use ink_prelude::vec::Vec;


pub use self::vanchor::{VAnchor, VAnchorRef};

use ink_lang as ink;

impl SpreadAllocate for VAnchorRef {
    fn allocate_spread(_ptr: &mut ink_primitives::KeyPtr) -> Self {
        FromAccountId::from_account_id([0; 32].into())
    }
}

#[ink::chain_extension]
pub trait VerifyProof {
    type ErrorCode = VerifyProofErr;

    #[ink(extension = 1102, returns_result = false)]
    fn verify_2_2_proof(bytes: (Vec<u8>, Vec<u8>)) -> bool;

    #[ink(extension = 1103, returns_result = false)]
    fn verify_2_16_proof(bytes: (Vec<u8>, Vec<u8>)) -> bool;
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

#[ink::contract(env = crate::CustomEnvironment)]
pub mod vanchor {
    use super::VerifyProofErr;
    use crate::linkable_merkle_tree::{Edge, LinkableMerkleTree};
    use crate::merkle_tree::MerkleTree;
    use governed_token_wrapper::governed_token_wrapper::GovernedTokenWrapperRef;
    use ink_prelude::string::String;
    use ink_prelude::vec::Vec;
    use ink_storage::traits::{PackedLayout, SpreadLayout, StorageLayout};
    use ink_storage::{traits::SpreadAllocate, Mapping};
    use openbrush::contracts::psp22::extensions::metadata::*;
    use openbrush::contracts::traits::psp22::PSP22;
    use openbrush::traits::Storage;
    use poseidon::poseidon::PoseidonRef;
    use protocol_ink_lib::field_ops::{ArkworksIntoFieldBn254, IntoPrimeField};
    use protocol_ink_lib::keccak::Keccak256;
    use protocol_ink_lib::utils::{element_encoder, truncate_and_pad};
    use protocol_ink_lib::vanchor_verifier::VAnchorVerifier;
    use protocol_ink_lib::zeroes::zeroes;

    use ink_env::hash::{HashOutput, Keccak256 as inkKeccak256};


    use ark_ff::PrimeField;
    use ark_ff::BigInteger;
    use arkworks_setups::Curve;



    /// The vanchor result type.
    pub type Result<T> = core::result::Result<T, Error>;
    pub const INK_CHAIN_TYPE: [u8; 2] = [4, 0];
    pub const ERROR_MSG: &'static str =
        "requested transfer failed. this can be the case if the contract does not\
    have sufficient free funds or if the transfer would have brought the\
    contract's balance below minimum balance.";

    #[ink(storage)]
    #[derive(SpreadAllocate, Storage)]
    pub struct VAnchor {
        #[storage_field]
        psp22: psp22::Data,

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
        pub max_deposit_amt: Balance,
        /// minimum withdrawal amount
        pub min_withdraw_amt: Balance,
        /// maximum externalities amount
        pub max_ext_amt: Balance,
        /// maximum fee
        pub max_fee: Balance,
        pub handler: AccountId,
        pub proposal_nonce: u64,

        /// used nullifiers
        pub used_nullifiers: Mapping<[u8; 32], bool>,

        pub poseidon: PoseidonRef,
        pub verifier_2_2: Vec<u8>,
        pub verifier_16_2: Vec<u8>,
        pub token_wrapper: GovernedTokenWrapperRef,
    }

    impl PSP22 for VAnchor {}

    #[ink(event)]
    pub struct TransactDeposit {
        #[ink(topic)]
        from: AccountId,
        #[ink(topic)]
        to: AccountId,
    }

    #[ink(event)]
    pub struct TransactWithdraw {
        #[ink(topic)]
        input_nullifier: [u8; 32],
        #[ink(topic)]
        output_commitment: [u8; 32],
    }

    #[derive(Default, Debug, scale::Encode, scale::Decode, Clone, SpreadLayout, PackedLayout)]
    #[cfg_attr(feature = "std", derive(StorageLayout, scale_info::TypeInfo))]
    pub struct ExtData {
        pub recipient: AccountId,
        pub relayer: AccountId,
        pub ext_amount: i128, // Still `String` since represents `i128` value
        pub fee: u128,
        pub encrypted_output1: Vec<u8>,
        pub encrypted_output2: Vec<u8>,
    }

    #[derive(Default, Debug, scale::Encode, scale::Decode, Clone, SpreadLayout, PackedLayout)]
    #[cfg_attr(feature = "std", derive(StorageLayout, scale_info::TypeInfo))]
    pub struct ProofData {
        pub proof: Vec<u8>,
        pub public_amount: [u8; 32],
        pub roots: Vec<[u8; 32]>,
        pub input_nullifiers: Vec<[u8; 32]>,
        pub output_commitments: Vec<[u8; 32]>,
        pub ext_data_hash: [u8; 32],
    }

    #[derive(Default, Debug, scale::Encode, scale::Decode, Clone, SpreadLayout, PackedLayout)]
    #[cfg_attr(feature = "std", derive(StorageLayout, scale_info::TypeInfo))]
    pub struct TokenWrapperData {
        pub name: Option<String>,
        pub symbol: Option<String>,
        pub decimal: u8,
        pub governor: AccountId,
        pub fee_recipient: AccountId,
        pub fee_percentage: Balance,
        pub is_native_allowed: bool,
        pub wrapping_limit: Balance,
        pub proposal_nonce: u64,
        pub total_supply: Balance,
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
        /// Edge already exists
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
        /// Invalid deposit amount
        InvalidWithdrawAmount,
        /// Insufficient funds
        InsufficientFunds,
        /// Transfer Error
        TransferError,
        /// Wrapping Error
        WrappingError,
        /// UnWrapping Error
        UnWrappingError,
        /// Invalid Nonce
        InvalidNonce,
    }

    impl VAnchor {
        #[ink(constructor)]
        pub fn new(
            max_edges: u32,
            chain_id: u64,
            levels: u32,
            max_deposit_amt: Balance,
            min_withdraw_amt: Balance,
            max_ext_amt: Balance,
            max_fee: Balance,
            tokenwrapper_addr: AccountId,
            handler: AccountId,
            token_wrapper_data: TokenWrapperData,
            version: u32,
            poseidon_contract_hash: Hash,
            token_wrapper_contract_hash: Hash,
        ) -> Self {
            ink_env::debug_println!("Instantiating contract");
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

            let token_wrapper = GovernedTokenWrapperRef::new(
                token_wrapper_data.name,
                token_wrapper_data.symbol,
                token_wrapper_data.decimal,
                token_wrapper_data.governor,
                token_wrapper_data.fee_recipient,
                token_wrapper_data.fee_percentage,
                token_wrapper_data.is_native_allowed,
                token_wrapper_data.wrapping_limit,
                token_wrapper_data.proposal_nonce,
                token_wrapper_data.total_supply,
            )
            .endowment(0)
            .code_hash(token_wrapper_contract_hash)
            .salt_bytes(salt)
            .instantiate()
            .unwrap_or_else(|error| {
                panic!(
                    "failed at instantiating the Token Wrapper contract: {:?}",
                    error
                )
            });

            let verifier_2_2 =
                protocol_ink_lib::vanchor_verifier::VAnchorVerifier::new(max_edges, 2, 2)
                    .unwrap_or_else(|error| {
                        panic!(
                            "failed at constructing verifier(2,2) for the Vanchor contract: {:?}",
                            error
                        )
                    });

            let verifier_16_2 =
                protocol_ink_lib::vanchor_verifier::VAnchorVerifier::new(max_edges, 16, 2)
                    .unwrap_or_else(|error| {
                        panic!(
                            "failed at constructing verifier(16,2) for the Vanchor contract: {:?}",
                            error
                        )
                    });

            ink_lang::utils::initialize_contract(|contract: &mut VAnchor| {
                contract.chain_id = chain_id;
                contract.creator = Self::env().caller();
                contract.max_ext_amt = max_ext_amt;
                contract.min_withdraw_amt = min_withdraw_amt;
                contract.max_deposit_amt = max_deposit_amt;
                contract.max_fee = max_fee;
                contract.tokenwrapper_addr = tokenwrapper_addr;
                contract.handler = handler;

                contract.linkable_tree.max_edges = max_edges;
                contract.linkable_tree.chain_id_list = Vec::new();

                contract.merkle_tree.levels = levels;
                contract.merkle_tree.current_root_index = 0;
                contract.merkle_tree.next_index = 0;

                contract.poseidon = poseidon;
                contract.verifier_2_2 = verifier_2_2;
                contract.verifier_16_2 = verifier_16_2;
                contract.token_wrapper = token_wrapper;

                for i in 0..levels {
                    contract.merkle_tree.filled_subtrees.insert(i, &zeroes(i));
                }

                contract.merkle_tree.roots.insert(0, &zeroes(levels));
            })
        }

        /// Sets handler address for contract
        ///
        /// * `handler` - The contract handler to sent
        /// * `nonce` -  The nonce tracking updates to this contract
        #[ink(message, selector = 3)]
        pub fn set_handler(&mut self, handler: AccountId, nonce: u64) -> Result<()> {
            // only current handler can execute this function
            if self.handler != self.env().caller() {
                return Err(Error::Unauthorized);
            }

            if nonce <= self.proposal_nonce || self.proposal_nonce + 1048 < nonce {
                return Err(Error::InvalidNonce);
            }

            self.handler = handler;
            self.proposal_nonce = nonce;

            Ok(())
        }

        /// Returns the `governor` value.
        #[ink(message)]
        pub fn handler(&self) -> AccountId {
            self.handler
        }

        #[ink(message)]
        pub fn update_vanchor_config(&mut self, max_ext_amt: u128, max_fee: u128) -> Result<()> {
            if self.creator != Self::env().caller() {
                return Err(Error::Unauthorized);
            }

            self.max_ext_amt = max_ext_amt;
            self.max_fee = max_fee;

            Ok(())
        }

        #[ink(message)]
        pub fn configure_max_deposit_limit(&mut self, max_deposit_amt: Balance) -> Result<()> {
            if self.creator != Self::env().caller() {
                return Err(Error::Unauthorized);
            }

            self.max_deposit_amt = max_deposit_amt;

            Ok(())
        }

        #[ink(message)]
        pub fn configure_min_withdrawal_limit(
            &mut self,
            min_withdrawal_amt: Balance,
        ) -> Result<()> {
            if self.creator != Self::env().caller() {
                return Err(Error::Unauthorized);
            }

            self.min_withdraw_amt = min_withdrawal_amt;

            Ok(())
        }

        #[ink(message)]
        pub fn update_edge(
            &mut self,
            src_chain_id: u64,
            root: [u8; 32],
            latest_leaf_index: u32,
            target: [u8; 32],
        ) -> Result<()> {
            let edge = Edge {
                chain_id: src_chain_id,
                root,
                latest_leaf_index,
                target,
            };

            self.linkable_tree.update_edge(edge);

            Ok(())
        }

        #[ink(message)]
        pub fn transact_deposit(
            &mut self,
            proof_data: ProofData,
            ext_data: ExtData,
            recv_token_addr: AccountId,
            recv_token_amt: Balance,
        ) -> Result<()> {
            if self.tokenwrapper_addr != recv_token_addr {
                return Err(Error::Unauthorized);
            }

            self.validate_proof(proof_data.clone(), ext_data.clone());

            let ext_data_fee: u128 = ext_data.fee.clone();
            let ext_amt: i128 = ext_data.ext_amount.clone();
            let abs_ext_amt = ext_amt.unsigned_abs();

            let is_withdraw = ext_amt.is_negative();

            if is_withdraw {
                ink_env::debug_println!("invalid execution entry");
                return Err(Error::InvalidExecutionEntry);
            } else {
                if abs_ext_amt > self.max_deposit_amt {
                    ink_env::debug_println!("invalid deposit amount");
                    return Err(Error::InvalidDepositAmount);
                };
                if abs_ext_amt != recv_token_amt {
                    ink_env::debug_println!("insufficient funds");
                    return Err(Error::InsufficientFunds);
                };
            }

            let fee_exists = ext_data_fee != 0;

            if fee_exists {
                // PSP22 Token Transfer
                if self
                    .transfer_from(
                        self.tokenwrapper_addr,
                        ext_data.relayer.clone(),
                        ext_data_fee,
                        Vec::<u8>::new(),
                    )
                    .is_err()
                {
                    ink_env::debug_println!("transfer error");
                    return Err(Error::TransferError);
                }
            }

            self.execute_insertions(proof_data.clone());
            Ok(())
        }

        #[ink(message, payable)]
        pub fn transact_deposit_wrap_native(
            &mut self,
            proof_data: ProofData,
            ext_data: ExtData,
        ) -> Result<()> {
            let ext_data_fee: u128 = ext_data.fee.clone();
            let ext_amt: i128 = ext_data.ext_amount.clone();
            let abs_ext_amt = ext_amt.unsigned_abs();

            let amount_to_wrap = self
                .token_wrapper
                .get_amount_to_wrap(abs_ext_amt)
                .map_err(|_| Error::WrappingError)?;

            // get native token amount
            let recv_token_amt = self.env().transferred_value();

            if recv_token_amt != amount_to_wrap {
                return Err(Error::InsufficientFunds);
            };

            self.validate_proof(proof_data.clone(), ext_data.clone());

            let is_withdraw = ext_amt.is_negative();
            if is_withdraw {
                return Err(Error::InvalidExecutionEntry);
            } else {
                if abs_ext_amt > self.max_deposit_amt {
                    return Err(Error::InvalidDepositAmount);
                };

                let zero_address = self.token_wrapper.get_zero_address();

                // wrap token
                self.token_wrapper
                    .wrap_for_and_send_to(
                        zero_address,
                        self.env().caller(),
                        0,
                        self.env().account_id(),
                    )
                    .map_err(|_| Error::WrappingError)?;
            }

            let fee_exists = ext_data_fee != 0;
            if fee_exists {
                // PSP22 Token Transfer
                if self
                    .transfer_from(
                        self.tokenwrapper_addr,
                        ext_data.relayer.clone(),
                        ext_data_fee,
                        Vec::<u8>::new(),
                    )
                    .is_err()
                {
                    return Err(Error::TransferError);
                }
            }

            self.execute_insertions(proof_data.clone());
            Ok(())
        }

        #[ink(message)]
        pub fn transact_deposit_wrap_psp22(
            &mut self,
            proof_data: ProofData,
            ext_data: ExtData,
            recv_token_addr: AccountId,
            recv_token_amt: Balance,
        ) -> Result<()> {
            let ext_data_fee: u128 = ext_data.fee.clone();
            let ext_amt: i128 = ext_data.ext_amount.clone();
            let abs_ext_amt = ext_amt.unsigned_abs();

            // Only non-"TokenWrapper" Cw20 token contract can execute this message.
            if self.tokenwrapper_addr == self.env().account_id() {
                return Err(Error::Unauthorized);
            }

            let amount_to_wrap = self
                .token_wrapper
                .get_amount_to_wrap(abs_ext_amt)
                .map_err(|_| Error::WrappingError)?;

            if recv_token_amt != amount_to_wrap {
                return Err(Error::InsufficientFunds);
            };

            self.validate_proof(proof_data.clone(), ext_data.clone());

            let is_withdraw = ext_amt.is_negative();
            if is_withdraw {
                return Err(Error::InvalidExecutionEntry);
            } else {
                if abs_ext_amt > self.max_deposit_amt {
                    return Err(Error::InvalidDepositAmount);
                };

                // wrap token
                self.token_wrapper
                    .wrap_for_and_send_to(
                        self.tokenwrapper_addr,
                        self.env().caller(),
                        amount_to_wrap,
                        self.env().account_id(),
                    )
                    .map_err(|_| Error::WrappingError)?;
            }

            let fee_exists = ext_data_fee != 0;
            if fee_exists {
                // PSP22 Token Transfer
                if self
                    .transfer_from(
                        self.tokenwrapper_addr,
                        ext_data.relayer.clone(),
                        ext_data_fee,
                        Vec::<u8>::new(),
                    )
                    .is_err()
                {
                    return Err(Error::TransferError);
                }
            }
            self.execute_insertions(proof_data.clone());

            Ok(())
        }

        #[ink(message)]
        pub fn transact_withdraw(
            &mut self,
            proof_data: ProofData,
            ext_data: ExtData,
        ) -> Result<()> {
            self.validate_proof(proof_data.clone(), ext_data.clone());

            let ext_data_fee: u128 = ext_data.fee.clone();
            let ext_amt: i128 = ext_data.ext_amount.clone();
            let abs_ext_amt = ext_amt.unsigned_abs();

            if ext_amt.is_positive() {
                return Err(Error::InvalidExecutionEntry);
            } else {
                if abs_ext_amt < self.min_withdraw_amt {
                    return Err(Error::InvalidWithdrawAmount);
                };

                if self
                    .transfer_from(
                        self.tokenwrapper_addr,
                        ext_data.relayer.clone(),
                        ext_data_fee,
                        Vec::<u8>::new(),
                    )
                    .is_err()
                {
                    return Err(Error::TransferError);
                }
            }

            let fee_exists = ext_data_fee != 0;

            if fee_exists {
                // PSP22 Token Transfer
                if self
                    .transfer_from(
                        self.tokenwrapper_addr,
                        ext_data.relayer.clone(),
                        ext_data_fee,
                        Vec::<u8>::new(),
                    )
                    .is_err()
                {
                    return Err(Error::TransferError);
                }
            }

            Ok(())
        }

        #[ink(message)]
        pub fn transact_withdraw_unwrap(
            &mut self,
            proof_data: ProofData,
            ext_data: ExtData,
        ) -> Result<()> {
            self.validate_proof(proof_data.clone(), ext_data.clone());

            let ext_data_fee: u128 = ext_data.fee.clone();
            let ext_amt: i128 = ext_data.ext_amount.clone();
            let abs_ext_amt = ext_amt.unsigned_abs();

            if ext_amt.is_positive() {
                return Err(Error::InvalidExecutionEntry);
            } else {
                if abs_ext_amt < self.min_withdraw_amt {
                    return Err(Error::InvalidWithdrawAmount);
                };

                self.token_wrapper
                    .unwrap_and_send_to(self.tokenwrapper_addr, abs_ext_amt, ext_data.recipient)
                    .map_err(|_| Error::UnWrappingError)?;
            }

            let fee_exists = ext_data_fee != 0;
            if fee_exists {
                // PSP22 Token Transfer
                if self
                    .transfer_from(
                        self.tokenwrapper_addr,
                        ext_data.relayer.clone(),
                        ext_data_fee,
                        Vec::<u8>::new(),
                    )
                    .is_err()
                {
                    return Err(Error::TransferError);
                }
            }

            Ok(())
        }

        #[ink(message, payable)]
        pub fn wrap_native(&mut self) -> Result<()> {
            let zero_address = self.token_wrapper.get_zero_address();

            // wrap token
            self.token_wrapper
                .wrap(zero_address, 0)
                .map_err(|_| Error::WrappingError)
        }

        #[ink(message)]
        pub fn wrap_psp22_token(
            &mut self,
            token_address: AccountId,
            amount: Balance,
        ) -> Result<()> {
            // wrap token
            self.token_wrapper
                .wrap(token_address, amount)
                .map_err(|_| Error::WrappingError)
        }

        #[ink(message, payable)]
        pub fn unwrap_into_native_token(&mut self) -> Result<()> {
            let zero_address = self.token_wrapper.get_zero_address();

            // wrap token
            self.token_wrapper
                .unwrap_for(zero_address, 0, self.env().caller())
                .map_err(|_| Error::UnWrappingError)
        }

        #[ink(message)]
        pub fn unwrap_into_psp22_token(
            &mut self,
            token_address: AccountId,
            amount: Balance,
        ) -> Result<()> {
            // wrap token
            self.token_wrapper
                .unwrap_for(token_address, amount, self.env().caller())
                .map_err(|_| Error::UnWrappingError)
        }

        #[ink(message)]
        pub fn custom_roots_for_2(&mut self, levels: u32) -> [Vec<u8>; 2] {
            let custom_roots = Some([zeroes(levels), zeroes(levels)].map(|x| x.to_vec()));

            custom_roots.unwrap()
        }

        #[ink(message)]
        pub fn verify_proof_on_chain(&mut self, public_inputs: Vec<Vec<u8>>, proof_bytes: Vec<u8> ) -> bool {
            let message = ink_prelude::format!("public_inputs is {:?}", public_inputs);
            ink_env::debug_println!("{}",message);

            let mut inputs:Vec<u8> = Vec::new();
            for val in &public_inputs {
                inputs.append(&mut val.as_slice().to_vec());
            }

            let message = ink_prelude::format!("inputs is {:?}", inputs);
            ink_env::debug_println!("{}",message);

            let tuple: (Vec<u8>, Vec<u8>) = (inputs, proof_bytes);
            // Get the on-chain proof verification result
            let proof_result = self.env().extension().verify_2_2_proof(tuple).unwrap_or(false);

            proof_result

        }

        #[ink(message)]
        pub fn verify_proof_on_chain_2(&mut self, public_inputs: Vec<u8>, proof_bytes: Vec<u8> ) -> bool {
            let tuple: (Vec<u8>, Vec<u8>) = (public_inputs, proof_bytes);
            // Get the on-chain proof verification result
            let proof_result = self.env().extension().verify_2_2_proof(tuple).unwrap_or(false);

            proof_result

        }

        #[ink(message)]
        pub fn print_only(&mut self, sender: Vec<u8>, ext_amount: Vec<u8>, fee: Vec<u8> ) -> bool {

            let message = ink_prelude::format!("print only sender {:?}", sender);
            ink_env::debug_println!("{}",message);

            let message = ink_prelude::format!("print only ext_amount {:?}", ext_amount);
            ink_env::debug_println!("{}",message);

            let message = ink_prelude::format!("print only fee {:?}", fee);
            ink_env::debug_println!("{}",message);

            true

        }


/*#[ink(message)]
pub fn construct_data(
  &mut self,
  chain_id: u64,
  recipient: AccountId,
  relayer: AccountId,
  levels: u32,
  pk_bytes: Vec<u8>
) -> Result<(ExtData, ProofData)> {
  let ext_amount = 10_i128;
  let fee = 0_u128;

  let public_amount = 10_i128;

  let chain_type = [4, 0];
  let chain_id = &self
      .compute_chain_id_type(chain_id.clone(), &chain_type);
  let in_chain_ids = [chain_id.clone(); 2];
  let in_amounts = [0, 0];
  let in_indices = [0, 1];
  let out_chain_ids = [chain_id.clone(); 2];
  let out_amounts = [10, 0];

  let in_utxos = crate::test_util::setup_utxos_2_2_2(in_chain_ids, in_amounts, Some(in_indices));
  // We are adding indices to out utxos, since they will be used as an input utxos in next transaction
  let out_utxos =
      crate::test_util::setup_utxos_2_2_2(out_chain_ids, out_amounts, Some(in_indices));

  let output1 = out_utxos[0].commitment.into_repr().to_bytes_le();
  let output2 = out_utxos[1].commitment.into_repr().to_bytes_le();

  let ext_data = ExtData {
      recipient: recipient,
      relayer: relayer,
      ext_amount: ext_amount,
      fee: fee,
      encrypted_output1: element_encoder(&output1),
      encrypted_output2: element_encoder(&output2),
  };

  let ext_data_hash = self.hash_ext_data(ext_data.clone(), ext_amount, fee);

  let custom_roots = Some([zeroes(levels), zeroes(levels)].map(|x| x.to_vec()));
  let (proof, public_inputs) = crate::test_util::setup_zk_circuit_2_2_2(
      public_amount,
      chain_id.clone(),
      ext_data_hash.to_vec(),
      in_utxos,
      out_utxos,
      custom_roots,
      pk_bytes,
  );

  // Deconstructing public inputs
  let (_chain_id, public_amount, root_set, nullifiers, commitments, ext_data_hash) =
      crate::test_util::deconstruct_public_inputs_el_2_2_2(&public_inputs);

  // Constructing proof data
  let root_set = root_set.into_iter().map(|v| v.0).collect();
  let nullifiers = nullifiers.into_iter().map(|v| v.0).collect();
  let commitments = commitments.into_iter().map(|v| v.0).collect();
  let proof_data = ProofData {
      proof: proof,
      public_amount: public_amount.0,
      roots: root_set,
      input_nullifiers: nullifiers,
      output_commitments: commitments,
      ext_data_hash: ext_data_hash.0,
  };



  Ok((ext_data, proof_data))

}*/


fn validate_proof(&mut self, proof_data: ProofData, ext_data: ExtData) -> Result<()> {
  let ext_data_fee: u128 = ext_data.fee;
  let ext_amt: i128 = ext_data.ext_amount.clone();

  // Validation 1. Double check the number of roots.
  if self.linkable_tree.max_edges != proof_data.roots.len() as u32 {
      return Err(Error::UnmatchedEdges);
  }

  if !self.merkle_tree.is_known_root(proof_data.roots[0]) {
      return Err(Error::UnknownRoot);
  }

  if !self
      .linkable_tree
      .is_valid_neighbor_roots(&proof_data.roots[1..])
  {
      ink_env::debug_println!("invalid merkle roots");
      return Err(Error::InvalidMerkleRoots);
  } else {
      ink_env::debug_println!("valid merkle roots");
  }

  for nullifier in &proof_data.input_nullifiers {
      if self.is_known_nullifier(*nullifier) {
          ink_env::debug_println!("already revealed nullifier");
          return Err(Error::AlreadyRevealedNullfier);
      }
  }

  // Compute hash of abi encoded ext_data, reduced into field from config
  // Ensure that the passed external data hash matches the computed one
  let mut ext_data_args = Vec::new();
    let recipient_bytes = truncate_and_pad(ext_data.recipient.as_ref());
    let message = ink_prelude::format!("recipient bytes truncate is {:?}", recipient_bytes);
    ink_env::debug_println!("{}",message);
    let relayer_bytes = truncate_and_pad(ext_data.relayer.as_ref());
    let message = ink_prelude::format!("relayer bytes truncate is {:?}", recipient_bytes);
    ink_env::debug_println!("{}",message);

    let recipient_bytes = element_encoder(ext_data.recipient.as_ref());
    let message = ink_prelude::format!("recipient bytes encoder is {:?}", recipient_bytes);
    ink_env::debug_println!("{}",message);
    let relayer_bytes = element_encoder(ext_data.relayer.as_ref());
    let message = ink_prelude::format!("relayer bytes encoder is {:?}", relayer_bytes);
    ink_env::debug_println!("{}",message);

  let mut fee_bytes = element_encoder(&ext_data_fee.to_le_bytes());
    fee_bytes.reverse();
    let message = ink_prelude::format!("fee bytes encoder is {:?}", fee_bytes);
    ink_env::debug_println!("{}",message);
  let mut ext_amt_bytes = element_encoder(&ext_amt.to_le_bytes());
    ext_amt_bytes.reverse();
    let message = ink_prelude::format!("ext amt bytes encoder is {:?}", ext_amt_bytes);
    ink_env::debug_println!("{}",message);

    let message = ink_prelude::format!("encrypted_output1 is {:?}", &ext_data.encrypted_output1);
    ink_env::debug_println!("{}",message);

    let message = ink_prelude::format!("encrypted_output2 is {:?}", &ext_data.encrypted_output2);
    ink_env::debug_println!("{}",message);

   // let recipient_token = Token::Bytes(recipient_bytes.to_vec());
    //let message = ink_prelude::format!("recipient in token is {:?}", &recipient_token);
    //ink_env::debug_println!("{}",message);

  ext_data_args.extend_from_slice(&recipient_bytes);
    ext_data_args.extend_from_slice(&relayer_bytes);
    ext_data_args.extend_from_slice(&[0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 9, 24, 78, 114, 160, 0]);
  ext_data_args.extend_from_slice(&[0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0]);
  ext_data_args.extend_from_slice(&ext_data.encrypted_output1);
  ext_data_args.extend_from_slice(&ext_data.encrypted_output2);

  let computed_ext_data_hash =
      Keccak256::hash(&ext_data_args).map_err(|_| Error::HashError)?;
    let message = ink_prelude::format!("computed_ext_data_hash is {:?}", computed_ext_data_hash);
    ink_env::debug_println!("{}",message);

    let mut ink_hash = <inkKeccak256 as HashOutput>::Type::default();
    let result = ink_env::hash_bytes::<inkKeccak256>(&ext_data_args, &mut ink_hash);

    let message = ink_prelude::format!("ink hash is {:?}", ink_hash);
    ink_env::debug_println!("{}",message);

    let message = ink_prelude::format!("proof_data.ext_data_hash is {:?}", proof_data.ext_data_hash);
    ink_env::debug_println!("{}",message);

  if computed_ext_data_hash != proof_data.ext_data_hash {
      ink_env::debug_println!("invalid ext data");
     // return Err(Error::InvalidExtData);
  }

  let abs_ext_amt = ext_amt.unsigned_abs();
  // Making sure that public amount and fee are correct
  if ext_data_fee > self.max_fee {
      ink_env::debug_println!("invalid fee amount");
      return Err(Error::InvalidFeeAmount);
  }

  if abs_ext_amt > self.max_ext_amt {
      let message = ink_prelude::format!("abs_ext_amt is {:?}", abs_ext_amt);
      ink_env::debug_println!("{}",message);

      let message = ink_prelude::format!("max_ext_amt is {:?}", self.max_ext_amt);
      ink_env::debug_println!("{}",message);

      ink_env::debug_println!("invalid ext amount");
      return Err(Error::InvalidExtAmount);
  }

  // Public amounnt can also be negative, in which
  // case it would wrap around the field, so we should check if FIELD_SIZE -
  // public_amount == proof_data.public_amount, in case of a negative ext_amount
  let calc_public_amt = ext_amt - ext_data_fee as i128;
  let calc_public_amt_bytes =
      element_encoder(&ArkworksIntoFieldBn254::into_field(calc_public_amt));
  if calc_public_amt_bytes != proof_data.public_amount {
      ink_env::debug_println!("invalid public amount");
      return Err(Error::InvalidPublicAmount);
  }

  let computed_chain_id_type =   &self
        .compute_chain_id_type(self.chain_id, &INK_CHAIN_TYPE);

    let message = ink_prelude::format!("computed chain id type is {:?}", computed_chain_id_type);
    ink_env::debug_println!("{}",message);

  let computed_chain_id_type_bytes= computed_chain_id_type.to_le_bytes();

    let message = ink_prelude::format!("computed chain id type bytes is {:?}", computed_chain_id_type_bytes);
    ink_env::debug_println!("{}",message);

  // Construct public inputs
  let chain_id_type_bytes = element_encoder(
      &computed_chain_id_type_bytes
  );

    let message = ink_prelude::format!(" chain id type bytes is {:?}", chain_id_type_bytes);
    ink_env::debug_println!("{}",message);

  let mut bytes = Vec::new();
  bytes.extend_from_slice(&proof_data.public_amount);
  bytes.extend_from_slice(&proof_data.ext_data_hash);
  for null in &proof_data.input_nullifiers {
      bytes.extend_from_slice(null);
  }
  for comm in &proof_data.output_commitments {
      bytes.extend_from_slice(comm);
  }

    let message = ink_prelude::format!("chain id bytes is {:?}", chain_id_type_bytes);
    ink_env::debug_println!("{}",message);

  /*let chain_id_type_bytes = [
      56, 4, 0, 0, 0, 2, 0, 0, 0,
      0, 0, 0, 0, 0, 0, 0, 0, 0,
      0, 0, 0, 0, 0, 0, 0, 0, 0,
      0, 0, 0, 0, 0
  ];*/
  bytes.extend_from_slice(&element_encoder(&chain_id_type_bytes));
  for root in &proof_data.roots {
      bytes.extend_from_slice(root);
  }

  let result:Result<bool> = match (
      proof_data.input_nullifiers.len(),
      proof_data.output_commitments.len(),
  ) {
      (2, 2) => {
          let tuple: (Vec<u8>, Vec<u8>) = (bytes, proof_data.proof);
          // Get the on-chain proof verification result
          let proof_result = self.env().extension().verify_2_2_proof(tuple).unwrap_or(false);
          Ok(proof_result)
      }
      (16, 2) => {
          let tuple: (Vec<u8>, Vec<u8>) = (bytes, proof_data.proof);
          // Get the on-chain proof verification result
          let proof_result = self.env().extension().verify_2_16_proof(tuple).unwrap_or(false);
          Ok(proof_result)
      }
      _ => Ok(false),
  };

  if !result.unwrap() {
      ink_env::debug_println!("invalid transaction proof amount");
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
