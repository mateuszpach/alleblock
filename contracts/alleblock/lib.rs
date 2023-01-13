#![cfg_attr(not(feature = "std"), no_std)]

use ink_lang as ink;

#[ink::contract]
mod alleblock {
    use ink_prelude::{string::String, vec::Vec};
    use ink_storage::traits::{SpreadAllocate, PackedLayout, SpreadLayout};
    use ink_storage::Mapping;

    #[derive(PackedLayout,SpreadLayout, Debug, PartialEq, Eq, scale::Encode, scale::Decode, Clone)]
    #[cfg_attr(feature = "std", derive(scale_info::TypeInfo))]
    pub enum AuctionState {
        InProgress,
        Terminated,
        Finished
    }


    #[derive(PackedLayout, SpreadLayout, scale::Encode, scale::Decode, Clone, Debug)]
    #[cfg_attr(feature = "std", derive(::scale_info::TypeInfo))]
    pub struct AuctionInfo {
        id: u128,
        creator: AccountId,
        description: String,
        minimal_bid: u128,
        actual_bid: u128,
        actual_winner: AccountId,
        creation_date: Timestamp,
        finish_date: Timestamp,
        auction_state: AuctionState
    }
    

    #[ink(storage)]
    // #[derive(SpreadAllocate)]
    pub struct Alleblock {
        // auctions: Mapping<u128, AuctionInfo>,
        auctions: Vec<AuctionInfo>
    }

    use ink_lang::utils::initialize_contract;
    impl Alleblock {
        #[ink(constructor)]
        pub fn new() -> Self {
            Self { auctions: Vec::new() }
        }

        #[ink(message)]
        pub fn create_auction(&mut self, starting_price: u128, description: String, duration: u128) {
            
        }

        #[ink(message)]
        pub fn bid(&mut self, auction_id: u128) {
            
        }

        #[ink(message)]
        pub fn finish_auction(&mut self, auction_id: u128) {
            
        }


        #[ink(message)]
        pub fn terminate_auction(&mut self, auction_id: u128) {
            
        }

        #[ink(message)]
        pub fn get_auctions(&self) -> Vec<AuctionInfo> {
            return self.auctions.clone();
        }
    }

    #[cfg(test)]
    mod tests {
        /// Imports all the definitions from the outer scope so we can use them here.
        use super::*;

        /// Imports `ink_lang` so we can use `#[ink::test]`.
        use ink_lang as ink;
    }
}
