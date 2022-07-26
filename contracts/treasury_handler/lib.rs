#![cfg_attr(not(feature = "std"), no_std)]

use ink_lang as ink;

#[ink::contract]
mod treasury_handler {
    use ink_env::hash::Blake2x256;
    use ink_prelude::string::String;
    use ink_prelude::vec::Vec;
    use ink_storage::traits::{PackedLayout, SpreadLayout, StorageLayout};
    use ink_storage::{traits::SpreadAllocate, Mapping};
    use protocol_ink_lib::blake::blake2b_256_4_bytes_output;
    use protocol_ink_lib::keccak::Keccak256;
    use protocol_ink_lib::utils::{
        element_encoder, element_encoder_for_eight_bytes, element_encoder_for_four_bytes,
        element_encoder_for_one_byte,
    };
    use treasury::TreasuryRef;

    /// The treasury wrapper handler result type.
    pub type Result<T> = core::result::Result<T, Error>;

    #[ink(storage)]
    #[derive(SpreadAllocate)]
    pub struct TreasuryHandler {
        /// Contract address of previously deployed Bridge.
        bridge_address: AccountId,
        /// resourceID => token contract address
        resource_id_to_contract_address: Mapping<[u8; 32], AccountId>,
        /// Execution contract address => resourceID
        contract_address_to_resource_id: Mapping<AccountId, [u8; 32]>,
        /// Execution contract address => is whitelisted
        contract_whitelist: Mapping<AccountId, bool>,
        /// (src_chain_id, nonce) -> UpdateRecord
        update_records: Mapping<(u64, u64), UpdateRecord>,
        treasury: TreasuryRef,
    }

    #[derive(Default, Debug, scale::Encode, scale::Decode, Clone, SpreadLayout, PackedLayout)]
    #[cfg_attr(feature = "std", derive(StorageLayout, scale_info::TypeInfo))]
    pub struct UpdateRecord {
        pub treasury_address: AccountId,
        pub execution_chain_id: u64,
        pub nonce: u64,
        pub resource_id: [u8; 32],
        pub update_value: [u8; 32],
    }

    /// The token wrapper handler error types.
    #[derive(Debug, PartialEq, Eq, scale::Encode, scale::Decode)]
    #[cfg_attr(feature = "std", derive(scale_info::TypeInfo))]
    pub enum Error {
        /// Unauthorized
        Unauthorized,
        /// Invalid Resource Id
        InvalidResourceId,
        /// Contract Address Not Whitelisted
        UnWhitelistedContractAddress,
        /// Invalid Function signature
        InvalidFunctionSignature,
        /// No Update Record found
        UpdateRecordNotFound,
        /// Invalid Contract Address
        InvalidContractAddress,
    }

    impl TreasuryHandler {
        /// Instantiates the Treasury handler contract
        ///
        /// * `bridge_address` -  Contract address of previously deployed Bridge.
        /// * `initial_resource_ids` - These are the resource ids the contract will initially support
        /// * `initial_contract_addresses` - These are the the contract addresses that the contract will initially support
        /// * `version` - contract version
        #[ink(constructor)]
        pub fn new(
            bridge_address: AccountId,
            initial_resource_ids: Vec<[u8; 32]>,
            initial_contract_addresses: Vec<AccountId>,
            version: u32,
            treasury_contract_hash: Hash,
        ) -> Self {
            ink_lang::codegen::initialize_contract(|instance: &mut Self| {
                let salt = version.to_le_bytes();
                let treasury = TreasuryRef::new(instance.env().account_id())
                    .endowment(0)
                    .code_hash(treasury_contract_hash)
                    .salt_bytes(salt)
                    .instantiate()
                    .unwrap_or_else(|error| {
                        panic!("failed at instantiating the Treasury contract: {:?}", error)
                    });

                instance.bridge_address = bridge_address;
                instance.treasury = treasury;

                if initial_resource_ids.len() != initial_contract_addresses.len() {
                    panic!("initial_resource_ids and initial_contract_addresses len mismatch");
                }

                for i in 0..initial_resource_ids.len() {
                    let resource_id =
                        ink_prelude::format!("resource_id is {:?}", initial_resource_ids[i]);
                    ink_env::debug_println!("{}", &resource_id);

                    let contract_address = ink_prelude::format!(
                        "contract_address is {:?}",
                        initial_contract_addresses[i]
                    );
                    ink_env::debug_println!("{}", &contract_address);

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
        pub fn set_resource(
            &mut self,
            resource_id: [u8; 32],
            contract_address: AccountId,
        ) -> Result<()> {
            self.resource_id_to_contract_address
                .insert(resource_id, &contract_address);
            self.contract_address_to_resource_id
                .insert(contract_address.clone(), &resource_id);
            self.contract_whitelist
                .insert(contract_address.clone(), &true);

            Ok(())
        }

        /// Sets the bridge address
        ///
        /// * `bridge_address` -  The bridge address to migrate to
        #[ink(message)]
        pub fn migrate_bridge(&mut self, bridge_address: AccountId) -> Result<()> {
            if self.env().caller() != self.bridge_address {
                return Err(Error::Unauthorized);
            }
            self.bridge_address = bridge_address;

            Ok(())
        }

        /// Executes proposal
        ///
        /// * `resource_id` -  The resource id
        /// * `data` - The data to execute
        #[ink(message, payable)]
        pub fn execute_proposal(&mut self, resource_id: [u8; 32], data: Vec<u8>) -> Result<()> {
            let message = ink_prelude::format!(" data is {:?}", data);
            ink_env::debug_println!("{}", &message);

            let message = ink_prelude::format!(" resource id is {:?}", resource_id);
            ink_env::debug_println!("{}", &message);

            // Parse the (proposal)`data`.
            let parsed_resource_id = element_encoder(&data[0..32]);
            let message = ink_prelude::format!(" parsed_resource_id is {:?}", parsed_resource_id);
            ink_env::debug_println!("{}", &message);

            if self.env().caller() != self.bridge_address {
                return Err(Error::Unauthorized);
            }

            if parsed_resource_id != resource_id {
                return Err(Error::InvalidResourceId);
            }

            let anchor_address = self.resource_id_to_contract_address.get(resource_id);

            if anchor_address.is_none() {
                return Err(Error::InvalidResourceId);
            }

            let is_contract_whitelisted = self.contract_whitelist.get(anchor_address.unwrap());

            // check if contract address is whitelisted
            if is_contract_whitelisted.is_none() || !is_contract_whitelisted.unwrap() {
                return Err(Error::UnWhitelistedContractAddress);
            }

            // extract function signature
            let function_signature = element_encoder_for_four_bytes(&data[32..36]);
            let arguments = &data[36..];
            self.execute_function_signature(function_signature, arguments);

            Ok(())
        }

        /// Executes the function signature
        ///
        /// * `function_signature` -  The signature to be interpreted and executed on the vanchor contract
        /// * `arguments` - The function arguments to be passed to respective functions in the vanchor contract
        pub fn execute_function_signature(
            &mut self,
            function_signature: [u8; 4],
            arguments: &[u8],
        ) -> Result<()> {
            let message = ink_prelude::format!(" contract caller is {:?}", self.env().caller());
            ink_env::debug_println!("{}", &message);

            let message = ink_prelude::format!("contract address is {:?}", self.env().account_id());
            ink_env::debug_println!("{}", &message);

            if function_signature
                == blake2b_256_4_bytes_output(b"Treasury::set_handler".to_vec().as_slice())
            {
                let nonce_bytes: [u8; 4] = element_encoder_for_four_bytes(&arguments[0..4]);
                let token_address: [u8; 32] = element_encoder(&arguments[4..36]);

                let message = ink_prelude::format!(" token_address  is {:?}", token_address);
                ink_env::debug_println!("{}", &message);

                let nonce = u32::from_be_bytes(nonce_bytes);

                let message = ink_prelude::format!(" nonce  is {:?}", nonce);
                ink_env::debug_println!("{}", &message);

                self.treasury.set_handler(token_address.into(), nonce);
            } else if function_signature
                == blake2b_256_4_bytes_output(b"Treasury::rescue_tokens".to_vec().as_slice())
            {
                ink_env::debug_println!("trying to rescue tokens");
                let nonce_bytes: [u8; 4] = element_encoder_for_four_bytes(&arguments[0..4]);
                let token_address: [u8; 32] = element_encoder(&arguments[4..36]);
                let to: [u8; 32] = element_encoder(&arguments[36..68]);
                let amount_to_rescue_bytes = element_encoder_for_four_bytes(&arguments[68..72]);

                let nonce = u32::from_be_bytes(nonce_bytes);
                let amount_to_rescue = u32::from_be_bytes(amount_to_rescue_bytes);

                self.treasury.rescue_tokens(
                    token_address.into(),
                    to.into(),
                    amount_to_rescue.into(),
                    nonce,
                );
            }
            Ok(())
        }

        /// Gets update record
        ///
        /// * `src_chain_id` -  The src_chain_id to query
        /// * `nonce` - nonce
        #[ink(message)]
        pub fn read_update_record(&self, src_chain_id: u64, nonce: u64) -> Result<UpdateRecord> {
            if self.update_records.get((src_chain_id, nonce)).is_none() {
                return Err(Error::UpdateRecordNotFound);
            }

            Ok(self.update_records.get((src_chain_id, nonce)).unwrap())
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

        #[ink(message)]
        pub fn construct_data_for_set_handler(
            &self,
            resource_id: [u8; 32],
            function_signature: [u8; 4],
            nonce: [u8; 4],
            address: AccountId,
        ) -> Result<Vec<u8>> {
            let mut result: Vec<u8> = [
                resource_id.as_slice(),
                function_signature.as_slice(),
                nonce.as_slice(),
                address.as_ref(),
            ]
            .concat();
            let message = ink_prelude::format!("result is {:?}", result);
            ink_env::debug_println!("{}", &message);

            let func_sig =
                blake2b_256_4_bytes_output(b"Treasury::rescue_tokens".to_vec().as_slice());
            let message = ink_prelude::format!("rescue token function sig is {:?}", func_sig);
            ink_env::debug_println!("{}", &message);

            let amount_bytes: [u8; 4] = transform_u32_to_array_of_u8(100);
            let message = ink_prelude::format!("amount is {:?}", amount_bytes);
            ink_env::debug_println!("{}", &message);

            let nonce_bytes: [u8; 4] = transform_u32_to_array_of_u8(2096);
            let message = ink_prelude::format!("nonce is {:?}", nonce_bytes);
            ink_env::debug_println!("{}", &message);

            Ok(result)
        }

        #[ink(message)]
        pub fn construct_data_for_rescue_tokens(
            &self,
            resource_id: [u8; 32],
            function_signature: [u8; 4],
            nonce: [u8; 4],
            token_address: AccountId,
            to: AccountId,
            amount_to_rescue: [u8; 4],
        ) -> Result<Vec<u8>> {
            let mut result: Vec<u8> = [
                resource_id.as_slice(),
                function_signature.as_slice(),
                nonce.as_slice(),
                token_address.as_ref(),
                to.as_ref(),
                amount_to_rescue.as_slice(),
            ]
            .concat();
            let message = ink_prelude::format!("result is {:?}", result);
            ink_env::debug_println!("{}", &message);
            Ok(result)
        }
    }

    pub fn transform_u32_to_array_of_u8(x: u32) -> [u8; 4] {
        let b1: u8 = ((x >> 24) & 0xff) as u8;
        let b2: u8 = ((x >> 16) & 0xff) as u8;
        let b3: u8 = ((x >> 8) & 0xff) as u8;
        let b4: u8 = (x & 0xff) as u8;
        return [b1, b2, b3, b4];
    }
}
