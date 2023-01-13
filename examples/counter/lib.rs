#![cfg_attr(not(feature = "std"), no_std)]

use ink_lang as ink;

#[ink::contract]
mod counter {
    use ink_storage::{traits::SpreadAllocate, Mapping};

    /// Counts number of calls
    #[ink(storage)]
    #[derive(SpreadAllocate)]
    pub struct Counter {
        calls: Mapping<AccountId, u128>,
    }

    use ink_lang::utils::initialize_contract;
    impl Counter {
        /// Constructor that initializes calls to 0
        #[ink(constructor)]
        pub fn new() -> Self {
            initialize_contract(|_| {
            })
        }

        /// Increases calls count by 1
        #[ink(message)]
        pub fn call_me(&mut self) {
            let account = self.env().caller();
            let caller_calls = self.get_calls(account);
            self.calls.insert(account, &(caller_calls + 1));
        }

        /// Returns number of calls
        #[ink(message)]
        pub fn get_calls(&self, account: AccountId) -> u128 {
            return match self.calls.get(&account) {
                Some(value) => value,
                None => 0,
            }
        }
    }
}
