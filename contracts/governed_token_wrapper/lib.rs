#![cfg_attr(not(feature = "std"), no_std)]
#![feature(min_specialization)]

use ink_lang as ink;


#[brush::contract]
mod governed_token_wrapper {
    use brush::contracts::psp22::extensions::metadata::*;
    use brush::contracts::psp22::extensions::mintable::*;
    use brush::contracts::psp22::extensions::wrapper::*;
    use brush::contracts::traits::psp22::PSP22;
    use brush::contracts::psp22::*;
    use brush::test_utils::*;
    use ink_prelude::string::String;
    use ink_storage::traits::{PackedLayout, SpreadLayout, StorageLayout};
    use ink_storage::{traits::SpreadAllocate, Mapping};
    use ink_prelude::vec::Vec;

    /// The vanchor result type.
    pub type Result<T> = core::result::Result<T, Error>;
    pub const ERROR_MSG: &'static str =
        "requested transfer failed. this can be the case if the contract does not\
    have sufficient free funds or if the transfer would have brought the\
    contract's balance below minimum balance.";

    #[ink(storage)]
    #[derive(Default, SpreadAllocate, PSP22Storage, PSP22WrapperStorage, PSP22MetadataStorage)]
    pub struct GovernedTokenWrapper {
        #[PSP22StorageField]
        psp22: PSP22Data,
        #[PSP22MetadataStorageField]
        metadata: PSP22MetadataData,
        #[PSP22WrapperStorageField]
        wrapper: PSP22WrapperData,

        // /// Governance - related params
        governor: AccountId,
        native_token_denom: String,
        fee_recipient: AccountId,
        fee_percentage: Balance,
        is_native_allowed: bool,
        wrapping_limit: u128,
        proposal_nonce: u64,

        tokens: Mapping<AccountId, bool>,
        historical_tokens: Mapping<AccountId, bool>,

        valid: Mapping<AccountId, bool>,
        historically_valid: Mapping<AccountId, bool>,
    }


    impl PSP22 for GovernedTokenWrapper {}

    impl PSP22Metadata for GovernedTokenWrapper {}

    impl PSP22Mintable for GovernedTokenWrapper {}

    impl PSP22Wrapper for GovernedTokenWrapper {}

    /// The token wrapper error types.
    #[derive(Debug, PartialEq, Eq, scale::Encode, scale::Decode)]
    #[cfg_attr(feature = "std", derive(scale_info::TypeInfo))]
    pub enum Error {
        /// Invalid amount provided for native wrapping
        InvalidAmountForNativeWrapping,
        /// Native wrapping is not allowed for this token wrapper
        NativeWrappingNotAllowed,
        /// Invalid value sent for wrapping
        InvalidValueSentForWrapping,
        /// Invalid token address
        InvalidTokenAddress,
        /// Invalid token amount
        InvalidTokenAmount
    }

    impl GovernedTokenWrapper {
        #[ink(constructor)]
        pub fn new(total_supply: Balance, name: Option<String>, symbol: Option<String>, decimal: u8,
        governor: AccountId, native_token_denom: String, fee_recipient: AccountId, fee_percentage: Balance,
        is_native_allowed: bool, wrapping_limit: u128, proposal_nonce: u64, token_address: AccountId) -> Self {
            ink_lang::codegen::initialize_contract(|instance: &mut Self| {
                // for wrapping
                instance._init(token_address);

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
                instance.fee_percentage = fee_percentage;
                instance.is_native_allowed = is_native_allowed;
                instance.wrapping_limit = wrapping_limit;
                instance.proposal_nonce = proposal_nonce;
            })
        }


        #[ink(message, payable)]
        pub fn wrap(&mut self, token_address: Option<AccountId>, amount: Balance) {
            self.is_valid_wrapping(token_address, amount);

            let amount_to_use = if token_address.is_none() {
                self.env().transferred_value()
            } else {
                amount
            };

            let cost_to_wrap = self.get_fee_from_amount(amount_to_use);

            let leftover = amount_to_use.saturating_mul(cost_to_wrap);

            if token_address.is_none() {
                // mint the native value sent to the contract
                self.mint(self.env().caller(), leftover);

                // transfer costToWrap to the feeRecipient
                if self
                    .env()
                    .transfer(self.fee_recipient, cost_to_wrap)
                    .is_err()
                {
                    panic!("{}", ERROR_MSG);
                }
            } else {

                // psp22 transfer of  liquidity to token wrapper contract
                self.transfer_from(self.env().caller(), self.env().account_id(),  leftover, Vec::<u8>::new()).is_ok();

                // psp22 transfer to fee recipient
                self.transfer_from(self.env().caller(), self.fee_recipient,  cost_to_wrap, Vec::<u8>::new()).is_ok();

                // mint the wrapped token for the sender
                self.mint(self.env().caller(), leftover);
            }
        }

        fn is_valid_wrapping(&mut self, token_address: Option<AccountId>,
                             amount: Balance) -> Result<()> {

            if token_address.is_none() {
                if amount == 0 {
                    return Err(Error::InvalidAmountForNativeWrapping)
                }

                if !self.is_native_allowed {
                    return Err(Error::NativeWrappingNotAllowed)
                }

            } else {
                if  self.env().transferred_value() == 0 {
                    return Err(Error::InvalidValueSentForWrapping)
                }

                if !self.is_valid_address(token_address.unwrap()) {
                    return Err(Error::InvalidTokenAddress)
                }
            }

            if !self.is_valid_amount(amount) {
                return Err(Error::InvalidTokenAmount)
            }

            Ok(())
        }

        fn is_valid_address(&mut self, token_address: AccountId) -> bool {
            self.valid.get(token_address).is_some()
        }

        fn is_valid_amount(&mut self, amount: Balance) -> bool {
            amount.saturating_add(self.psp22.supply) <= self.wrapping_limit
        }

        fn get_fee_from_amount(&mut self, amount_to_wrap: Balance) -> Balance {
            amount_to_wrap.saturating_mul(self.fee_percentage).saturating_div(100)
        }
    }
}
