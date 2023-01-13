#![cfg_attr(not(feature = "std"), no_std)]

use ink_lang as ink;

#[ink::contract]
mod alleblock {
    use ink_prelude::{string::String, vec::Vec};
    use ink_storage::traits::{PackedLayout, SpreadLayout};

    #[derive(PackedLayout,SpreadLayout, Debug, PartialEq, Eq, scale::Encode, scale::Decode, Clone)]
    #[cfg_attr(feature = "std", derive(scale_info::TypeInfo))]
    pub enum AuctionState {
        InProgress,
        Terminated,
        Finished
    }

    #[derive(Debug, PartialEq, Eq, scale::Encode, scale::Decode)]
    #[cfg_attr(feature = "std", derive(scale_info::TypeInfo))]
    pub enum Error {
        TooLowBidError,
        TooLowFeeError,
        AfterFinishDateError,
        AuctionNotInProgressError
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
    pub struct Alleblock {
        /// list of all contract's auctions
        auctions: Vec<AuctionInfo>,

        /// fee for creating the auction
        create_auction_fee: u128,

        /// fraction of the final price that contract takes as fee
        /// the contract will take actual_bid/finalize_fee currency
        finalize_fee: u32,
    }

    /// result type
    pub type Result<T> = core::result::Result<T, Error>;

    impl Alleblock {
        /// constructor setting height of the fees
        #[ink(constructor)]
        pub fn new(create_auction_fee: u128, finalize_fee: u32) -> Self {
            Self { 
                auctions: Vec::new(),
                create_auction_fee,
                finalize_fee
             }
        }

        #[ink(message, payable)]
        pub fn create_auction(&mut self, starting_price: u128, description: String, duration: u128) -> Result<()> {
            return Ok(())
        }

        #[ink(message, payable)]
        pub fn bid(&mut self, auction_id: u128) -> Result<()> {
            return Ok(())
        }

        #[ink(message)]
        pub fn finish_auction(&mut self, auction_id: u128) -> Result<()> {
            return Ok(())
        }


        #[ink(message, payable)]
        pub fn terminate_auction(&mut self, auction_id: u128) -> Result<()> {
            return Ok(())
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
