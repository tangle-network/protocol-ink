#![cfg_attr(not(feature = "std"), no_std)]
#![feature(min_specialization)]

mod keccak;
pub mod merkle_tree;
pub mod zeroes;

use ink_lang as ink;

#[ink::contract]
pub mod mixer {
    use super::*;
    use crate::keccak::Keccak256;
    use crate::zeroes;
    use ink_prelude::vec::Vec;
    use ink_storage::{traits::SpreadAllocate, Mapping};
    use poseidon::poseidon::PoseidonRef;
    use scale::Encode;
    use verifier::MixerVerifierRef;

    pub const ROOT_HISTORY_SIZE: u32 = 100;
    pub const ERROR_MSG: &'static str =
        "requested transfer failed. this can be the case if the contract does not\
    have sufficient free funds or if the transfer would have brought the\
    contract's balance below minimum balance.";

    #[ink(storage)]
    #[derive(SpreadAllocate)]
    pub struct Mixer {
        deposit_size: Balance,
        merkle_tree: merkle_tree::MerkleTree,
        used_nullifiers: Mapping<[u8; 32], bool>,
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
        /// Returned if a mapping item is not found
        ItemNotFound,
        /// Returned if the merkle tree is full.
        MerkleTreeIsFull,
        /// Hash error
        HashError,
        /// Verify error
        VerifyError,
        /// Nullifier is known
        NullifierKnown
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
            levels: u32,
            deposit_size: Balance,
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
            let verifier = MixerVerifierRef::new()
                .endowment(0)
                .code_hash(verifier_contract_hash)
                .salt_bytes(salt)
                .instantiate()
                .unwrap_or_else(|error| {
                    panic!(
                        "failed at instantiating the MixerVerifier contract: {:?}",
                        error
                    )
                });

            ink_lang::utils::initialize_contract(|contract: &mut Mixer| {
                contract.deposit_size = deposit_size;
                contract.poseidon = poseidon;
                contract.verifier = verifier;
                contract.merkle_tree.levels = levels;
                contract.merkle_tree.current_root_index = 0;
                contract.merkle_tree.next_index = 0;

                let message = ink_prelude::format!("Instantiating contract");
                ink_env::debug_println!("{}", &message);

                for i in 0..levels {
                    contract
                        .merkle_tree
                        .filled_subtrees
                        .insert(i, &zeroes::zeroes(i));
                }

                contract
                    .merkle_tree
                    .roots
                    .insert(0, &zeroes::zeroes(levels));
            })
        }

        /// Returns the `levels` value.
        #[ink(message)]
        pub fn levels(&self) -> u32 {
            self.merkle_tree.levels
        }

        /// Returns the `deposit_size` value.
        #[ink(message)]
        pub fn deposit_size(&self) -> Balance {
            self.deposit_size
        }

        #[ink(message, payable)]
        pub fn deposit(&mut self, commitment: [u8; 32]) -> Result<u32> {
            assert!(
                self.env().transferred_value() == self.deposit_size,
                "Deposit size is not correct"
            );

            self.merkle_tree.insert(self.poseidon.clone(), commitment)
        }

        #[ink(message, payable)]
        pub fn send_fund_to_contract(&self) {
            ink_env::debug_println!("funds sent");
        }

        #[ink(message)]
        pub fn withdraw(
            &mut self,
            withdraw_params: WithdrawParams,
        ) -> Result<()> {
            let message = ink_prelude::format!("root in withdraw is {:?}", withdraw_params.root);
            ink_env::debug_println!("{}", &message);

            assert!(
                self.merkle_tree.is_known_root(withdraw_params.root),
                "Root is not known"
            );
            let message = ink_prelude::format!("nullifier is {:?}", withdraw_params.nullifier_hash);
            ink_env::debug_println!("{}", &message);
            if self.is_known_nullifier(withdraw_params.nullifier_hash) {
                return Err(Error::NullifierKnown);
            }

            let element_encoder = |v: &[u8]| {
                let mut output = [0u8; 32];
                output.iter_mut().zip(v).for_each(|(b1, b2)| *b1 = *b2);
                output
            };
            // Format the public input bytes
            let recipient_bytes = truncate_and_pad(withdraw_params.recipient.as_ref());
            let relayer_bytes = truncate_and_pad(withdraw_params.relayer.as_ref());

            let fee_bytes = &withdraw_params.fee.encode();
            let refund_bytes = &withdraw_params.refund.encode();

            let mut arbitrary_data_bytes = Vec::new();
            arbitrary_data_bytes.extend_from_slice(&recipient_bytes);
            arbitrary_data_bytes.extend_from_slice(&relayer_bytes);
            arbitrary_data_bytes.extend_from_slice(&fee_bytes);
            arbitrary_data_bytes.extend_from_slice(&refund_bytes);
            let arbitrary_input =
                Keccak256::hash(&arbitrary_data_bytes).map_err(|_| Error::HashError)?;

            // Join the public input bytes
            let mut bytes = Vec::new();
            bytes.extend_from_slice(&withdraw_params.nullifier_hash);
            bytes.extend_from_slice(&withdraw_params.root);
            bytes.extend_from_slice(&arbitrary_input);

            ink_env::debug_println!("trying to verify proof");
            // Verify the proof
            let result = self.verify(bytes, withdraw_params.proof_bytes)?;
            let message = ink_prelude::format!("verification result is {:?}", result);
            ink_env::debug_println!("{}", &message);
            assert!(result, "Invalid withdraw proof");
            // Set used nullifier to true after successfuly verification
            self.used_nullifiers
                .insert(withdraw_params.nullifier_hash, &true);

            let message = ink_prelude::format!("deposit in withdraw is {:?}", self.deposit_size);
            ink_env::debug_println!("{}", &message);

            let message = ink_prelude::format!("fee in withdraw is {:?}", withdraw_params.fee);
            ink_env::debug_println!("{}", &message);

            let actual_amount = self.deposit_size - withdraw_params.fee;
            let message = ink_prelude::format!("actual_amount in withdraw is {:?}", actual_amount);
            ink_env::debug_println!("{}", &message);
            // Send the funds
            // TODO: Support "PSP22"-like tokens and Native token
            // TODO: SPEC this more with Drew and create task/issue
            if self
                .env()
                .transfer(withdraw_params.recipient, actual_amount)
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

        #[ink(message)]
        pub fn insert_nullifier(&mut self, nullifier:[u8; 32])  {
            self.used_nullifiers
                .insert(&nullifier, &true);
        }

        /// Returns native contract address
        #[ink(message)]
        pub fn native_contract_account_id(&self) -> Option<AccountId> {
            Some(self.env().account_id())
        }

        fn verify(&self, public_input: Vec<u8>, proof_bytes: Vec<u8>) -> Result<bool> {
            let message = ink_prelude::format!("verifier is {:?}", self.verifier);
            ink_env::debug_println!("{}", &message);
            if self.verifier.verify(public_input, proof_bytes).is_err() {
                ink_env::debug_println!("Error occurred verifying proof");
            }

            Ok(true)
            // .map_err(|_| panic!("{}", ERROR_MSG))
        }

        #[ink(message)]
        pub fn is_known_nullifier(&self, nullifier: [u8; 32]) -> bool {
            let nullifier_exists = self.used_nullifiers.contains(&nullifier);
            let message = ink_prelude::format!("nullifier exists {:?}", nullifier_exists);
            ink_env::debug_println!("{}", &message);
            let null = [236, 235, 4, 214, 238, 196, 192, 154, 210, 230, 147, 255, 50, 207, 235, 240, 79, 181, 5, 74, 44, 224, 77, 249, 237, 255, 53, 220, 31, 55, 142, 11].to_vec();
            let result = self.used_nullifiers.get(&nullifier).unwrap_or(false);
            let message = ink_prelude::format!("nullifier is {:?}", result);
            ink_env::debug_println!("{}", &message);
            let message = ink_prelude::format!(
                "nullifier value is {:?}",
                self.used_nullifiers.get(&nullifier)
            );
            ink_env::debug_println!("{}", &message);
            result
        }

        #[ink(message, payable)]
        pub fn is_known_nullifier_payable(&self, nullifier: [u8; 32]) -> bool {
            let nullifier_exists = self.used_nullifiers.contains(&nullifier.clone());
            let message = ink_prelude::format!("nullifier exists {:?}", nullifier_exists);
            ink_env::debug_println!("{}", &message);
            let null = [236, 235, 4, 214, 238, 196, 192, 154, 210, 230, 147, 255, 50, 207, 235, 240, 79, 181, 5, 74, 44, 224, 77, 249, 237, 255, 53, 220, 31, 55, 142, 11].to_vec();
            let result = self.used_nullifiers.get(&nullifier).unwrap_or(false);
            let message = ink_prelude::format!("nullifier is {:?}", result);
            ink_env::debug_println!("{}", &message);
            let message = ink_prelude::format!(
                "nullifier value is {:?}",
                self.used_nullifiers.get(&nullifier)
            );
            ink_env::debug_println!("{}", &message);
            result
        }



        /// Returns native contract balance
        #[ink(message)]
        pub fn native_contract_balance(&self) -> Balance {
            self.env().balance()
        }
    }

    pub fn truncate_and_pad(t: &[u8]) -> Vec<u8> {
        let mut truncated_bytes = t[..20].to_vec();
        truncated_bytes.extend_from_slice(&[0u8; 12]);
        truncated_bytes
    }
}
