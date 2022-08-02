#![cfg_attr(not(feature = "std"), no_std)]

use ink_lang as ink;

#[ink::contract]
mod anchor_handler {
    use ink_prelude::string::String;
    use ink_prelude::vec::Vec;
    use ink_storage::traits::{PackedLayout, SpreadLayout, StorageLayout};
    use ink_storage::{traits::SpreadAllocate, Mapping};
    use protocol_ink_lib::blake::blake2b_256_4_bytes_output;
    use protocol_ink_lib::keccak::Keccak256;
    use protocol_ink_lib::utils::{element_encoder, element_encoder_for_eight_bytes, element_encoder_for_four_bytes, element_encoder_for_one_byte, element_encoder_for_sixteen_bytes};
    use vanchor::vanchor::TokenWrapperData;
    use vanchor::VAnchorRef;

    /// The anchor handler result type.
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
        vanchor: VAnchorRef,
    }

    #[derive(Default, Debug, scale::Encode, scale::Decode, Clone, SpreadLayout, PackedLayout)]
    #[cfg_attr(feature = "std", derive(StorageLayout, scale_info::TypeInfo))]
    pub struct UpdateRecord {
        pub token_address: AccountId,
        pub src_chain_id: u64,
        pub resource_id: [u8; 32],
        pub merkle_root: [u8; 32],
        pub leaf_id: u32,
    }

    #[derive(Default, Debug, scale::Encode, scale::Decode, Clone, SpreadLayout, PackedLayout)]
    #[cfg_attr(feature = "std", derive(StorageLayout, scale_info::TypeInfo))]
    pub struct VAnchorData {
        pub max_edges: u32,
        pub chain_id: u64,
        pub levels: u32,
        pub max_deposit_amt: Balance,
        pub min_withdraw_amt: Balance,
        pub max_ext_amt: Balance,
        pub max_fee: Balance,
        pub tokenwrapper_addr: AccountId,
        pub handler: AccountId,
        pub version: u32,
        pub poseidon_contract_hash: Hash,
        pub verifier_contract_hash: Hash,
        pub token_wrapper_contract_hash: Hash,
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
        /// Configure max deposit limit error
        ConfigureMaxDepositLimitError,
        /// Configure max withdrawal limit error
        ConfigureMinWithdrawalLimitError,
        /// Update Edge error
        UpdateEdgeError,
        /// Set Handler Error
        SetHandlerError,
    }

    impl AnchorHandler {
        #[ink(constructor)]
        pub fn new(
            bridge_address: AccountId,
            initial_resource_ids: Vec<[u8; 32]>,
            initial_contract_addresses: Vec<AccountId>,
            vanchor_contract_hash: Hash,
            vanchor_data: VAnchorData,
            token_wrapper_data: TokenWrapperData,
        ) -> Self {
            let salt = vanchor_data.version.to_le_bytes();

            let vanchor = VAnchorRef::new(
                vanchor_data.max_edges,
                vanchor_data.chain_id,
                vanchor_data.levels,
                vanchor_data.max_deposit_amt,
                vanchor_data.min_withdraw_amt,
                vanchor_data.max_ext_amt,
                vanchor_data.max_fee,
                vanchor_data.tokenwrapper_addr,
                vanchor_data.handler,
                token_wrapper_data,
                vanchor_data.version,
                vanchor_data.poseidon_contract_hash,
                vanchor_data.verifier_contract_hash,
                vanchor_data.token_wrapper_contract_hash,
            )
            .endowment(0)
            .code_hash(vanchor_contract_hash)
            .salt_bytes(salt)
            .instantiate()
            .unwrap_or_else(|error| {
                panic!(
                    "failed at instantiating the Token Wrapper contract: {:?}",
                    error
                )
            });
            ink_lang::codegen::initialize_contract(|instance: &mut Self| {
                instance.vanchor = vanchor;
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
        #[ink(message, selector = 1)]
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
        #[ink(message, selector = 2)]
        pub fn execute_proposal(&mut self, resource_id: [u8; 32], data: Vec<u8>) -> Result<()> {
            // Parse the (proposal)`data`.
            let parsed_resource_id = element_encoder(&data[0..32]);

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
            if function_signature
                == blake2b_256_4_bytes_output(
                    b"VAnchor::set_handler".to_vec().as_slice(),
                )
            {
                let nonce_bytes: [u8; 8] = element_encoder_for_eight_bytes(&arguments[0..8]);
                let token_address: [u8; 32] = element_encoder(&arguments[8..40]);

                let nonce = u64::from_be_bytes(nonce_bytes);

                if self.vanchor.set_handler(token_address.into(), nonce)
                    .is_err()
                {
                    return Err(Error::SetHandlerError);
                }
            } else if function_signature
                == blake2b_256_4_bytes_output(b"VAnchor::update_edge".to_vec().as_slice())
            {
                let src_chain_id_bytes: [u8; 8] = element_encoder_for_eight_bytes(&arguments[0..8]);
                let root: [u8; 32] = element_encoder(&arguments[8..40]);
                let latest_leaf_index_bytes: [u8; 4] =
                    element_encoder_for_four_bytes(&arguments[40..44]);
                let target: [u8; 32] = element_encoder(&arguments[44..76]);

                let src_chain_id = u64::from_be_bytes(src_chain_id_bytes);
                let latest_leaf_index = u32::from_be_bytes(latest_leaf_index_bytes);

                if self.vanchor.update_edge(src_chain_id, root, latest_leaf_index, target)
                    .is_err()
                {
                    return Err(Error::UpdateEdgeError);
                }
            } else if function_signature
                == blake2b_256_4_bytes_output(
                    b"VAnchor::configure_max_deposit_limit".to_vec().as_slice(),
                )
            {
                let amount_bytes: [u8; 16] = element_encoder_for_sixteen_bytes(&arguments[0..16]);

                let amount = u128::from_be_bytes(amount_bytes);

                if self.vanchor.configure_max_deposit_limit(amount.into())
                    .is_err()
                {
                    return Err(Error::ConfigureMaxDepositLimitError);
                }
            } else if function_signature
                == blake2b_256_4_bytes_output(
                    b"VAnchor::configure_min_withdrawal_limit"
                        .to_vec()
                        .as_slice(),
                )
            {
                let amount_bytes: [u8; 16] = element_encoder_for_sixteen_bytes(&arguments[0..16]);

                let amount = u128::from_be_bytes(amount_bytes);

                if self.vanchor.configure_min_withdrawal_limit(amount.into())
                    .is_err()
                {
                    return Err(Error::ConfigureMinWithdrawalLimitError);
                }
            } else {
                return Err(Error::InvalidFunctionSignature);
            }
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

        #[ink(message)]
        pub fn get_function_signature(&self, function_type: String) -> Result<[u8; 4]> {
            let function_signature =
                blake2b_256_4_bytes_output(function_type.as_bytes().to_vec().as_slice());

            Ok(function_signature)
        }

        #[ink(message)]
        pub fn get_set_handler_function_signature(&self) -> Result<[u8; 4]> {
            let function_signature =
                blake2b_256_4_bytes_output(b"VAnchor::set_handler".to_vec().as_slice());

            Ok(function_signature)
        }

        #[ink(message)]
        pub fn get_update_edge_function_signature(&self) -> Result<[u8; 4]> {
            let function_signature =
                blake2b_256_4_bytes_output(b"VAnchor::update_edge".to_vec().as_slice());

            Ok(function_signature)
        }

        #[ink(message)]
        pub fn get_configure_max_deposit_limit_function_signature(&self) -> Result<[u8; 4]> {
            let function_signature =
                blake2b_256_4_bytes_output(b"VAnchor::configure_max_deposit_limit".to_vec().as_slice());

            Ok(function_signature)
        }

        #[ink(message)]
        pub fn get_configure_min_withdrawal_limit_function_signature(&self) -> Result<[u8; 4]> {
            let function_signature =
                blake2b_256_4_bytes_output(b"VAnchor::configure_min_withdrawal_limit".to_vec().as_slice());

            Ok(function_signature)
        }

        #[ink(message)]
        pub fn construct_data_for_set_handler(
            &self,
            resource_id: [u8; 32],
            function_signature: [u8; 4],
            nonce: [u8; 8],
            handler: AccountId,
        ) -> Result<Vec<u8>> {
            let mut result: Vec<u8> = [
                resource_id.as_slice(),
                function_signature.as_slice(),
                nonce.as_slice(),
                handler.as_ref(),
            ]
                .concat();

            Ok(result)
        }

        #[ink(message)]
        pub fn construct_data_for_update_edge(
            &self,
            resource_id: [u8; 32],
            function_signature: [u8; 4],
            src_chain_id: [u8; 8],
            root: [u8; 32],
            last_leaf_index: [u8; 4],
            target: [u8; 32],
        ) -> Result<Vec<u8>> {
            let mut result: Vec<u8> = [
                resource_id.as_slice(),
                function_signature.as_slice(),
                src_chain_id.as_slice(),
                root.as_slice(),
                last_leaf_index.as_slice(),
                target.as_slice(),
            ]
                .concat();

            Ok(result)
        }

        #[ink(message)]
        pub fn construct_data_for_limit_amount(
            &self,
            resource_id: [u8; 32],
            function_signature: [u8; 4],
            amount: [u8; 16],
        ) -> Result<Vec<u8>> {
            let mut result: Vec<u8> = [
                resource_id.as_slice(),
                function_signature.as_slice(),
                amount.as_slice(),
            ]
                .concat();

            Ok(result)
        }
    }
}
