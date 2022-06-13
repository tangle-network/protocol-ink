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

    pub const ZERO_ADDRESS: [u8; 32] = [0; 32];

    /// The contract storage
    #[ink(storage)]
    #[derive(Default, SpreadAllocate, PSP22Storage, PSP22WrapperStorage, PSP22MetadataStorage)]
    pub struct GovernedTokenWrapper {
        #[PSP22StorageField]
        psp22: PSP22Data,
        #[PSP22MetadataStorageField]
        metadata: PSP22MetadataData,
        #[PSP22WrapperStorageField]
        wrapper: PSP22WrapperData,

        /// The contract governor
        governor: AccountId,
        /// The address of the fee recipient
        fee_recipient: AccountId,
        /// The percentage fee for wrapping
        fee_percentage: Balance,
        /// To determine if native wrapping is allowed
        is_native_allowed: bool,
        /// The contract wrapping limit
        wrapping_limit: Balance,
        /// The nonce for adding/removing address
        proposal_nonce: u64,
        /// Map of token addresses
        tokens: Mapping<AccountId, bool>,
        /// Map of historical token addresses
        historical_tokens: Mapping<AccountId, bool>,
        /// Map of tokens that are valid
        valid: Mapping<AccountId, bool>,
        /// Map of tokens that are historically valid
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
        /// Token Address already exists
        TokenAddressAlreadyExists,
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
        /// TransferError
        TransferError,
        /// Token Mint Error
        TokenMintError,
        /// Token Burn Error
        TokenBurnError,
        /// PSP22 allowance error
        PSP22AllowanceError,
    }

    #[ink(event)]
    pub struct Wrap {
        #[ink(topic)]
        sender: Option<AccountId>,
        #[ink(topic)]
        mint_for: Option<AccountId>,
        amount: Balance,
    }

    impl GovernedTokenWrapper {
        /// Initializes the contract
        ///
        /// # Arguments
        /// * `name` - The contract's token name
        /// * `symbol` - The contract's token symbol
        /// * `decimal` - The contract's decimal value
        /// * `governor` - The contract's governor
        /// * `fee_recipient` - The contract's fee recipient address
        /// * `fee_percentage` - The contract's fee percentage
        /// * `is_native_allowed` - Determines if the contract should allow native token wrapping
        /// * `wrapping_limit` - The contract's wrapping limit
        /// * `proposal_nonce` - The nonce used for adding/removing token address
        #[ink(constructor)]
        pub fn new(
            name: Option<String>,
            symbol: Option<String>,
            decimal: u8,
            governor: AccountId,
            fee_recipient: AccountId,
            fee_percentage: Balance,
            is_native_allowed: bool,
            wrapping_limit: Balance,
            proposal_nonce: u64,
            total_supply: Balance,
        ) -> Self {
            ink_lang::codegen::initialize_contract(|instance: &mut Self| {
                instance.metadata.name = name;
                instance.metadata.symbol = symbol;
                instance.metadata.decimals = decimal;

                instance.psp22.supply = total_supply;

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
        /// * `token_address` - The address of PSP22 to wrap into, if token_address is None,
        /// then it's a Native token address
        /// * `amount` - The amount of token to transfer
        #[ink(message, payable)]
        pub fn wrap(&mut self, token_address: AccountId, amount: Balance) -> Result<()> {
            self.is_valid_wrapping(token_address, amount)?;

            // determine amount to use
            let amount_to_use = self.determine_amount_to_use(token_address, amount);

            let cost_to_wrap = self.get_fee_from_amount(amount_to_use);
            let leftover = amount_to_use.saturating_sub(cost_to_wrap);

            self.do_wrap(
                token_address.clone(),
                self.env().caller(),
                self.env().caller(),
                cost_to_wrap,
                leftover,
            )?;

            Ok(())
        }

        /// Used to unwrap/burn the wrapper token on behalf of a sender.
        ///
        /// * `token_address` -  The optional address of PSP22 to unwrap into, if token_address is None,
        /// then it's a Native token address
        /// * `amount` -  The the amount of token to transfer
        #[ink(message, payable)]
        pub fn unwrap(&mut self, token_address: AccountId, amount: Balance) -> Result<()> {
            self.is_valid_unwrapping(token_address, amount)?;

            self.do_unwrap(
                token_address.clone(),
                self.env().caller(),
                self.env().caller(),
                amount,
            )?;

            Ok(())
        }

        /// Used to unwrap/burn the wrapper token on behalf of a sender.
        ///
        /// * `token_address` - is the optional address of PSP22 to unwrap into,
        /// * `amount` - is the amount of tokens to burn
        /// * `recipient` is the address to transfer to
        #[ink(message, payable)]
        pub fn unwrap_and_send_to(
            &mut self,
            token_address: AccountId,
            amount: Balance,
            recipient: AccountId,
        ) -> Result<()> {
            self.is_valid_unwrapping(token_address, amount)?;

            self.do_unwrap(
                token_address.clone(),
                recipient,
                self.env().caller(),
                amount,
            )?;

            Ok(())
        }

        /// Used to wrap tokens on behalf of a sender
        ///
        /// * `token_address` - is the optional Account id of PSP22 to wrap into,
        /// * `amount` - is the amount of tokens to transfer
        /// * `sender` -  is the Account id of sender where assets are sent from.
        #[ink(message, payable)]
        pub fn wrap_for(
            &mut self,
            token_address: AccountId,
            sender: AccountId,
            amount: Balance,
        ) -> Result<()> {
            self.is_valid_wrapping(token_address, amount)?;

            // determine amount to use
            let amount_to_use = self.determine_amount_to_use(token_address, amount);

            let cost_to_wrap = self.get_fee_from_amount(amount_to_use);
            let leftover = amount_to_use.saturating_sub(cost_to_wrap);

            self.do_wrap(
                token_address.clone(),
                sender,
                sender,
                cost_to_wrap,
                leftover,
            )?;

            self.env().emit_event(Wrap {
                sender: Some(sender),
                mint_for: Some(sender),
                amount,
            });

            Ok(())
        }

        /// Used to wrap tokens on behalf of a sender and mint to a potentially different address
        ///
        /// * `token_address` - is the optional address of PSP22 to wrap into,
        /// * `sender` - is Address of sender where assets are sent from.
        /// * `amount` - is the amount of tokens to transfer
        /// * `recipient` - is the recipient of the wrapped tokens.
        #[ink(message, payable)]
        pub fn wrap_for_and_send_to(
            &mut self,
            token_address: AccountId,
            sender: AccountId,
            amount: Balance,
            recipient: AccountId,
        ) -> Result<()> {
            self.is_valid_wrapping(token_address, amount)?;

            // determine amount to use
            let amount_to_use = self.determine_amount_to_use(token_address, amount);

            let cost_to_wrap = self.get_fee_from_amount(amount_to_use);

            let leftover = amount_to_use.saturating_sub(cost_to_wrap);

            self.do_wrap(
                token_address.clone(),
                sender,
                recipient,
                cost_to_wrap,
                leftover,
            )?;

            Ok(())
        }

        /// Used to unwrap/burn the wrapper token on behalf of a sender.
        ///
        /// * `token_address` - is the optional address of PSP22 to unwrap into, if token_address is None,
        ///  then it's a Native token address
        /// * `amount` - is the amount of token to transfer
        /// * `sender` - is the Address of sender where liquidity is send to.
        #[ink(message, payable)]
        pub fn unwrap_for(
            &mut self,
            token_address: AccountId,
            amount: Balance,
            sender: AccountId,
        ) -> Result<()> {
            self.is_valid_unwrapping(token_address, amount)?;
            self.do_unwrap(token_address.clone(), sender, sender, amount)?;

            Ok(())
        }

        /// Adds a token at `token_address` to the GovernedTokenWrapper's wrapping list
        ///
        /// * `token_address` - The address of the token to be added
        /// * `nonce` -  The nonce tracking updates to this contract
        #[ink(message)]
        pub fn add_token_address(&mut self, token_address: AccountId, nonce: u64) -> Result<()> {
            // only contract governor can execute this function
            self.is_governor(self.env().caller())?;

            // check if token address already exists
            if self.is_valid_address(token_address) {
                return Err(Error::TokenAddressAlreadyExists);
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

        /// Removes a token at `token_address` from the GovernedTokenWrapper's wrapping list
        ///
        /// * `token_address`:  The address of the token to be added
        /// * `nonce`: The nonce tracking updates to this contract
        #[ink(message)]
        pub fn remove_token_address(&mut self, token_address: AccountId, nonce: u64) -> Result<()> {
            self.is_governor(self.env().caller())?;

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

        /// Updates contract configs
        ///
        /// * `governor` - Sets the contract's governor
        /// * `is_native_allowed` - Determines if the contract should allow native token wrapping
        /// * `wrapping_limit` - Sets the contract's wrapping limit
        /// * `fee_percentage` - Sets the contract's fee percentage
        /// * `fee_recipient` - Sets the contract's fee recipient address
        #[ink(message)]
        pub fn update_config(
            &mut self,
            governor: Option<AccountId>,
            is_native_allowed: Option<bool>,
            wrapping_limit: Option<Balance>,
            fee_percentage: Option<Balance>,
            fee_recipient: Option<AccountId>,
        ) -> Result<()> {
            // only contract governor can execute this function
            self.is_governor(self.env().caller())?;

            if governor.is_some() {
                self.governor = governor.unwrap();
            }

            if is_native_allowed.is_some() {
                self.is_native_allowed = is_native_allowed.unwrap();
            }

            if wrapping_limit.is_some() {
                self.wrapping_limit = wrapping_limit.unwrap_or(self.wrapping_limit);
            }

            if fee_percentage.is_some() {
                self.fee_percentage = fee_percentage.unwrap();
            }

            if fee_recipient.is_some() {
                self.fee_recipient = fee_recipient.unwrap();
            }

            Ok(())
        }

        /// Handles unwrapping by transferring token to the sender and burning for the burn_for address
        ///
        /// * `token_address` - is the address of PSP22 to unwrap into,
        /// * `sender` - is Address of sender where assets are sent from.
        /// * `burn_for` - is the address that the token gets burnt from.
        /// * `amount` - is the amount of tokens to transfer
        fn do_unwrap(
            &mut self,
            token_address: AccountId,
            sender: AccountId,
            burn_for: AccountId,
            amount: Balance,
        ) -> Result<()> {
            // burn wrapped token from sender
            if self.burn(burn_for, amount).is_err() {
                return Err(Error::TokenBurnError);
            }

            if self.is_account_id_zero(token_address) {
                // transfer native liquidity from the token wrapper to the sender
                if self.env().transfer(sender, amount).is_err() {
                    return Err(Error::TransferError);
                }
            } else {
                // transfer PSP22 liquidity from the token wrapper to the sender
                if self
                    .transfer_from(self.env().account_id(), sender, amount, Vec::<u8>::new())
                    .is_err()
                {
                    return Err(Error::TransferError);
                }
            }

            Ok(())
        }

        /// Handles wrapping by transferring token to the sender and minting for the mint_for address
        ///
        /// * `token_address` - Is the address of PSP22 to unwrap into,
        /// * `sender` - Is Address of sender where assets are sent from.
        /// * `mint_for` - Is the address that the token gets burnt from.
        /// * `cost_to_wrap` - Is the cost for wrapping token.
        /// * `leftover` - Is the amount of leftover.
        fn do_wrap(
            &mut self,
            token_address: AccountId,
            sender: AccountId,
            mint_for: AccountId,
            cost_to_wrap: Balance,
            leftover: Balance,
        ) -> Result<()> {
            if self.is_account_id_zero(token_address) {
                // mint the native value sent to the contract
                if self.mint(mint_for, leftover).is_err() {
                    return Err(Error::TokenMintError);
                }

                // transfer costToWrap to the feeRecipient
                if self
                    .env()
                    .transfer(self.fee_recipient, cost_to_wrap)
                    .is_err()
                {
                    return Err(Error::TransferError);
                }
            } else {
                // psp22 transfer of liquidity to token wrapper contract
                if sender == self.env().caller() {
                    if self
                        .transfer(self.env().account_id(), leftover, Vec::<u8>::new())
                        .is_err()
                    {
                        return Err(Error::TransferError);
                    }
                } else {
                    if self
                        .transfer_from(sender, self.env().account_id(), leftover, Vec::<u8>::new())
                        .is_err()
                    {
                        return Err(Error::TransferError);
                    }
                }

                // psp22 transfer to fee recipient
                if self
                    .transfer_from(sender, self.fee_recipient, cost_to_wrap, Vec::<u8>::new())
                    .is_err()
                {
                    return Err(Error::TransferError);
                }

                // mint the wrapped token for the sender
                if self.mint(mint_for, leftover).is_err() {
                    return Err(Error::TransferError);
                }
            }

            Ok(())
        }

        /// Determines the amount to use
        ///
        /// * `token_address` - is the optional address of PSP22 to wrap into,
        /// * `amount` - is the amount of tokens to transfer
        fn determine_amount_to_use(
            &mut self,
            token_address: AccountId,
            amount: Balance,
        ) -> Balance {
            if self.is_account_id_zero(token_address) {
                self.env().transferred_value()
            } else {
                amount
            }
        }

        /// Determines if an account is zero
        ///
        /// * `account_id` - an address to determine,
        fn is_account_id_zero(&mut self, account_id: AccountId) -> bool {
            account_id == ZERO_ADDRESS.into()
        }

        /// Checks to determine if it's safe to wrap
        ///
        /// * `token_address` - Is the address for wrapping,
        /// * `amount` - Is the amount for wrapping.
        fn is_valid_wrapping(&mut self, token_address: AccountId, amount: Balance) -> Result<()> {
            if self.is_account_id_zero(token_address) {
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

                if !self.is_valid_address(token_address) {
                    return Err(Error::InvalidTokenAddress);
                }

                if !self.is_valid_amount(amount) {
                    return Err(Error::InvalidTokenAmount);
                }
            }

            Ok(())
        }

        /// Checks to determine if it's safe to unwrap
        /// # Arguments
        ///
        /// * `token_address` - Is the address for unwrapping,
        /// * `amount` - Is the amount for unwrapping.
        fn is_valid_unwrapping(&mut self, token_address: AccountId, amount: Balance) -> Result<()> {
            if self.is_account_id_zero(token_address) {
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

                if !self.is_address_historically_valid(token_address) {
                    return Err(Error::InvalidHistoricalTokenAddress);
                }
            }

            Ok(())
        }

        /// Determines if token address is a valid one
        ///
        /// * `token_address` - The token address to chcek
        fn is_valid_address(&mut self, token_address: AccountId) -> bool {
            let is_valid = self.valid.get(token_address).unwrap_or(false);
            is_valid
        }

        /// Determines if token address is historically valid
        ///
        /// * `token_address` - The token address to check
        fn is_address_historically_valid(&mut self, token_address: AccountId) -> bool {
            self.historically_valid.get(token_address).is_some()
        }

        /// Determines if amount is valid for wrapping
        ///
        /// * `amount` - The amount
        fn is_valid_amount(&mut self, amount: Balance) -> bool {
            let amount_add_supply = amount.saturating_add(self.psp22.supply);

            amount_add_supply <= self.wrapping_limit
        }

        /// Calculates the fee to be sent to fee recipient
        ///
        /// * `amount_to_wrap` - The amount to wrap
        fn get_fee_from_amount(&mut self, amount_to_wrap: Balance) -> Balance {
            amount_to_wrap
                .saturating_mul(self.fee_percentage)
                .saturating_div(100)
        }

        /// Determine if an account id/address is a governor
        ///
        /// * `address` - The address to check
        fn is_governor(&mut self, address: AccountId) -> Result<()> {
            if self.governor != address {
                return Err(Error::Unauthorize);
            }

            Ok(())
        }

        /// Returns the `governor` value.
        #[ink(message)]
        pub fn governor(&self) -> AccountId {
            self.governor
        }

        /// Returns the `is_native_allowed` value.
        #[ink(message)]
        pub fn is_native_allowed(&self) -> bool {
            self.is_native_allowed
        }

        /// Returns the `wrapping_limit` value.
        #[ink(message)]
        pub fn wrapping_limit(&self) -> u128 {
            self.wrapping_limit
        }

        /// Returns the `fee_percentage` value.
        #[ink(message)]
        pub fn fee_percentage(&self) -> Balance {
            self.fee_percentage
        }

        /// Returns the `fee_recipient` value.
        #[ink(message)]
        pub fn fee_recipient(&self) -> AccountId {
            self.fee_recipient
        }

        /// Returns the token `name` .
        #[ink(message)]
        pub fn name(&self) -> Option<String> {
            self.metadata.name.clone()
        }

        /// Returns the `proposal_nonce` value.
        #[ink(message)]
        pub fn nonce(&self) -> u64 {
            self.proposal_nonce
        }

        /// Checks if a token_address is a valid one.
        ///
        /// * `token_address` - The address to check
        #[ink(message)]
        pub fn is_valid_token_address(&self, token_address: AccountId) -> bool {
            self.valid.get(token_address).unwrap()
        }

        /// Returns total psp22 token supply
        #[ink(message)]
        pub fn total_supply(&self) -> Balance {
            self.psp22.supply
        }

        /// Returns psp22 balance for an address
        ///
        /// * `address` - The address to check
        #[ink(message)]
        pub fn psp22_balance(&self, address: AccountId) -> Balance {
            self.balance_of(address)
        }

        /// Updates psp22 contract balance
        ///
        /// * `amount` - the amount
        #[ink(message)]
        pub fn update_psp22_contract_balance(&mut self, amount: Balance) {
            let account_id = self.env().account_id();
            self.psp22.balances.insert(&account_id, &amount);
        }

        /// Transfers psp22 token to contract
        ///
        /// * `amount` - the amount
        #[ink(message)]
        pub fn transfer_psp22_to_contract(&mut self, amount: Balance) -> Result<()> {
            let account_id = self.env().account_id();
            if self
                .transfer_from(self.governor, account_id, amount, Vec::<u8>::new())
                .is_err()
            {
                return Err(Error::TransferError);
            }
            Ok(())
        }

        /// Returns contract psp22 balance
        #[ink(message)]
        pub fn psp22_contract_balance(&self) -> Balance {
            self.balance_of(self.env().account_id())
        }

        /// Returns native contract balance
        #[ink(message)]
        pub fn native_contract_balance(&self) -> Balance {
            self.env().balance()
        }

        /// Returns native contract address
        #[ink(message)]
        pub fn native_contract_account_id(&self) -> Option<AccountId> {
            Some(self.env().account_id())
        }

        /// sets the psp22 allowance for the spender(spend on behalf of owner)
        #[ink(message)]
        pub fn set_psp22_allowance(&mut self, spender: AccountId, amount: Balance) -> Result<()> {
            // psp22 call to increase allowance
            if self.increase_allowance(spender, amount).is_err() {
                return Err(Error::PSP22AllowanceError);
            }
            Ok(())
        }

        /// sets the psp22 allowance for the spender(spend on behalf of owner)
        ///
        /// * `owner` - owner's address
        /// * `spender` - spender's address
        /// * `amount` - amount to spend
        #[ink(message)]
        pub fn set_psp22_allowance_for_owner(
            &mut self,
            owner: AccountId,
            spender: AccountId,
            amount: Balance,
        ) -> Result<()> {
            // psp22 call to increase allowance
            self.psp22.allowances.insert((owner, spender), &amount);
            Ok(())
        }

        /// Gets the psp22 allowance for the spender(spend on behalf of owner)
        ///
        /// * `owner` - owner's address
        /// * `spender` - spender's address
        #[ink(message)]
        pub fn get_psp22_allowance(&self, owner: AccountId, spender: AccountId) -> Balance {
            self.allowance(owner, spender)
        }

        /// Returns the zero address represented by the contract
        #[ink(message)]
        pub fn get_zero_address(&self) -> AccountId {
            ZERO_ADDRESS.into()
        }

        /// Transfer's psp22 token to an address
        ///
        /// * `account_id` - address to transfer to
        /// * `amount` - amount to transfer
        #[ink(message)]
        pub fn transfer_psp22(&mut self, account_id: AccountId, amount: Balance) -> Result<()> {
            if self.transfer(account_id, amount, Vec::<u8>::new()).is_err() {
                return Err(Error::TransferError);
            }
            Ok(())
        }

        /// Transfer's native token to an address
        ///
        /// * `account_id` - address to transfer to
        /// * `amount` - amount to transfer
        #[ink(message, payable)]
        pub fn transfer_native(&mut self, account_id: AccountId, amount: Balance) -> Result<()> {
            if self.env().transfer(account_id, amount).is_err() {
                return Err(Error::TransferError);
            }
            Ok(())
        }

        /// Insert's psp22 token for an address
        ///
        /// * `account_id` - address to transfer to
        /// * `amount` - amount to transfer
        #[ink(message)]
        pub fn insert_psp22_balance(
            &mut self,
            account_id: AccountId,
            amount: Balance,
        ) -> Result<()> {
            self.psp22.balances.insert(&account_id, &amount);
            Ok(())
        }
    }
}
