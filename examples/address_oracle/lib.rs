#![cfg_attr(not(feature = "std"), no_std)]

use ink_lang as ink;

#[ink::contract]
mod address_oracle {

    #[ink(storage)]
    pub struct AddressOracle {
    }

    impl AddressOracle {
        /// Constructor that does nothing
        #[ink(constructor)]
        pub fn new() -> Self {
            Self { }
        }

        /// returns array representing account id
        #[ink(message)]
        pub fn show_address(&self, account: AccountId) -> [u8; 32] {
            return *account.as_ref();
        }

        /// returns address representing array
        #[ink(message)]
        pub fn show_account_id(&self, account: [u8; 32] ) -> AccountId {
            return AccountId::from(account);
        }
    }
}
