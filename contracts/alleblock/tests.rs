use core::fmt::Debug;

use crate::alleblock::{self, *};
use ink_env::{AccountId, DefaultEnvironment};
/// Imports `ink_lang` so we can use `#[ink::test]`.
use ink_lang as ink;

// helper functions

fn set_caller_and_update_balance(account: AccountId, new_balance: Option<u128>) {
    ink_env::test::set_caller::<DefaultEnvironment>(account);
    if let Some(balance) = new_balance {
        ink_env::test::set_balance::<DefaultEnvironment>(account, balance);
    }
}

fn set_value_transferred(amount: u128) {
    ink_env::test::set_value_transferred::<DefaultEnvironment>(amount)
}

fn assert_account_balance_equals(account: AccountId, balance: u128) {
    assert_eq!(
        ink_env::test::get_account_balance::<DefaultEnvironment>(account),
        Ok(balance)
    )
}

fn assert_call_result_eq<T: PartialEq + Debug>(
    call_result: alleblock::Result<T>,
    expected: alleblock::Result<T>,
) {
    assert_eq!(call_result, expected);
    if expected.is_ok() {
        ink_env::test::transfer_in::<DefaultEnvironment>(ink_env::transferred_value::<
            DefaultEnvironment,
        >());
    }
}

// tests

#[ink::test]
fn creation_test() {
    let contract = Alleblock::new(1, 2);
    assert_eq!(contract.get_create_auction_fee(), 1);
    assert_eq!(contract.get_finalize_fee(), 2);
}

#[ink::test]
fn no_such_auction_test() {
    let mut contract = Alleblock::new(1, 2);
    assert_call_result_eq(contract.bid(0), Err(Error::NoSuchAuctionError));
    assert_call_result_eq(contract.finish_auction(0), Err(Error::NoSuchAuctionError));
    assert_call_result_eq(contract.cancel_auction(0), Err(Error::NoSuchAuctionError));
}

#[ink::test]
fn create_auction_test() {
    let mut contract = Alleblock::new(1, 2);
    let accounts = ink_env::test::default_accounts::<DefaultEnvironment>();

    set_caller_and_update_balance(accounts.bob, Some(1000));
    set_value_transferred(1);

    assert_call_result_eq(
        contract.create_auction(5, "auction 1".to_string(), 5),
        Ok(0),
    );
    assert_call_result_eq(
        contract.create_auction(15, "auction 2".to_string(), 3),
        Ok(1),
    );

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
            auction_state: AuctionState::InProgress,
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
            auction_state: AuctionState::InProgress,
        },
    ];
    assert_eq!(contract.get_auctions(), expected_auctions)
}

#[ink::test]
fn auction_creation_fee_test() {
    let auction_creation_fee = 10;
    let mut contract = Alleblock::new(auction_creation_fee, 20);
    let accounts = ink_env::test::default_accounts::<DefaultEnvironment>();

    set_caller_and_update_balance(accounts.bob, Some(1000));

    set_value_transferred(auction_creation_fee - 1);
    assert_call_result_eq(
        contract.create_auction(5, "auction 1".to_string(), 3),
        Err(Error::TooLowFeeError),
    );

    set_value_transferred(auction_creation_fee);
    assert_call_result_eq(
        contract.create_auction(5, "auction 1".to_string(), 3),
        Ok(0),
    );
}

#[ink::test]
fn bid_state_errors_test() {
    let mut contract = Alleblock::new(10, 20);
    let accounts = ink_env::test::default_accounts::<DefaultEnvironment>();

    set_caller_and_update_balance(accounts.bob, Some(1000));
    set_value_transferred(10);

    assert_call_result_eq(
        contract.create_auction(5, "normal auction".to_string(), 100),
        Ok(0),
    );
    assert_call_result_eq(
        contract.create_auction(5, "finalized auction".to_string(), 3),
        Ok(1),
    );
    assert_call_result_eq(
        contract.create_auction(5, "cancelled auction".to_string(), 100),
        Ok(2),
    );
    assert_call_result_eq(
        contract.create_auction(5, "auction after deadline".to_string(), 3),
        Ok(3),
    );
    let (normal_id, finalized_id, cancelled_id, after_deadline_id) = (0, 1, 2, 3);

    ink_env::test::advance_block::<DefaultEnvironment>();
    assert_call_result_eq(contract.finish_auction(finalized_id), Ok(()));
    assert_call_result_eq(contract.cancel_auction(cancelled_id), Ok(()));

    set_caller_and_update_balance(accounts.eve, Some(1000));
    set_value_transferred(500);
    assert_call_result_eq(contract.bid(normal_id), Ok(()));
    assert_call_result_eq(
        contract.bid(finalized_id),
        Err(Error::AuctionNotInProgressError),
    );
    assert_call_result_eq(
        contract.bid(cancelled_id),
        Err(Error::AuctionNotInProgressError),
    );
    assert_call_result_eq(
        contract.bid(after_deadline_id),
        Err(Error::AfterFinishDateError),
    );

    assert_eq!(
        accounts.eve,
        contract.get_auctions()[normal_id as usize].actual_winner
    );
    assert_eq!(
        AuctionState::InProgress,
        contract.get_auctions()[normal_id as usize].auction_state
    );
}

#[ink::test]
fn bid_test() {
    let minimum_bid = 5;
    let mut contract = Alleblock::new(10, 20);
    let accounts = ink_env::test::default_accounts::<DefaultEnvironment>();

    set_caller_and_update_balance(accounts.bob, Some(1000));
    set_value_transferred(10);
    assert_call_result_eq(
        contract.create_auction(minimum_bid, "auction".to_string(), 100),
        Ok(0),
    );

    set_caller_and_update_balance(accounts.eve, Some(1000));

    set_value_transferred(minimum_bid - 1);
    assert_call_result_eq(contract.bid(0), Err(Error::TooLowBidError));
    assert_eq!(accounts.bob, contract.get_auctions()[0].actual_winner);
    assert_eq!(0, contract.get_auctions()[0].actual_bid);

    set_value_transferred(minimum_bid);
    assert_call_result_eq(contract.bid(0), Ok(()));
    assert_eq!(accounts.eve, contract.get_auctions()[0].actual_winner);
    assert_eq!(minimum_bid, contract.get_auctions()[0].actual_bid);

    let eve_best_bid = 500;
    set_value_transferred(eve_best_bid);
    assert_call_result_eq(contract.bid(0), Ok(()));
    assert_eq!(eve_best_bid, contract.get_auctions()[0].actual_bid);

    set_caller_and_update_balance(accounts.charlie, Some(1500));

    set_value_transferred(eve_best_bid);
    assert_call_result_eq(contract.bid(0), Err(Error::TooLowBidError));
    assert_eq!(accounts.eve, contract.get_auctions()[0].actual_winner);
    assert_eq!(eve_best_bid, contract.get_auctions()[0].actual_bid);

    set_value_transferred(eve_best_bid + 1);
    assert_call_result_eq(contract.bid(0), Ok(()));
    assert_eq!(accounts.charlie, contract.get_auctions()[0].actual_winner);
    assert_eq!(eve_best_bid + 1, contract.get_auctions()[0].actual_bid);
    assert_account_balance_equals(accounts.eve, 1000);
}

#[ink::test]
fn finalize_state_errors_test() {
    let mut contract = Alleblock::new(10, 20);
    let accounts = ink_env::test::default_accounts::<DefaultEnvironment>();

    set_caller_and_update_balance(accounts.bob, Some(1000));
    set_value_transferred(10);

    assert_call_result_eq(
        contract.create_auction(5, "auction before deadline".to_string(), 100),
        Ok(0),
    );
    assert_call_result_eq(
        contract.create_auction(5, "cancelled auction".to_string(), 100),
        Ok(1),
    );
    assert_call_result_eq(
        contract.create_auction(5, "auction after deadline".to_string(), 3),
        Ok(2),
    );
    assert_call_result_eq(
        contract.create_auction(5, "auction after deadline - other finishes".to_string(), 3),
        Ok(3),
    );
    let (before_deadline_id, cancelled_id, after_deadline_id, after_deadline_other_finishes_id) =
        (0, 1, 2, 3);

    ink_env::test::advance_block::<DefaultEnvironment>();
    assert_call_result_eq(contract.cancel_auction(cancelled_id), Ok(()));
    set_caller_and_update_balance(accounts.eve, Some(1000));
    assert_call_result_eq(
        contract.finish_auction(after_deadline_other_finishes_id),
        Ok(()),
    );

    set_caller_and_update_balance(accounts.bob, None);
    assert_call_result_eq(
        contract.finish_auction(before_deadline_id),
        Err(Error::BeforeFinishDateError),
    );
    assert_call_result_eq(
        contract.finish_auction(cancelled_id),
        Err(Error::AuctionNotInProgressError),
    );
    assert_call_result_eq(contract.finish_auction(after_deadline_id), Ok(()));
    assert_call_result_eq(
        contract.finish_auction(after_deadline_id),
        Err(Error::AuctionNotInProgressError),
    );

    assert_eq!(
        AuctionState::Finished,
        contract.get_auctions()[2].auction_state
    );
}

#[ink::test]
fn finish_behaviour_test() {
    let finalize_fee = 20;
    let mut contract = Alleblock::new(10, finalize_fee);
    let accounts = ink_env::test::default_accounts::<DefaultEnvironment>();

    set_caller_and_update_balance(accounts.bob, Some(1010));
    set_value_transferred(10);
    assert_call_result_eq(
        contract.create_auction(5, "auction 1".to_string(), 3),
        Ok(0),
    );
    let bob_balance_after_contract_creation = 1000;

    let eve_bid = 500;
    set_caller_and_update_balance(accounts.eve, Some(1000));
    set_value_transferred(eve_bid);
    assert_call_result_eq(contract.bid(0), Ok(()));

    set_caller_and_update_balance(accounts.bob, None);
    ink_env::test::advance_block::<DefaultEnvironment>();
    set_value_transferred(0);
    assert_call_result_eq(contract.finish_auction(0), Ok(()));

    assert_account_balance_equals(
        accounts.bob,
        bob_balance_after_contract_creation + eve_bid - eve_bid / (finalize_fee as u128),
    );
}

#[ink::test]
fn cancel_state_errors_test() {
    let mut contract = Alleblock::new(10, 20);
    let accounts = ink_env::test::default_accounts::<DefaultEnvironment>();

    set_caller_and_update_balance(accounts.bob, Some(1000));
    set_value_transferred(10);

    assert_call_result_eq(
        contract.create_auction(5, "auction before deadline".to_string(), 100),
        Ok(0),
    );
    assert_call_result_eq(
        contract.create_auction(5, "finished auction".to_string(), 3),
        Ok(1),
    );
    assert_call_result_eq(
        contract.create_auction(5, "auction after deadline".to_string(), 3),
        Ok(2),
    );
    let (before_deadline_id, finished_id, after_deadline_id) = (0, 1, 2);

    ink_env::test::advance_block::<DefaultEnvironment>();
    assert_call_result_eq(contract.finish_auction(finished_id), Ok(()));

    set_caller_and_update_balance(accounts.eve, Some(1000));
    assert_call_result_eq(
        contract.cancel_auction(before_deadline_id),
        Err(Error::NotACreatorError),
    );
    assert_call_result_eq(
        contract.cancel_auction(after_deadline_id),
        Err(Error::NotACreatorError),
    );

    set_caller_and_update_balance(accounts.bob, None);
    assert_call_result_eq(contract.cancel_auction(before_deadline_id), Ok(()));
    assert_call_result_eq(
        contract.cancel_auction(finished_id),
        Err(Error::AuctionNotInProgressError),
    );
    assert_call_result_eq(contract.cancel_auction(after_deadline_id), Ok(()));
    assert_call_result_eq(
        contract.cancel_auction(after_deadline_id),
        Err(Error::AuctionNotInProgressError),
    );

    assert_eq!(
        AuctionState::Cancelled,
        contract.get_auctions()[0].auction_state
    );
    assert_eq!(
        AuctionState::Cancelled,
        contract.get_auctions()[2].auction_state
    );
}

#[ink::test]
fn cancel_behaviour_test() {
    let mut contract = Alleblock::new(10, 20);
    let accounts = ink_env::test::default_accounts::<DefaultEnvironment>();

    set_caller_and_update_balance(accounts.bob, Some(1010));
    set_value_transferred(10);
    assert_call_result_eq(
        contract.create_auction(5, "auction 1".to_string(), 3),
        Ok(0),
    );

    set_caller_and_update_balance(accounts.eve, Some(1000));
    set_value_transferred(500);
    assert_call_result_eq(contract.bid(0), Ok(()));

    set_caller_and_update_balance(accounts.bob, None);
    let fee = 500 / 20;

    set_value_transferred(fee - 1);
    assert_call_result_eq(contract.cancel_auction(0), Err(Error::TooLowFeeError));
    assert_eq!(
        contract.get_auctions()[0].auction_state,
        AuctionState::InProgress
    );

    set_value_transferred(fee);
    assert_call_result_eq(contract.cancel_auction(0), Ok(()));
    assert_account_balance_equals(accounts.eve, 1000);
}
