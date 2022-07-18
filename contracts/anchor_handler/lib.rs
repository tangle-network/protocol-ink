#![cfg_attr(not(feature = "std"), no_std)]

use ink_lang as ink;

#[ink::contract]
mod anchor_handler {
    use ink_prelude::string::String;
    use ink_prelude::vec::Vec;
    use ink_storage::traits::{PackedLayout, SpreadLayout, StorageLayout};
    use ink_storage::{traits::SpreadAllocate, Mapping};
    use protocol_ink_lib::keccak::Keccak256;
    use protocol_ink_lib::utils::{element_encoder, element_encoder_for_four_bytes};
    /// The token wrapper handler result type.
    pub type Result<T> = core::result::Result<T, Error>;

    #[ink(storage)]
    #[derive(SpreadAllocate)]
    pub struct AnchorHandler {
        /// Contract address of previously deployed Bridge.
        bridge_address: AccountId,
        /// resourceID => token contract address
        resource_id_to_contract_address: Mapping<[u8; 32], AccountId>,
        /// Execution contract address => resourceID
        contract_address_to_resource_id: Mapping<AccountId, [u8; 32]>,
        /// Execution contract address => is whitelisted
        contract_whitelist: Mapping<AccountId, bool>,
        /// (src_chain_id, height) -> UpdateRecord
        update_records: Mapping<(u64, u64), UpdateRecord>,
    }

    #[derive(Default, Debug, scale::Encode, scale::Decode, Clone, SpreadLayout, PackedLayout)]
    #[cfg_attr(feature = "std", derive(StorageLayout, scale_info::TypeInfo))]
    pub struct UpdateRecord {
        pub token_address: AccountId,
        pub src_chain_id: u64,
        pub resource_id: [u8; 32],
        pub merkle_root: [u8; 32],
        pub leaf_id: u64,
    }

    /// The token wrapper handler error types.
    #[derive(Debug, PartialEq, Eq, scale::Encode, scale::Decode)]
    #[cfg_attr(feature = "std", derive(scale_info::TypeInfo))]
    pub enum Error {
        /// Unauthorized
        Unauthorized,
        /// Invalid Resource Id
        InvalidResourceId,
        /// Invalid Contract Address
        InvalidContractAddress,
        /// Contract Address Not Whitelisted
        UnWhitelistedContractAddress,
        /// Invalid Function signature
        InvalidFunctionSignature,
        /// No Update Record found
        UpdateRecordNotFound,
    }

    impl AnchorHandler {
        #[ink(constructor)]
        pub fn new(
            bridge_address: AccountId,
            initial_resource_ids: Vec<[u8; 32]>,
            initial_contract_addresses: Vec<AccountId>,
        ) -> Self {
            ink_lang::codegen::initialize_contract(|instance: &mut Self| {
                instance.bridge_address = bridge_address;
                if initial_resource_ids.len() != initial_contract_addresses.len() {
                    panic!("initial_resource_ids and initial_contract_addresses len mismatch");
                }

                for i in 0..initial_resource_ids.len() {
                    let resource_id = initial_resource_ids[i];
                    let contract_address = initial_contract_addresses[i];

                    instance.set_resource(resource_id, contract_address);
                }
            })
        }

        /// Sets the resource_ids and addresses
        ///
        /// * `resource_id` -  The resource id to be mapped to.
        /// * `contract_address` -  The contract address to be mapped to
        #[ink(message)]
        pub fn set_resource(&mut self, resource_id: [u8; 32], contract_address: AccountId) {
            self.resource_id_to_contract_address
                .insert(resource_id, &contract_address);
            self.contract_address_to_resource_id
                .insert(contract_address.clone(), &resource_id);
        }

        /// Sets the bridge address
        ///
        /// * `bridge_address` -  The bridge address to migrate to
        #[ink(message)]
        pub fn migrate_bridge(&mut self, bridge_address: AccountId) -> Result<()> {
            if self.env().caller() != bridge_address {
                return Err(Error::Unauthorized);
            }
            self.bridge_address = bridge_address;

            Ok(())
        }

        /// Gets update record
        ///
        /// * `src_chain_id` -  The src_chain_id to query
        /// * `height` - The leaf height to query
        #[ink(message)]
        pub fn read_update_record(&self, src_chain_id: u64, height: u64) -> Result<UpdateRecord> {
            if self.update_records.get((src_chain_id, height)).is_none() {
                return Err(Error::UpdateRecordNotFound);
            }

            Ok(self.update_records.get((src_chain_id, height)).unwrap())
        }

        /// Gets bridge address
        #[ink(message)]
        pub fn get_bridge_address(&self) -> Result<AccountId> {
            Ok(self.bridge_address)
        }

        /// Queries contract address
        ///
        /// * `resource_id` -  The resource_id to query
        #[ink(message)]
        pub fn get_contract_address(&self, resource_id: [u8; 32]) -> Result<AccountId> {
            if self
                .resource_id_to_contract_address
                .get(resource_id)
                .is_none()
            {
                return Err(Error::InvalidResourceId);
            }

            Ok(self
                .resource_id_to_contract_address
                .get(resource_id)
                .unwrap())
        }

        /// Queries resource id
        ///
        /// * `address` -  The contract address to query
        #[ink(message)]
        pub fn get_resource_id(&self, address: AccountId) -> Result<[u8; 32]> {
            if self.contract_address_to_resource_id.get(address).is_none() {
                return Err(Error::InvalidContractAddress);
            }

            Ok(self.contract_address_to_resource_id.get(address).unwrap())
        }

        /// Queries if contract address is whitelisted
        ///
        /// * `address` -  The contract address to query
        #[ink(message)]
        pub fn is_contract_address_whitelisted(&self, address: AccountId) -> Result<bool> {
            if self.contract_whitelist.get(address).is_none() {
                return Err(Error::UnWhitelistedContractAddress);
            }

            Ok(self.contract_whitelist.get(address).unwrap())
        }
    }
}
