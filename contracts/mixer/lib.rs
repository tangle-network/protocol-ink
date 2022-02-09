#![cfg_attr(not(feature = "std"), no_std)]

mod zeroes;

#[cfg(test)]
mod test_util;

use ink_lang as ink;

#[ink::contract]
mod mixer {
    use crate::zeroes;
    use ink_storage::collections::HashMap;
    use poseidon::poseidon::{Poseidon, PoseidonRef};
    use verifier::mixer_verifier::{MixerVerifier, MixerVerifierRef};
    use wasm_utils::proof::truncate_and_pad;
    const ROOT_HISTORY_SIZE: u32 = 100;

    /// Defines the storage of your contract.
    /// Add new fields to the below struct in order
    /// to add new static storage fields to your contract.
    #[ink(storage)]
    pub struct Mixer {
        initialized: bool,
        deposit_size: Balance,
        levels: u32,
        current_root_index: u32,
        next_index: u32,
        filled_subtrees: HashMap<u32, [u8; 32]>,
        roots: HashMap<u32, [u8; 32]>,
        used_nullifiers: HashMap<[u8; 32], bool>,

        // Contracts
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
            levels: u32,
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
                    panic!(
                        "failed at instantiating the Poseidon contract: {:?}",
                        error
                    )
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
                levels,
                poseidon,
                verifier,
                initialized: false,
                current_root_index: 0,
                next_index: 1,
                filled_subtrees: HashMap::new(),
                roots: HashMap::new(),
                used_nullifiers: HashMap::new(),
            }
        }

        #[ink(message)]
        pub fn initialize(&mut self) -> Result<()> {
            if !self.initialized {
                return Err(Error::AlreadyInitialized);
            }

            for i in 0..self.levels {
                self.filled_subtrees[&i] = zeroes(i);
            }

            self.roots[&0] = zeroes(self.levels);
            self.initialized = true;
            Ok(())
        }

        #[ink(message)]
        pub fn deposit(&mut self, commitment: [u8; 32]) -> Result<u32> {
            if !self.initialized {
                return Err(Error::NotInitialized);
            }

            assert!(
                self.env().transferred_value() == self.deposit_size,
                "Deposit size is not correct"
            );
            self.insert(commitment)
        }

        #[ink(message)]
        pub fn withdraw(&mut self, withdraw_params: WithdrawParams) -> Result<()> {
            if !self.initialized {
                return Err(Error::NotInitialized);
            }

            assert!(
                self.is_known_root(withdraw_params.root),
                "Root is not known"
            );
            assert!(
                !self.is_known_nullifier(withdraw_params.nullifier_hash),
                "Nullifier is known"
            );
            self.used_nullifiers[&withdraw_params.nullifier_hash] = true;

            let element_encoder = |v: &[u8]| {
                let mut output = [0u8; 32];
                output.iter_mut().zip(v).for_each(|(b1, b2)| *b1 = *b2);
                output
            };

            // Format the public input bytes
            let recipient_bytes = wasm_utils::proof::truncate_and_pad(withdraw_params.recipient.as_ref());
            let relayer_bytes = wasm_utils::proof::truncate_and_pad(withdraw_params.relayer.as_ref());
            let fee_bytes = element_encoder(&withdraw_params.fee.to_be_bytes());
            let refund_bytes = element_encoder(&withdraw_params.refund.to_be_bytes());

            let mut bytes = Vec::new();
            bytes.extend_from_slice(&withdraw_params.nullifier_hash);
            bytes.extend_from_slice(&withdraw_params.root);
            bytes.extend_from_slice(&recipient_bytes);
            bytes.extend_from_slice(&relayer_bytes);
            bytes.extend_from_slice(&fee_bytes);
            bytes.extend_from_slice(&refund_bytes);

            let result = self.verify(bytes, withdraw_params.proof_bytes)?;
            assert!(result, "Invalid withdraw proof");

            // <T as pallet::Config<I>>::Currency::transfer(
            //     mixer.asset,
            //     &Self::account_id(),
            //     &recipient,
            //     mixer.deposit_size,
            // )?;
            Ok(())
        }

        pub fn verify(&self, public_input: Vec<u8>, proof_bytes: Vec<u8>) -> Result<bool> {
            self.verifier
                .verify(public_input, proof_bytes)
                .map_err(|_| Error::VerifyError)
        }

        pub fn hash_left_right(&self, left: [u8; 32], right: [u8; 32]) -> Result<[u8; 32]> {
            let inputs = vec![left, right];
            self.poseidon.hash(inputs).map_err(|_| Error::HashError)
        }

        pub fn insert(&mut self, leaf: [u8; 32]) -> Result<u32> {
            let next_index = self.next_index;
            if next_index == u32::from(2u32.pow(self.levels as u32)) {
                return Err(Error::MerkleTreeIsFull);
            }
            let mut current_index = next_index;
            let mut current_level_hash = leaf;
            let mut left: [u8; 32];
            let mut right: [u8; 32];

            for i in 0..self.levels {
                if current_index % 2 == 0 {
                    left = current_level_hash;
                    right = zeroes(i);
                    self.filled_subtrees[&i] = current_level_hash;
                } else {
                    left = self.filled_subtrees[&i];
                    right = current_level_hash;
                }

                current_level_hash = self.hash_left_right(left, right)?;
                current_index = current_index / 2;
            }

            let new_root_index = (self.current_root_index + 1) % ROOT_HISTORY_SIZE;
            self.current_root_index = new_root_index;
            self.roots[&new_root_index] = current_level_hash;
            self.next_index = next_index + 1;
            Ok(next_index)
        }

        pub fn get_root(&self) -> Result<[u8; 32]> {
            if !self.initialized {
                return Err(Error::NotInitialized);
            }

            Ok(self.roots[&self.current_root_index])
        }

        pub fn is_known_nullifier(&self, nullifier: [u8; 32]) -> bool {
            self.used_nullifiers.contains_key(&nullifier)
        }

        pub fn is_known_root(&self, root: [u8; 32]) -> bool {
            if root == [0u8; 32] {
                return false;
            }

            let mut i = self.current_root_index;
            if root == self.roots[&i] {
                return true;
            }

            if i == 0 {
                i = ROOT_HISTORY_SIZE;
            }

            i = i - 1;
            while i != self.current_root_index {
                if root == self.roots[&i] {
                    return true;
                }

                if i == 0 {
                    i = ROOT_HISTORY_SIZE;
                }

                if root == self.roots[&i] {
                    return true;
                }
            }

            false
        }
    }

    pub fn zeroes(index: u32) -> [u8; 32] {
        zeroes::zeroes(index)
    }

    /// Unit tests in Rust are normally defined within such a `#[cfg(test)]`
    /// module and test functions are marked with a `#[test]` attribute.
    /// The below code is technically just normal Rust code.
    #[cfg(test)]
    mod tests {
        /// Imports all the definitions from the outer scope so we can use them here.
        use super::*;
        use arkworks_utils::utils::common::{setup_params_x5_3, Curve};
        use crate::test_util::*;
        /// Imports `ink_lang` so we can use `#[ink::test]`.
        use ink_lang as ink;
        use ink_env::call;
        use ink_env::test;

        const SEED: u32 = 0;
        const TREE_DEPTH: usize = 30;
        const M: usize = 2;
        const DEPOSIT_SIZE: u128 = 10_000;
        const SUBSTRATE_CHAIN_TYPE: [u8; 2] = [2, 0];

        fn contract_id() -> AccountId {
            ink_env::test::get_current_contract_account_id::<ink_env::DefaultEnvironment>()
                .expect("Cannot get contract id")
        }

        fn set_sender(sender: AccountId) {
            let callee = ink_env::account_id::<ink_env::DefaultEnvironment>();
            test::push_execution_context::<Environment>(
                sender,
                callee,
                1000000,
                1000000,
                test::CallData::new(call::Selector::new([0x00; 4])), // dummy
            );
        }

        fn default_accounts(
        ) -> ink_env::test::DefaultAccounts<ink_env::DefaultEnvironment> {
            ink_env::test::default_accounts::<ink_env::DefaultEnvironment>()
                .expect("Off-chain environment should have been initialized already")
        }

        fn set_balance(account_id: AccountId, balance: Balance) {
            ink_env::test::set_account_balance::<ink_env::DefaultEnvironment>(
                account_id, balance,
            )
            .expect("Cannot set account balance");
        }

        fn get_balance(account_id: AccountId) -> Balance {
            ink_env::test::get_account_balance::<ink_env::DefaultEnvironment>(account_id)
                .expect("Cannot set account balance")
        }
        
        #[ink::test]
        fn should_deposit_and_withdraw() {
            let accounts = default_accounts();
            let alice = accounts.alice;
            let bob = accounts.bob;
            let charlie = accounts.charlie;

            set_sender(accounts.alice);
            set_balance(contract_id(), DEPOSIT_SIZE * 100);

            let levels = 30;
            let curve = Curve::Bn254;
            let (pk_bytes, vk_bytes) = setup_environment(curve);
            // deploy contracts
            let verifier = MixerVerifier::new();
            let hasher = Poseidon::new();
            let mixer = Mixer::new(
                levels,
                DEPOSIT_SIZE,
                hasher,
                verifier,
            );
            // arbitrary inputs
            let fee_value = 0;
            let refund_value = 0;
    
            // inputs
            let recipient_bytes = truncate_and_pad(bob.as_ref());
            let relayer_bytes = truncate_and_pad(charlie.as_ref());
    
            let (proof_bytes, root_element, nullifier_hash_element, leaf_element) =
                setup_wasm_utils_zk_circuit(
                    curve,
                    recipient_bytes,
                    relayer_bytes,
                    pk_bytes,
                    fee_value,
                    refund_value,
                );
    
            
            // check the balance before the withdraw.
            let balance_before = get_balance(bob);
    
            let mixer_tree_root = mixer.get_root().unwrap();
            assert_eq!(root_element.0, mixer_tree_root);
    
            // pub struct WithdrawParams {
            //     proof_bytes: Vec<u8>,
            //     root: [u8; 32],
            //     nullifier_hash: [u8; 32],
            //     recipient: AccountId,
            //     relayer: AccountId,
            //     fee: Balance,
            //     refund: Balance,
            // }

            let withdraw_params = WithdrawParams {
                proof_bytes,
                root: root_element.0,
                nullifier_hash: nullifier_hash_element.0,
                recipient: bob,
                relayer: charlie,
                fee: fee_value,
                refund: refund_value,
            };
            
            mixer.withdraw(withdraw_params).unwrap();
            // now we check the recipient balance again.
            let balance_after = get_balance(bob);
            assert_eq!(balance_after, balance_before + DEPOSIT_SIZE);
        }

        fn setup_environment(curve: Curve) -> (Vec<u8>, Vec<u8>) {
            match curve {
                Curve::Bn254 => {
                    let pk_bytes = include_bytes!(
                        "../../protocol-substrate-fixtures/mixer/bn254/x5/proving_key_uncompressed.bin"
                    );
                    let vk_bytes = include_bytes!(
                        "../../protocol-substrate-fixtures/mixer/bn254/x5/verifying_key.bin"
                    );
        
                    (pk_bytes.to_vec(), vk_bytes.to_vec())
                },
                Curve::Bls381 => {
                    unimplemented!()
                },
            }
        }
    }
}
