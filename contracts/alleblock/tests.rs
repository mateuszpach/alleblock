

mod tests {
    use crate::alleblock::*;

    use ink_env::{DefaultEnvironment, AccountId};
    /// Imports `ink_lang` so we can use `#[ink::test]`.
    use ink_lang as ink;

    // helper functions

    fn set_caller(account: AccountId, money: Option<u128>) {
        ink_env::test::set_caller::<DefaultEnvironment>(account);
        match money {
            Some(balance) => { ink_env::test::set_balance::<DefaultEnvironment>(account, balance); }
            None => {}
        }
    }
    
    fn set_transfer(amount: u128) {
        ink_env::test::set_value_transferred::<DefaultEnvironment>(amount)
    }

    fn assert_balance_equals(account: AccountId, money: u128) {
        assert_eq!(ink_env::test::get_account_balance::<DefaultEnvironment>(account), Ok(money))
    }

    // actual tests 

    #[ink::test]
    fn creation_test() {
        let contract = Alleblock::new(1, 2);
        assert_eq!(contract.get_create_auction_fee(), 1);
        assert_eq!(contract.get_finalize_fee(), 2);
    }

    
    #[ink::test]
    fn no_such_auction_test() {
        let mut contract = Alleblock::new(1, 2);
        assert_eq!(contract.bid(0), Err(Error::NoSuchAuctionError));
        assert_eq!(contract.finish_auction(0), Err(Error::NoSuchAuctionError));
        assert_eq!(contract.cancel_auction(0), Err(Error::NoSuchAuctionError));
    }

    #[ink::test]
    fn create_auction_test() {
        let mut contract = Alleblock::new(1, 2);
        let accounts = 
            ink_env::test::default_accounts::<DefaultEnvironment>();

        set_caller(accounts.bob, Some(1000));
        set_transfer(1);
        
        assert_eq!(contract.create_auction(5, "auction 1".to_string(), 5), Ok(0));
        assert_eq!(contract.create_auction(15, "auction 2".to_string(), 3), Ok(1));
        let caller = ink_env::caller::<DefaultEnvironment>();
        let expected_auctions = vec![
            AuctionInfo {
                id: 0,
                creator: caller,
                description: "auction 1".to_string(),
                minimal_bid: 5,
                actual_bid: 0,
                actual_winner: caller,
                creation_date: 0,
                finish_date: 5,
                auction_state: AuctionState::InProgress
            },
            AuctionInfo {
                id: 1,
                creator: caller,
                description: "auction 2".to_string(),
                minimal_bid: 15,
                actual_bid: 0,
                actual_winner: caller,
                creation_date: 0,
                finish_date: 3,
                auction_state: AuctionState::InProgress
            },

        ];
        assert_eq!(contract.get_auctions(), expected_auctions)
    }

    #[ink::test]
    fn auction_creation_fee_test() {
        let mut contract = Alleblock::new(10, 20);
        let accounts = 
            ink_env::test::default_accounts::<DefaultEnvironment>();
        
        set_caller(accounts.bob, Some(1000));
        set_transfer(9);
        assert_eq!( contract.create_auction(5, "auction 1".to_string(), 3), Err(Error::TooLowFeeError));
        set_transfer(10);
        assert_eq!( contract.create_auction(5, "auction 1".to_string(), 3), Ok(0));
    }
    
    #[ink::test]
    fn bid_state_errors_test() {
        let mut contract = Alleblock::new(10, 20);
        let accounts = 
            ink_env::test::default_accounts::<DefaultEnvironment>();
        
        set_caller(accounts.bob, Some(1000));
        set_transfer(10);
        assert_eq!( contract.create_auction(5, "normal auction".to_string(), 100), Ok(0));
        assert_eq!( contract.create_auction(5, "finalized auction".to_string(), 3), Ok(1));
        assert_eq!( contract.create_auction(5, "cancelled auction".to_string(), 100), Ok(2));
        assert_eq!( contract.create_auction(5, "auction after deadline".to_string(), 3), Ok(3));
        
        ink_env::test::advance_block::<DefaultEnvironment>();
        assert_eq!( contract.finish_auction(1), Ok(()));
        assert_eq!( contract.cancel_auction(2), Ok(()));

        set_caller(accounts.alice, Some(1000));
        set_transfer(500);
        assert_eq!(contract.bid(0), Ok(()));
        assert_eq!(contract.bid(1), Err(Error::AuctionNotInProgressError));
        assert_eq!(contract.bid(2), Err(Error::AuctionNotInProgressError));
        assert_eq!(contract.bid(3), Err(Error::AfterFinishDateError));

        assert_eq!(accounts.alice, contract.get_auctions()[0].actual_winner);
        assert_eq!(AuctionState::InProgress, contract.get_auctions()[0].auction_state);
    }

    #[ink::test]
    fn bid_test() {
        let mut contract = Alleblock::new(10, 20);
        let accounts = 
            ink_env::test::default_accounts::<DefaultEnvironment>();
        
        set_caller(accounts.bob, Some(1000));
        set_transfer(10);
        assert_eq!( contract.create_auction(5, "auction".to_string(), 100), Ok(0));
        
        set_caller(accounts.alice, Some(1000));
        set_transfer(4);
        assert_eq!(contract.bid(0), Err(Error::TooLowBidError));
        assert_eq!(accounts.bob, contract.get_auctions()[0].actual_winner);
        assert_eq!(0, contract.get_auctions()[0].actual_bid);
        set_transfer(5);
        assert_eq!(contract.bid(0), Ok(()));
        assert_eq!(accounts.alice, contract.get_auctions()[0].actual_winner);
        assert_eq!(5, contract.get_auctions()[0].actual_bid);
        set_transfer(500);
        assert_eq!(contract.bid(0), Ok(()));
        assert_eq!(500, contract.get_auctions()[0].actual_bid);

        set_caller(accounts.charlie, Some(1500));
        set_transfer(500);
        assert_eq!(contract.bid(0), Err(Error::TooLowBidError));
        assert_eq!(accounts.alice, contract.get_auctions()[0].actual_winner);
        assert_eq!(500, contract.get_auctions()[0].actual_bid);
        set_transfer(501);
        assert_eq!(contract.bid(0), Ok(()));
        assert_eq!(accounts.charlie, contract.get_auctions()[0].actual_winner);
        assert_eq!(501, contract.get_auctions()[0].actual_bid);
        assert_balance_equals(accounts.alice, 1505);

    }

    #[ink::test]
    fn finalize_state_errors_test() {
        let mut contract = Alleblock::new(10, 20);
        let accounts = 
            ink_env::test::default_accounts::<DefaultEnvironment>();
        
        set_caller(accounts.bob, Some(1000));
        set_transfer(10);
        assert_eq!( contract.create_auction(5, "auction before deadline".to_string(), 100), Ok(0));
        assert_eq!( contract.create_auction(5, "cancelled auction".to_string(), 100), Ok(1));
        assert_eq!( contract.create_auction(5, "auction after deadline".to_string(), 3), Ok(2));
        assert_eq!( contract.create_auction(5, "auction after deadline - other finishes".to_string(), 3), Ok(3));
        
        ink_env::test::advance_block::<DefaultEnvironment>();
        assert_eq!( contract.cancel_auction(1), Ok(()));

        set_caller(accounts.alice, Some(1000));
        assert_eq!(contract.finish_auction(3), Ok(()));

        set_caller(accounts.bob, None);
        assert_eq!(contract.finish_auction(0), Err(Error::BeforeFinishDateError));
        assert_eq!(contract.finish_auction(1), Err(Error::AuctionNotInProgressError));
        assert_eq!(contract.finish_auction(2), Ok(()));
        assert_eq!(contract.finish_auction(2), Err(Error::AuctionNotInProgressError));

        assert_eq!(AuctionState::Finished, contract.get_auctions()[2].auction_state);
    }

    #[ink::test]
    fn finish_behaviour_test() {
        let mut contract = Alleblock::new(10, 20);
        let accounts = 
            ink_env::test::default_accounts::<DefaultEnvironment>();
        
        set_caller(accounts.bob, Some(1000));
        set_transfer(10);
        assert_eq!( contract.create_auction(5, "auction 1".to_string(), 3), Ok(0));
        
        set_caller(accounts.alice, Some(1000));
        set_transfer(500);
        assert_eq!(contract.bid(0), Ok(()));

        set_caller(accounts.bob, None);
        ink_env::test::advance_block::<DefaultEnvironment>();
        assert_eq!( contract.finish_auction(0), Ok(()));
        assert_balance_equals(accounts.bob, 1000+500-500/20);
    }

    #[ink::test]
    fn cancel_state_errors_test() {
        let mut contract = Alleblock::new(10, 20);
        let accounts = 
            ink_env::test::default_accounts::<DefaultEnvironment>();
        
        set_caller(accounts.bob, Some(1000));
        set_transfer(10);
        assert_eq!( contract.create_auction(5, "auction before deadline".to_string(), 100), Ok(0));
        assert_eq!( contract.create_auction(5, "finished auction".to_string(), 3), Ok(1));
        assert_eq!( contract.create_auction(5, "auction after deadline".to_string(), 3), Ok(2));
        
        ink_env::test::advance_block::<DefaultEnvironment>();
        assert_eq!( contract.finish_auction(1), Ok(()));

        set_caller(accounts.alice, Some(1000));
        assert_eq!(contract.cancel_auction(0), Err(Error::NotACreatorError));
        assert_eq!(contract.cancel_auction(2), Err(Error::NotACreatorError));

        set_caller(accounts.bob, None);
        assert_eq!(contract.cancel_auction(0), Ok(()));
        assert_eq!(contract.cancel_auction(1), Err(Error::AuctionNotInProgressError));
        assert_eq!(contract.cancel_auction(2), Ok(()));
        assert_eq!(contract.cancel_auction(2), Err(Error::AuctionNotInProgressError));

        assert_eq!(AuctionState::Cancelled, contract.get_auctions()[0].auction_state);
        assert_eq!(AuctionState::Cancelled, contract.get_auctions()[2].auction_state);
    }
    
    #[ink::test]
    fn cancel_behaviour_test() {
        let mut contract = Alleblock::new(10, 20);
        let accounts = 
            ink_env::test::default_accounts::<DefaultEnvironment>();
        
        set_caller(accounts.bob, Some(1000));
        set_transfer(10);
        assert_eq!( contract.create_auction(5, "auction 1".to_string(), 3), Ok(0));
        
        set_caller(accounts.alice, Some(1000));
        set_transfer(500);
        assert_eq!(contract.bid(0), Ok(()));

        set_caller(accounts.bob, None);
        let fee = 500/20;
        set_transfer(fee-1);
        assert_eq!( contract.cancel_auction(0), Err(Error::TooLowFeeError));
        set_transfer(fee);
        assert_eq!( contract.cancel_auction(0), Ok(()));
        assert_balance_equals(accounts.alice, 1500);
    }

}