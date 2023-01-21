#![cfg_attr(not(feature = "std"), no_std)]

use ink_lang as ink;

#[ink::contract]
mod nft_alleblock {
    use ink_prelude::{string::String, vec::Vec};
    use ink_storage::traits::{PackedLayout, SpreadLayout};

    // needed to call psp34 contracts
    use ink_env::DefaultEnvironment;
    use ink_env::call::{build_call, Call, ExecutionInput, Selector};
    use openbrush::contracts::traits::psp34::Id;
    use openbrush::contracts::psp34::PSP34Error;

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
        NotAnOwnerError,
        TransferError,
        NoNftAllowanceError,
        NftTransferError,
    }

    #[derive(PackedLayout, PartialEq, SpreadLayout, scale::Encode, scale::Decode, Clone, Debug)]
    #[cfg_attr(feature = "std", derive(::scale_info::TypeInfo))]
    pub struct AuctionInfo {
        pub id: u64,
        pub owner: AccountId,
        pub description: String,
        pub starting_bid: u128,
        pub highest_bid: u128,
        pub highest_bidder: AccountId,
        pub creation_date: Timestamp,
        pub finish_date: Timestamp,
        pub auction_state: AuctionState,
        pub nft_contract_account: Option<AccountId>,
        pub nft_token_id: Option<Id>,
    }
    

    #[ink(storage)]
    pub struct NftAlleblock {
        /// list of all contract's auctions
        auctions: Vec<AuctionInfo>,

        /// fee for creating the auction
        create_auction_fee: u128,

        /// fraction of the final price that contract takes as fee
        /// the contract will take <highest_bid>/<finalize_fee_interest> currency
        finalize_fee_interest: u32,

        /// account of the owner of this contract
        /// this account receives fees gathered by this contract
        contract_owner: AccountId,
    }

    /// result type
    pub type Result<T> = core::result::Result<T, Error>;

    impl NftAlleblock {
        /// constructor setting the fees
        /// finalize_fee shouldn't be set to 0
        #[ink(constructor)]
        pub fn new(create_auction_fee: u128, _finalize_fee_interest: u32, contract_owner: AccountId) -> Self {
            let finalize_fee_interest = if _finalize_fee_interest == 0 {1} else {_finalize_fee_interest};
            Self { 
                auctions: Vec::new(),
                create_auction_fee,
                finalize_fee_interest,
                contract_owner,
            }
        }

        /// message used to create a brand new auction
        /// starting_bid -- lowest price at which the item can be sold (in the smallest chunk of currency, eg. picoTZERO)
        /// description -- description of item or service
        /// duration -- duration of auction in seconds, after creating the auction, everyone can bid for <duration> seconds
        /// nft_contract -- account of the origin contract of the nft to be auctioned (None if not selling nft)
        /// token_id -- id of the token to be auctioned (None if not selling nft)
        #[ink(message, payable)]
        pub fn create_auction(&mut self, 
            starting_bid: u128,
            description: String,
            duration: u64, 
            nft_contract: Option<AccountId>,
            token_id: Option<Id>
        ) -> Result<u64> {
            let transferred_value = self.env().transferred_value();
            let owner = self.env().caller();

            let nft_contract_account: Option<AccountId>;
            let nft_token_id: Option<Id>;

            // check if paid fee is high enough
            if self.create_auction_fee > transferred_value {
                return Err(Error::TooLowFeeError);
            }

            // transfer fee to the contract owner
            if self.env().transfer(self.contract_owner, transferred_value).is_err() {
                return Err(Error::TransferError);
            }

            // if nft is auctioned
            if nft_contract.is_some() && token_id.is_some() {
                nft_contract_account = nft_contract;
                nft_token_id = token_id;
                let unwrapped_account = nft_contract_account.clone().unwrap();
                let unwrapped_token = nft_token_id.clone().unwrap();

                // check if contract has allowance to take the token
                if !self.have_allowance(owner, unwrapped_account.clone(), unwrapped_token.clone()) {
                    return Err(Error::NoNftAllowanceError);
                }

                // freeze the nft in the contract account
                if self.transfer_token(self.env().account_id(), unwrapped_account.clone(), unwrapped_token.clone()).is_err() {
                    return Err(Error::NftTransferError);
                }
            }

            // if no nft is auctioned
            else {
                nft_contract_account = None;
                nft_token_id = None;
            }

            // create new auction
            let creation_date = self.env().block_timestamp();
            let finish_date = creation_date + duration;
            let auction_id = self.auctions.len() as u64;

            let fresh_auction = AuctionInfo {
                id: auction_id,
                owner,
                description,
                starting_bid,
                highest_bid: 0,
                highest_bidder: owner,
                creation_date,
                finish_date,
                auction_state: AuctionState::InProgress,
                nft_contract_account,
                nft_token_id,
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
            if transferred_value <= auction.highest_bid || transferred_value < auction.starting_bid {
                return Err(Error::TooLowBidError);
            }

            // transfer money back to the previous bidder
            if auction.highest_bid > 0 {
                if self.env().transfer(auction.highest_bidder, auction.highest_bid).is_err() {
                    return Err(Error::TransferError);
                }
            }

            // update auction data
            let auction_mut = match self.auctions.get_mut(auction_id as usize) {
                Some(x) => x,
                None => return Err(Error::NoSuchAuctionError)
            };
            auction_mut.highest_bidder = caller;
            auction_mut.highest_bid = transferred_value;

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

            // if anyone bid the auction
            if auction.highest_bid > 0 {
                let service_fee = auction.highest_bid.div_euclid(self.finalize_fee_interest as u128);

                // transfer money to the auction owner
                if self.env().transfer(auction.owner, auction.highest_bid - service_fee).is_err() {
                    return Err(Error::TransferError);
                }

                // transfer fee to the contract owner
                if self.env().transfer(self.contract_owner, service_fee).is_err() {
                    return Err(Error::TransferError);
                }
            }

            // send nft to the winner
            if auction.nft_contract_account.is_some() {
                if self.transfer_token(auction.highest_bidder, auction.nft_contract_account.clone().unwrap(), auction.nft_token_id.clone().unwrap()).is_err() {
                    return Err(Error::NftTransferError);
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
        /// only auction owner can call this message
        /// money is returned to the bidder
        /// owner has to pay the fee
        /// changes auction stated to Cancelled
        #[ink(message, payable)]
        pub fn cancel_auction(&mut self, auction_id: u64) -> Result<()> {
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

            // check if auction owner is the caller
            if caller != auction.owner {
                return Err(Error::NotAnOwnerError);
            }

            // perform only before auction finish date
            if block_timestamp > auction.finish_date {
                return Err(Error::AfterFinishDateError);
            }

            // if anyone has bid an auction
            if auction.highest_bid > 0 {
                // check if fee is high enough
                let service_fee = auction.highest_bid.div_euclid(self.finalize_fee_interest as u128);
                if transferred_value < service_fee {
                    return Err(Error::TooLowFeeError);
                }

                // return the money to the highest bidder
                if self.env().transfer(auction.highest_bidder, auction.highest_bid).is_err() {
                    return Err(Error::TransferError);
                }
            }

            // return nft to the auction owner
            if auction.nft_contract_account.is_some() {
                if self.transfer_token(auction.owner, auction.nft_contract_account.clone().unwrap(), auction.nft_token_id.clone().unwrap()).is_err() {
                    return Err(Error::NftTransferError);
                }
            }

            // transfer fee to the contract owner
            if self.env().transfer(self.contract_owner, transferred_value).is_err() {
                return Err(Error::TransferError);
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

        /// return the fee needed to crate an auction
        #[ink(message)]
        pub fn get_create_auction_fee(&self) -> u128 {
            return self.create_auction_fee.clone();
        }

        /// return the fee interest taken from finalized auction
        #[ink(message)]
        pub fn get_finalize_fee_interest(&self) -> u32 {
            return self.finalize_fee_interest.clone();
        }

        /// return the fee taken when finalizing particular auction
        #[ink(message)]
        pub fn get_finalize_fee_of(&self, auction_id: u64) -> Result<u128> {
            let auction = match self.auctions.get(auction_id as usize) {
                Some(x) => x,
                None => return Err(Error::NoSuchAuctionError)
            };
            return Ok(auction.highest_bid.div_euclid(self.finalize_fee_interest as u128));
        }

        /// return owner of the contract who receives all the fees
        #[ink(message)]
        pub fn get_contract_owner(&self) -> AccountId {
            return self.contract_owner.clone();
        }

        /// check if you are allowed to take this psp34 token
        /// call allowance(owner: AccountId, operator: AccountId, id: Option) ➔ bool
        /// selector: 0x4790f55a
        fn have_allowance(&self, owner: AccountId, nft_contract: AccountId, token_id: Id) -> bool {
            return build_call::<DefaultEnvironment>()
                .call_type(Call::new().callee(nft_contract))
                .exec_input(
                    ExecutionInput::new(Selector::new([0x47, 0x90, 0xf5, 0x5a]))
                    .push_arg(owner)
                    .push_arg(self.env().caller())
                    .push_arg(token_id)
                )
                .returns::<bool>()
                .fire()
                .unwrap()     
        }

        /// transfer nft to indicated address
        /// call transfer(to: AccountId, id: Id, data: [u8]) ➔ Result<(), PSP34Error>
        /// selector: 0x3128d61b
        fn transfer_token(&mut self, to: AccountId, nft_contract: AccountId, token_id: Id) -> core::result::Result<(), PSP34Error> {
            return build_call::<DefaultEnvironment>()
                .call_type(Call::new().callee(nft_contract))
                .exec_input(
                    ExecutionInput::new(Selector::new([0x31, 0x28, 0xd6, 0x1b]))
                    .push_arg(to)
                    .push_arg(token_id)
                    .push_arg(0x0)
                )
                .returns::<core::result::Result<(), PSP34Error>>()
                .fire()
                .unwrap()     
        }
    }

}

#[cfg(test)]
mod tests;