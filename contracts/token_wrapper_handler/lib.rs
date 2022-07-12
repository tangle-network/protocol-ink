#![cfg_attr(not(feature = "std"), no_std)]

use ink_lang as ink;

#[ink::contract]
mod token_wrapper_handler {
    use ink_env::is_contract;
    use ink_storage::{traits::SpreadAllocate, Mapping};
    use ink_prelude::vec::Vec;

    #[ink(storage)]
    #[derive(SpreadAllocate)]
    pub struct TokenWrapperHandler {
        /// Contract address of previously deployed Bridge.
        bridge_address: AccountId,
        /// resourceID => token contract address
        resource_id_to_contract_address: Mapping<[u8; 32], AccountId>,
        /// Execution contract address => resourceID
        contract_address_to_resource_id: Mapping<AccountId, [u8; 32]>,
        /// Execution contract address => is whitelisted
        contract_whitelist: Mapping<AccountId, bool>
    }

    /// The token wrapper handler result type.
    pub type Result<T> = core::result::Result<T, Error>;

    /// The token wrapper handler error types.
    #[derive(Debug, PartialEq, Eq, scale::Encode, scale::Decode)]
    #[cfg_attr(feature = "std", derive(scale_info::TypeInfo))]
    pub enum Error {
        /// Unauthorized
        Unauthorized,
        /// Invalid Resource Id
        InvalidResourceId,
        /// Contract Address Not Whitelisted
        UnWhitelistedContractAddress
    }

    impl TokenWrapperHandler {

        #[ink(constructor)]
        pub fn new(bridge_address: AccountId, initial_resource_ids: Vec<[u8; 32]>, initial_contract_addresses: Vec<AccountId>) -> Self {
            ink_lang::codegen::initialize_contract(|instance: &mut Self| {
                instance.bridge_address = bridge_address;

                if initial_resource_ids.len() != initial_contract_addresses.len() {
                    panic!(
                        "initial_resource_ids and initial_contract_addresses len mismatch"
                    );
                }
                let n = initial_resource_ids.len();

                for i in 0..initial_resource_ids.len() {
                    let resource_id = initial_resource_ids[i];
                    let contract_address = initial_contract_addresses[i];

                    instance.set_resource(resource_id, contract_address);
                }
            })
        }

        /// Sets the resource_ids and addresses
        #[ink(message)]
        pub fn set_resource(&mut self, resource_id: [u8; 32], contract_address: AccountId) {
            self.resource_id_to_contract_address.insert(resource_id, &contract_address);
            self.contract_address_to_resource_id.insert(contract_address.clone(), &resource_id);
            self.contract_whitelist.insert(contract_address.clone(), &true);
        }

        #[ink(message)]
        pub fn migrate_bridge(&mut self, bridge_address: AccountId) -> Result<()> {
            if self.env().caller() != bridge_address {
                return Err(Error::Unauthorized);
            }
            self.bridge_address = bridge_address;

            Ok(())
        }

        #[ink(message)]
        pub fn execute_proposal(&mut self, resource_id: [u8; 32],  data: Vec<u8>) -> Result<()> {
            // Parse the (proposal)`data`.
            let parsed_resource_id = element_encoder(&data[0..32]);
            let base64_encoded_proposal = &data[32..];

            if self.env().caller() != self.bridge_address {
                return Err(Error::Unauthorized);
            }

            if parsed_resource_id != resource_id {
                return Err(Error::InvalidResourceId);
            }

            let token_wrapper_address = self.resource_id_to_contract_address.get(resource_id);

            if token_wrapper_address.is_none() {
                return Err(Error::InvalidResourceId);
            }

            let is_contract_whitelisted = self.contract_whitelist.get(token_wrapper_address.unwrap());

            // check if contract address is whitelisted
            if is_contract_whitelisted.is_none() || !is_contract_whitelisted.unwrap() {
                return Err(Error::UnWhitelistedContractAddress)
            }

            Ok(())
        }

    }

    pub fn element_encoder(v: &[u8]) -> [u8; 32] {
        let mut output = [0u8; 32];
        output.iter_mut().zip(v).for_each(|(b1, b2)| *b1 = *b2);
        output
    }

}
