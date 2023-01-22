#![cfg_attr(not(feature = "std"), no_std)]

use ink_lang as ink;

pub use self::nft_storage::NftStorageRef;

#[ink::contract]
mod nft_storage {
    use openbrush::contracts::traits::psp34::Id;
    use openbrush::contracts::psp34::PSP34Error;
    use openbrush::contracts::traits::psp34::PSP34Ref;

    #[derive(Debug, PartialEq, Eq, scale::Encode, scale::Decode)]
    #[cfg_attr(feature = "std", derive(scale_info::TypeInfo))]
    pub enum Error {
        NotACreatorError,
    }

    #[ink(storage)]
    pub struct NftStorage {
        creator: AccountId,
        owner: AccountId,
    }

    pub type Result<T> = core::result::Result<T, Error>;

    impl NftStorage {
        /// Constructor that remembers creator of this contract
        #[ink(constructor)]
        pub fn new() -> Self {
            Self { 
                creator: Self::env().caller(),
                owner: Self::env().caller(),
            }
        }

        /// Set owner of this contract
        #[ink(message)]
        pub fn set_owner(&mut self, owner: AccountId) -> Result<()> {
            if self.env().caller() != self.creator && self.env().caller() != self.owner {
                return Err(Error::NotACreatorError)
            } 
            self.owner = owner;
            return Ok(());
        }

        /// transfer given token to given address
        #[ink(message)]
        pub fn transfer(&mut self, to: AccountId, nft_account: AccountId, nft_token: Id) -> core::result::Result<(), PSP34Error> { 
            return PSP34Ref::transfer(&nft_account, to, nft_token, [0x0].to_vec());
        }
    }
}
