#![cfg_attr(not(feature = "std"), no_std)]

use ink_lang as ink;

#[ink::contract]
mod treasury {
    use brush::contracts::psp22::*;
    use brush::contracts::traits::psp22::PSP22;
    use ink_storage::{traits::SpreadAllocate, Mapping};
    use protocol_ink_lib::utils::{is_account_id_zero, ZERO_ADDRESS};
    use ink_prelude::vec::Vec;


    /// The treasury result type.
    pub type Result<T> = core::result::Result<T, Error>;

    #[ink(storage)]
    #[derive(SpreadAllocate, PSP22Storage)]
    pub struct Treasury {
        #[PSP22StorageField]
        psp22: PSP22Data,
        treasury_handler: AccountId,
        proposal_nonce: u32,
    }

    /// The treasury error types.
    #[derive(Debug, PartialEq, Eq, scale::Encode, scale::Decode)]
    #[cfg_attr(feature = "std", derive(scale_info::TypeInfo))]
    pub enum Error {
        /// Unauthorized
        Unauthorized,
        /// Invalid Nonce
        InvalidNonce,
        /// Invalid Rescue Amount
        InvalidRescueAmount,
        /// Invalid Address
        InvalidAddress,
        /// Transfer Error
        TransferError,
    }

    impl Treasury {
        #[ink(constructor)]
        pub fn new(treasury_handler: AccountId) -> Self {
            ink_lang::codegen::initialize_contract(|instance: &mut Self| {
                instance.treasury_handler = treasury_handler;
                instance.proposal_nonce = 0;
            })
        }

        #[ink(message, payable)]
        pub fn rescue_tokens(
            &mut self,
            token_address: AccountId,
            to: AccountId,
            amount_to_rescue: Balance,
            nonce: u32,
        ) -> Result<()> {
            if is_account_id_zero(to) {
                return Err(Error::InvalidAddress);
            }

            if self.env().caller() != self.treasury_handler {
                return Err(Error::Unauthorized);
            }

            if self.proposal_nonce > nonce || self.proposal_nonce + 1048 < nonce {
                return Err(Error::InvalidNonce);
            }

            if amount_to_rescue == 0 {
                return Err(Error::InvalidRescueAmount);
            }

            if is_account_id_zero(token_address) {
                let native_balance = self.env().balance();

                if native_balance >= amount_to_rescue {
                    if self.env().transfer(to, amount_to_rescue).is_err() {
                        return Err(Error::TransferError);
                    }
                } else {
                    if self.env().transfer(to, native_balance).is_err() {
                        return Err(Error::TransferError);
                    }
                }
            } else {
                let psp22_balance = self.balance_of(self.env().account_id());
                if psp22_balance >= amount_to_rescue {
                    if self
                        .transfer_from(
                            self.env().account_id(),
                            to,
                            amount_to_rescue,
                            Vec::<u8>::new(),
                        )
                        .is_err()
                    {
                        return Err(Error::TransferError);
                    }
                } else {
                    if self
                        .transfer_from(self.env().account_id(), to, psp22_balance, Vec::<u8>::new())
                        .is_err()
                    {
                        return Err(Error::TransferError);
                    }
                }
            }

            self.proposal_nonce = nonce;
            Ok(())
        }

        /// Used to set new treasury handler address
        ///
        /// * `handler` -  The new address to be set
        /// * `nonce` - The nonce to use
        #[ink(message)]
        pub fn set_handler(&mut self, handler: AccountId, nonce: u32) -> Result<()> {
            if self.env().caller() != self.treasury_handler {
                return Err(Error::Unauthorized);
            }

            if self.proposal_nonce > nonce || self.proposal_nonce + 1048 < nonce {
                return Err(Error::InvalidNonce);
            }

            self.proposal_nonce = nonce;
            self.treasury_handler = handler;

            Ok(())
        }
    }
}
