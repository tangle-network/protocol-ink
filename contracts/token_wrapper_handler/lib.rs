#![cfg_attr(not(feature = "std"), no_std)]

use ink_lang as ink;

#[ink::contract]
mod token_wrapper_handler {
    use governed_token_wrapper::GovernedTokenWrapperRef;
    use ink_prelude::string::String;
    use ink_prelude::vec::Vec;
    use ink_storage::traits::{PackedLayout, SpreadLayout, StorageLayout};
    use ink_storage::{traits::SpreadAllocate, Mapping};
    use protocol_ink_lib::keccak::Keccak256;
    use protocol_ink_lib::utils::{
        element_encoder, element_encoder_for_four_bytes, element_encoder_for_one_byte,
    };

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
        contract_whitelist: Mapping<AccountId, bool>,

        pub token_wrapper: GovernedTokenWrapperRef,
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
        UnWhitelistedContractAddress,
        /// Invalid Function signature
        InvalidFunctionSignature,
    }

    // Represents the token wrapper contract instantiation configs/data
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

    impl TokenWrapperHandler {
        /// Instantiates the Token wrapper handler contract
        ///
        /// * `bridge_address` -  Contract address of previously deployed Bridge.
        /// * `initial_resource_ids` - These are the resource ids the contract will initially support
        /// * `initial_contract_addresses` - These are the the contract addresses that the contract will initially support
        /// * `version` - contract version
        /// * `token_wrapper_contract_hash` - The hash representation of the token wrapper contract
        /// * `token_wrapper_data` - token wrapper instantiation data/config
        #[ink(constructor)]
        pub fn new(
            bridge_address: AccountId,
            initial_resource_ids: Vec<[u8; 32]>,
            initial_contract_addresses: Vec<AccountId>,
            version: u32,
            token_wrapper_contract_hash: Hash,
            token_wrapper_data: TokenWrapperData,
        ) -> Self {
            let salt = version.to_le_bytes();

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

            ink_lang::codegen::initialize_contract(|instance: &mut Self| {
                instance.bridge_address = bridge_address;
                instance.token_wrapper = token_wrapper;

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
            self.contract_whitelist
                .insert(contract_address.clone(), &true);
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

        /// Sets the bridge address
        ///
        /// * `bridge_address` -  The bridge address to migrate to
        #[ink(message)]
        pub fn execute_proposal(&mut self, resource_id: [u8; 32], data: Vec<u8>) -> Result<()> {
            // Parse the (proposal)`data`.
            let parsed_resource_id = element_encoder(&data[0..32]);

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

            let is_contract_whitelisted =
                self.contract_whitelist.get(token_wrapper_address.unwrap());

            // check if contract address is whitelisted
            if is_contract_whitelisted.is_none() || !is_contract_whitelisted.unwrap() {
                return Err(Error::UnWhitelistedContractAddress);
            }

            let function_signature = element_encoder_for_four_bytes(&data[32..36]);
            let arguments = &data[36..];
            self.execute_function_signature(function_signature, arguments);

            Ok(())
        }

        /// Executes the function signature
        ///
        /// * `function_signature` -  The signature to be interpreted and executed on the token-wrapper contract
        /// * `arguments` - The function arguments to be passed to respective functions in the token-wrapper contract
        pub fn execute_function_signature(
            &mut self,
            function_signature: [u8; 4],
            arguments: &[u8],
        ) -> Result<()> {
            if function_signature
                == Keccak256::hash_with_four_bytes_output(b"setFee".to_vec().as_slice()).unwrap()
            {
                let fee_bytes: [u8; 1] = element_encoder_for_one_byte(&arguments[4..5]);

                let fee = u8::from_be_bytes(fee_bytes);

                self.token_wrapper
                    .update_config(None, None, None, Some(fee.into()), None);
            } else if function_signature
                == Keccak256::hash_with_four_bytes_output(b"addTokenAddress".to_vec().as_slice())
                    .unwrap()
            {
                let nonce_bytes: [u8; 4] = element_encoder_for_four_bytes(&arguments[0..4]);
                let token_address: [u8; 32] = element_encoder(&arguments[4..36]);

                let nonce = u32::from_be_bytes(nonce_bytes);

                self.token_wrapper
                    .add_token_address(token_address.into(), nonce.into());
            } else if function_signature
                == Keccak256::hash_with_four_bytes_output(b"removeTokenAddress".to_vec().as_slice())
                    .unwrap()
            {
                let nonce_bytes: [u8; 4] = element_encoder_for_four_bytes(&arguments[0..4]);
                let token_address: [u8; 32] = element_encoder(&arguments[4..36]);

                let nonce = u32::from_be_bytes(nonce_bytes);

                self.token_wrapper
                    .remove_token_address(token_address.into(), nonce.into());
            } else if function_signature
                == Keccak256::hash_with_four_bytes_output(b"removeTokenAddress".to_vec().as_slice())
                    .unwrap()
            {
                let fee_recipient: [u8; 32] = element_encoder(&arguments[4..36]);

                self.token_wrapper.update_config(
                    None,
                    None,
                    None,
                    None,
                    Some(fee_recipient.into()),
                );
            } else {
                return Err(Error::InvalidFunctionSignature);
            }
            Ok(())
        }
    }
}
