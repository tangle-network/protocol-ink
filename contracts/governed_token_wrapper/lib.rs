#![cfg_attr(not(feature = "std"), no_std)]
#![feature(min_specialization)]

use ink_lang as ink;


#[brush::contract]
mod governed_token_wrapper {
    use brush::contracts::psp22::extensions::metadata::*;
    use brush::contracts::traits::psp22::extensions::metadata::PSP22Metadata;
    use brush::contracts::traits::psp22::PSP22;
    use ink_prelude::string::String;
    use ink_storage::traits::{PackedLayout, SpreadLayout, StorageLayout};
    use ink_storage::{traits::SpreadAllocate, Mapping};

    #[ink(storage)]
    #[derive(Default, SpreadAllocate, PSP22Storage, PSP22MetadataStorage)]
    pub struct GovernedTokenWrapper {
        #[PSP22StorageField]
        psp22: PSP22Data,
        #[PSP22MetadataStorageField]
        metadata: PSP22MetadataData,

        // /// Governance - related params
        governor: AccountId,
        native_token_denom: String,
        fee_recipient: AccountId,
        //fee_percentage: Decimal,
        is_native_allowed: bool,
        wrapping_limit: u128,
        proposal_nonce: u64,

        tokens: Mapping<AccountId, bool>,
        historical_tokens: Mapping<AccountId, bool>,
    }


    impl PSP22 for GovernedTokenWrapper {}

    impl PSP22Metadata for GovernedTokenWrapper {}

    impl GovernedTokenWrapper {
        #[ink(constructor)]
        pub fn new(total_supply: Balance, name: Option<String>, symbol: Option<String>, decimal: u8,
        governor: AccountId, native_token_denom: String, fee_recipient: AccountId, /*fee_percentage: Decimal,*/
        is_native_allowed: bool, wrapping_limit: u128, proposal_nonce: u64) -> Self {
            ink_lang::codegen::initialize_contract(|instance: &mut Self| {
                instance.metadata.name = name;
                instance.metadata.symbol = symbol;
                instance.metadata.decimals = decimal;
                instance
                    ._mint(instance.env().caller(), total_supply)
                    .expect("Should mint total_supply");

                // Governance config
                instance.governor  = governor;
                instance.native_token_denom = native_token_denom;
                instance.fee_recipient = fee_recipient;
                //instance.fee_percentage = fee_percentage;
                instance.is_native_allowed = is_native_allowed;
                instance.wrapping_limit = wrapping_limit;
                instance.proposal_nonce = proposal_nonce;
            })
        }


        #[ink(message)]
        pub fn flip(&mut self) {
        }
    }
}
