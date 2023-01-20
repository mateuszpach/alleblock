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
        Finished,
        Cancelled
    }

    #[derive(Debug, PartialEq, Eq, scale::Encode, scale::Decode)]
    #[cfg_attr(feature = "std", derive(scale_info::TypeInfo))]
    pub enum Error {
        TooLowBidError,
        TooLowFeeError,
        AfterFinishDateError,
        BeforeFinishDateError,
        AuctionNotInProgressError,
        NoSuchAuctionError,
        NotACreatorError,
        TransferError
    }

    #[derive(PackedLayout, PartialEq, SpreadLayout, scale::Encode, scale::Decode, Clone, Debug)]
    #[cfg_attr(feature = "std", derive(::scale_info::TypeInfo))]
    pub struct AuctionInfo {
        pub id: u64,
        pub creator: AccountId,
        pub description: String,
        pub minimal_bid: u128,
        pub actual_bid: u128,
        pub actual_winner: AccountId,
        pub creation_date: Timestamp,
        pub finish_date: Timestamp,
        pub auction_state: AuctionState
    }
    

    #[ink(storage)]
    pub struct Alleblock {
        /// list of all contract's auctions
        auctions: Vec<AuctionInfo>,

        /// fee for creating the auction
        create_auction_fee: u128,

        /// fraction of the final price that contract takes as fee
        /// the contract will take <actual_bid>/<finalize_fee> currency
        finalize_fee: u32,
    }

    /// result type
    pub type Result<T> = core::result::Result<T, Error>;

    impl Alleblock {
        /// constructor setting height of the fees
        /// finalize_fee shouldn't be set to 0
        #[ink(constructor)]
        pub fn new(create_auction_fee: u128, _finalize_fee: u32) -> Self {
            let finalize_fee = if _finalize_fee == 0 {1} else {_finalize_fee};
            Self { 
                auctions: Vec::new(),
                create_auction_fee,
                finalize_fee
            }
        }

        /// message used to create a brand new auction
        /// minimal_bid -- lowest price at which the item can be sold (in the smallest chunk of currency, eg. picoTZERO)
        /// description -- description of item or service
        /// duration -- duration of auction in seconds, after creating the auction, everyone can bid for <duration> seconds
        #[ink(message, payable)]
        pub fn create_auction(&mut self, minimal_bid: u128, description: String, duration: u64) -> Result<u64> {
            if self.create_auction_fee > self.env().transferred_value() {
                return Err(Error::TooLowFeeError);
            }

            let creation_date = self.env().block_timestamp();
            let finish_date = creation_date + duration;
            let auction_id = self.auctions.len() as u64;

            let fresh_auction = AuctionInfo {
                id: auction_id,
                creator: self.env().caller(),
                description,
                minimal_bid,
                actual_bid: 0,
                actual_winner: self.env().caller(),
                creation_date,
                finish_date,
                auction_state: AuctionState::InProgress
            };

            self.auctions.push(fresh_auction);

            return Ok(auction_id);
        }


        /// try to bid auction with given id
        #[ink(message, payable)]
        pub fn bid(&mut self, auction_id: u64) -> Result<()> {
            let caller = self.env().caller();
            let transferred_value = self.env().transferred_value();
            let block_timestamp = self.env().block_timestamp();

            let auction = match self.auctions.get(auction_id as usize) {
                Some(x) => x,
                None => return Err(Error::NoSuchAuctionError)
            };

            // check if auction is in progress
            if auction.auction_state != AuctionState::InProgress {
                return Err(Error::AuctionNotInProgressError);
            }

            // perform only before auction finish date
            if block_timestamp > auction.finish_date {
                return Err(Error::AfterFinishDateError);
            }

            // check if enough money is transferred
            if transferred_value <= auction.actual_bid || transferred_value < auction.minimal_bid {
                return Err(Error::TooLowBidError);
            }

            // transfer money back to the previous winner
            if auction.actual_bid > 0 {
                if self.env().transfer(auction.actual_winner, auction.actual_bid).is_err() {
                    return Err(Error::TransferError);
                }
            }

            // update auction data
            let auction_mut = match self.auctions.get_mut(auction_id as usize) {
                Some(x) => x,
                None => return Err(Error::NoSuchAuctionError)
            };
            auction_mut.actual_winner = caller;
            auction_mut.actual_bid = transferred_value;

            return Ok(());
        }


        /// finish an auction, so the owner gets the auction money
        /// can be called only after the auction's finish date
        /// changes the auction state to Finished
        #[ink(message)]
        pub fn finish_auction(&mut self, auction_id: u64) -> Result<()> {
            let block_timestamp = self.env().block_timestamp();

            let auction = match self.auctions.get(auction_id as usize) {
                Some(x) => x,
                None => return Err(Error::NoSuchAuctionError)
            };

            // check if auction is in progress
            if auction.auction_state != AuctionState::InProgress {
                return Err(Error::AuctionNotInProgressError);
            }

            // perform only after auction finish date
            if block_timestamp <= auction.finish_date {
                return Err(Error::BeforeFinishDateError);
            }

            // transfer money to the auction creator
            if auction.actual_bid > 0 {
                let service_fee = auction.actual_bid.div_euclid(self.finalize_fee as u128);
                if self.env().transfer(auction.creator, auction.actual_bid - service_fee).is_err() {
                    return Err(Error::TransferError);
                }
            }

            // update auction data
            let auction_mut = match self.auctions.get_mut(auction_id as usize) {
                Some(x) => x,
                None => return Err(Error::NoSuchAuctionError)
            };
            auction_mut.auction_state = AuctionState::Finished;

            return Ok(());
        }

        /// cancel an auction 
        /// only auction creator can call this message
        /// money is returned to the bidder
        /// creator has to pay the fee
        /// changes auction stated to Cancelled
        #[ink(message, payable)]
        pub fn cancel_auction(&mut self, auction_id: u64) -> Result<()> {
            let caller = self.env().caller();
            let transferred_value = self.env().transferred_value();

            let auction = match self.auctions.get(auction_id as usize) {
                Some(x) => x,
                None => return Err(Error::NoSuchAuctionError)
            };

            // check if auction is in progress
            if auction.auction_state != AuctionState::InProgress {
                return Err(Error::AuctionNotInProgressError);
            }

            // check if auction creator is the caller
            if caller != auction.creator {
                return Err(Error::NotACreatorError);
            }

            // if anyone has bid an auction
            if auction.actual_bid > 0 {
                // check if fee is high enough
                let service_fee = auction.actual_bid.div_euclid(self.finalize_fee as u128);
                if transferred_value < service_fee {
                    return Err(Error::TooLowFeeError);
                }

                // return the money to the actual winner
                if self.env().transfer(auction.actual_winner, auction.actual_bid).is_err() {
                    return Err(Error::TransferError);
                }
            }

            // update auction data
            let auction_mut = match self.auctions.get_mut(auction_id as usize) {
                Some(x) => x,
                None => return Err(Error::NoSuchAuctionError)
            };
            auction_mut.auction_state = AuctionState::Cancelled;

            return Ok(());
        }

        /// return list of all the auctions
        #[ink(message)]
        pub fn get_auctions(&self) -> Vec<AuctionInfo> {
            return self.auctions.clone();
        }

        /// return fee needed to crate an auction
        #[ink(message)]
        pub fn get_create_auction_fee(&self) -> u128 {
            return self.create_auction_fee.clone();
        }

        /// return fee taken from finalized auction
        #[ink(message)]
        pub fn get_finalize_fee(&self) -> u32 {
            return self.finalize_fee.clone();
        }
    }

}

#[cfg(test)]
mod tests;