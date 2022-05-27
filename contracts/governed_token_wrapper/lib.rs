#![cfg_attr(not(feature = "std"), no_std)]
#![feature(min_specialization)]

use ink_lang as ink;

#[brush::contract]
mod governed_token_wrapper {
    use brush::contracts::psp22::extensions::burnable::*;
    use brush::contracts::psp22::extensions::metadata::*;
    use brush::contracts::psp22::extensions::mintable::*;
    use brush::contracts::psp22::extensions::wrapper::*;
    use brush::contracts::psp22::*;
    use brush::contracts::traits::psp22::PSP22;
    use brush::test_utils::*;
    use ink_prelude::string::String;
    use ink_prelude::vec::Vec;
    use ink_storage::traits::{PackedLayout, SpreadLayout, StorageLayout};
    use ink_storage::{traits::SpreadAllocate, Mapping};

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

    impl PSP22Burnable for GovernedTokenWrapper {}

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
        /// Invalid token address
        ValidTokenAddress,
        /// Invalid token amount
        InvalidTokenAmount,
        /// Insufficient native balance
        InsufficientNativeBalance,
        /// Native unwrapping is not allowed for this token wrapper
        NativeUnwrappingNotAllowed,
        /// Insufficient PSP22 balance
        InsufficientPSP22Balance,
        /// Invalid historical token address
        InvalidHistoricalTokenAddress,
        /// Unauthorized
        Unauthorize,
        /// Invalid Nonce
        InvalidNonce,
        /// Nonce must increment by 1
        NonceMustIncrementByOne,
    }

    impl GovernedTokenWrapper {
        #[ink(constructor)]
        pub fn new(
            name: Option<String>,
            symbol: Option<String>,
            decimal: u8,
            governor: AccountId,
            fee_recipient: AccountId,
            fee_percentage: Balance,
            is_native_allowed: bool,
            wrapping_limit: u128,
            proposal_nonce: u64,
            token_address: AccountId,
        ) -> Self {
            ink_lang::codegen::initialize_contract(|instance: &mut Self| {
                // for wrapping
                instance._init(token_address);

                instance.metadata.name = name;
                instance.metadata.symbol = symbol;
                instance.metadata.decimals = decimal;

                // Governance config
                instance.governor = governor;
                instance.fee_recipient = fee_recipient;
                instance.fee_percentage = fee_percentage;
                instance.is_native_allowed = is_native_allowed;
                instance.wrapping_limit = wrapping_limit;
                instance.proposal_nonce = proposal_nonce;
            })
        }

        /// Used to wrap tokens on behalf of a sender.
        ///
        /// token_address is the address of PSP22 to transfer to, if token_address is None,
        /// then it's a Native token address
        ///
        /// amount is the amount of token to transfer
        #[ink(message, payable)]
        pub fn wrap(&mut self, token_address: Option<AccountId>, amount: Balance) {
            self.is_valid_wrapping(token_address, amount);

            // determine amount to use
            let amount_to_use = if token_address.is_none() {
                self.env().transferred_value()
            } else {
                amount
            };

            let cost_to_wrap = self.get_fee_from_amount(amount_to_use);

            let leftover = amount_to_use.saturating_sub(cost_to_wrap);

            self.do_wrap(
                token_address.clone(),
                self.env().caller(),
                self.env().caller(),
                cost_to_wrap,
                leftover,
            );
        }

        /// Used to unwrap/burn the wrapper token on behalf of a sender.
        ///
        /// token_address is the address of PSP22 to transfer to, if token_address is None,
        /// then it's a Native token address
        ///
        /// amount is the amount of token to transfer
        #[ink(message, payable)]
        pub fn unwrap(&mut self, token_address: Option<AccountId>, amount: Balance) {
            self.is_valid_unwrapping(token_address, amount);

            self.do_unwrap(
                token_address.clone(),
                self.env().caller(),
                self.env().caller(),
                amount,
            );
        }

        /// Used to unwrap/burn the wrapper token on behalf of a sender.
        ///
        /// token_address is the address of PSP22 to unwrap into,
        ///
        /// amount is the amount of tokens to burn
        ///
        /// recipient is the address to transfer to
        #[ink(message, payable)]
        pub fn unwrap_and_send_to(
            &mut self,
            token_address: Option<AccountId>,
            amount: Balance,
            recipient: AccountId,
        ) {
            self.is_valid_unwrapping(token_address, amount);

            self.do_unwrap(
                token_address.clone(),
                recipient,
                self.env().caller(),
                amount,
            );
        }

        /// Used to wrap tokens on behalf of a sender
        ///
        /// token_address is the address of PSP22 to unwrap into,
        ///
        /// amount is the amount of tokens to transfer
        ///
        /// sender is the Address of sender where assets are sent from.
        #[ink(message, payable)]
        pub fn wrap_for(
            &mut self,
            token_address: Option<AccountId>,
            sender: AccountId,
            amount: Balance,
        ) {
            self.is_valid_wrapping(token_address, amount);

            // determine amount to use
            let amount_to_use = if token_address.is_none() {
                self.env().transferred_value()
            } else {
                amount
            };

            let cost_to_wrap = self.get_fee_from_amount(amount_to_use);

            let leftover = amount_to_use.saturating_sub(cost_to_wrap);

            self.do_wrap(
                token_address.clone(),
                sender,
                sender,
                cost_to_wrap,
                leftover,
            );
        }
        /// Used to wrap tokens on behalf of a sender and mint to a potentially different address
        ///
        /// token_address is the address of PSP22 to unwrap into,
        ///
        /// sender is Address of sender where assets are sent from.
        ///
        /// amount is the amount of tokens to transfer
        ///
        /// Recipient is the recipient of the wrapped tokens.
        #[ink(message, payable)]
        pub fn wrap_for_and_send_to(
            &mut self,
            token_address: Option<AccountId>,
            sender: AccountId,
            amount: Balance,
            recipient: AccountId,
        ) {
            self.is_valid_wrapping(token_address, amount);

            // determine amount to use
            let amount_to_use = if token_address.is_none() {
                self.env().transferred_value()
            } else {
                amount
            };

            let cost_to_wrap = self.get_fee_from_amount(amount_to_use);

            let leftover = amount_to_use.saturating_sub(cost_to_wrap);

            self.do_wrap(
                token_address.clone(),
                sender,
                recipient,
                cost_to_wrap,
                leftover,
            );
        }

        /// Used to unwrap/burn the wrapper token on behalf of a sender.
        ///
        /// token_address is the address of PSP22 to transfer to, if token_address is None,
        /// then it's a Native token address
        ///
        /// amount is the amount of token to transfer
        ///
        /// sender is the Address of sender where liquidity is send to.
        #[ink(message, payable)]
        pub fn unwrap_for(
            &mut self,
            token_address: Option<AccountId>,
            amount: Balance,
            sender: AccountId,
        ) {
            self.is_valid_unwrapping(token_address, amount);
            self.do_unwrap(token_address.clone(), sender, sender, amount);
        }

        ///  Adds a token at `_tokenAddress` to the GovernedTokenWrapper's wrapping list
        ///
        /// tokenAddress:  The address of the token to be added
        ///
        /// nonce: The nonce tracking updates to this contract
        #[ink(message)]
        pub fn add_token_address(&mut self, token_address: AccountId, nonce: u64) -> Result<()> {
            // only contract governor can execute this function
            self.is_governor(self.env().caller());

            // check if token address already exists
            if self.is_valid_address(token_address) {
                return Err(Error::ValidTokenAddress);
            }

            if self.proposal_nonce > nonce {
                return Err(Error::InvalidNonce);
            }

            if nonce != self.proposal_nonce + 1 {
                return Err(Error::NonceMustIncrementByOne);
            }

            self.valid.insert(token_address, &true);
            self.historically_valid.insert(token_address, &true);
            self.tokens.insert(token_address, &true);
            self.historical_tokens.insert(token_address, &true);

            self.proposal_nonce = nonce;

            Ok(())
        }

        ///  Removes a token at `_tokenAddress` from the GovernedTokenWrapper's wrapping list
        ///
        /// tokenAddress:  The address of the token to be added
        ///
        /// nonce: The nonce tracking updates to this contract
        pub fn remove_token_address(&mut self, token_address: AccountId, nonce: u64) -> Result<()> {
            self.is_governor(self.env().caller());

            // check if token address already exists
            if !self.is_valid_address(token_address) {
                return Err(Error::InvalidTokenAddress);
            }

            if self.proposal_nonce > nonce {
                return Err(Error::InvalidNonce);
            }

            if nonce != self.proposal_nonce + 1 {
                return Err(Error::NonceMustIncrementByOne);
            }

            self.valid.insert(token_address, &false);
            self.tokens.insert(token_address, &false);

            self.proposal_nonce = nonce;
            Ok(())
        }
        /// Handles unwrapping by transferring token to the sender and burning for the burn_for address
        fn do_unwrap(
            &mut self,
            token_address: Option<AccountId>,
            sender: AccountId,
            burn_for: AccountId,
            amount: Balance,
        ) {
            // burn wrapped token from sender
            self.burn(burn_for, amount);

            if token_address.is_none() {
                // transfer native liquidity from the token wrapper to the sender
                if self.env().transfer(sender, amount).is_err() {
                    panic!("{}", ERROR_MSG);
                }
            } else {
                // transfer PSP22 liquidity from the token wrapper to the sender
                self.transfer(sender, amount, Vec::<u8>::new()).is_ok();
            }
        }

        /// Handles wrapping by transferring token to the sender and minting for the mint_for address
        fn do_wrap(
            &mut self,
            token_address: Option<AccountId>,
            sender: AccountId,
            mint_for: AccountId,
            cost_to_wrap: Balance,
            leftover: Balance,
        ) {
            if token_address.is_none() {
                // mint the native value sent to the contract
                self.mint(mint_for, leftover);

                // transfer costToWrap to the feeRecipient
                if self
                    .env()
                    .transfer(self.fee_recipient, cost_to_wrap)
                    .is_err()
                {
                    panic!("{}", ERROR_MSG);
                }
            } else {
                // psp22 transfer of liquidity to token wrapper contract
                self.transfer_from(sender, self.env().account_id(), leftover, Vec::<u8>::new())
                    .is_ok();

                // psp22 transfer to fee recipient
                self.transfer_from(sender, self.fee_recipient, cost_to_wrap, Vec::<u8>::new())
                    .is_ok();

                // mint the wrapped token for the sender
                self.mint(mint_for, leftover);
            }
        }

        /// Checks to determine if it's safe to wrap
        fn is_valid_wrapping(
            &mut self,
            token_address: Option<AccountId>,
            amount: Balance,
        ) -> Result<()> {
            if token_address.is_none() {
                if amount != 0 {
                    return Err(Error::InvalidAmountForNativeWrapping);
                }

                if !self.is_native_allowed {
                    return Err(Error::NativeWrappingNotAllowed);
                }
            } else {
                if self.env().transferred_value() != 0 {
                    return Err(Error::InvalidValueSentForWrapping);
                }

                if !self.is_valid_address(token_address.unwrap()) {
                    return Err(Error::InvalidTokenAddress);
                }
            }

            if !self.is_valid_amount(amount) {
                return Err(Error::InvalidTokenAmount);
            }

            Ok(())
        }

        fn is_valid_unwrapping(
            &mut self,
            token_address: Option<AccountId>,
            amount: Balance,
        ) -> Result<()> {
            if token_address.is_none() {
                if amount >= self.env().balance() {
                    return Err(Error::InsufficientNativeBalance);
                }

                if !self.is_native_allowed {
                    return Err(Error::NativeUnwrappingNotAllowed);
                }
            } else {
                if amount >= self.balance_of(self.env().account_id()) {
                    return Err(Error::InsufficientPSP22Balance);
                }

                if !self.is_address_historically_valid(token_address.unwrap()) {
                    return Err(Error::InvalidHistoricalTokenAddress);
                }
            }

            Ok(())
        }

        /// Determines if token address is a valid one
        fn is_valid_address(&mut self, token_address: AccountId) -> bool {
            self.valid.get(token_address).is_some()
        }

        /// Determines if token address is historically valid
        fn is_address_historically_valid(&mut self, token_address: AccountId) -> bool {
            self.historically_valid.get(token_address).is_some()
        }

        /// Determines if amount is valid for wrapping
        fn is_valid_amount(&mut self, amount: Balance) -> bool {
            amount.saturating_add(self.psp22.supply) <= self.wrapping_limit
        }

        /// Calculates the fee to be sent to fee recipient
        fn get_fee_from_amount(&mut self, amount_to_wrap: Balance) -> Balance {
            amount_to_wrap
                .saturating_mul(self.fee_percentage)
                .saturating_div(100)
        }

        fn is_governor(&mut self, address: AccountId) -> Result<()> {
            if self.governor != address {
                return Err(Error::Unauthorize);
            }

            Ok(())
        }
    }
}
