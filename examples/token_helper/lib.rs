#![cfg_attr(not(feature = "std"), no_std)]

use ink_lang as ink;

#[ink::contract]
mod token_helper {
    use ink_env::call::{build_call, Call, ExecutionInput, Selector};
    use ink_env::{DefaultEnvironment, CallFlags};

    static CALLEE_ADDRESS: [u8; 32] = [0xbc, 0x61, 0xc0, 0x26, 0x75, 0x7c, 0xdd, 0xbc, 0xb2, 0x0c, 0x4d, 0x56, 0xb0, 0xf7, 0xad, 0x59, 0xfb, 0x5d, 0x57, 0x5a, 0x9b, 0xdf, 0x8f, 0xb9, 0x8b, 0x40, 0x5d, 0x19, 0x25, 0xb5, 0x34, 0x52];
    static CALLEE_SELECTOR: [u8; 4] = [0x99, 0x0a, 0x10, 0xf2];

    #[ink(storage)]
    pub struct TokenHelper {
    }

    #[derive(Debug, PartialEq, Eq, scale::Encode, scale::Decode)]
    #[cfg_attr(feature = "std", derive(scale_info::TypeInfo))]
    pub enum Error {
        TooLowFeeError,
        TransactionFailed,
    }

    impl TokenHelper {
        #[ink(constructor)]
        pub fn new() -> Self {
            Self {}
        }

        // pay at least 10
        #[ink(message, payable)]
        pub fn transfer(&mut self) -> Result<(), Error> {
            let transferred = self.env().transferred_value();
            if transferred < 10 {
                return Err(Error::TooLowFeeError)
            }
            return Ok(())
        }

        // call counter as contract
        #[ink(message)]
        pub fn call_counter_contract(&mut self) -> Result<(), Error> {
            let result = build_call::<DefaultEnvironment>()
                .call_type(
                    Call::new()
                        .callee(AccountId::from(CALLEE_ADDRESS)) //contract to call
                )
                .exec_input(
                    ExecutionInput::new(Selector::from(CALLEE_SELECTOR))
                )
                .returns::<()>()
                .fire()
                .map_err(|_| Error::TransactionFailed);
            result
        }

        // forward call counter as caller
        #[ink(message, selector = 0x990a10f2)]
        pub fn call_counter_forward(&mut self) -> Result<(), Error> {
            build_call::<DefaultEnvironment>()
            .call_type(
                Call::new()
                    .callee(AccountId::from(CALLEE_ADDRESS)) //contract to call
            )
            .call_flags(
                CallFlags::default()
                    .set_forward_input(true)
                    .set_tail_call(true),
            )
            .fire()
            .unwrap_or_else(|err| {
                panic!(
                    "cross-contract call to failed due to {:?}",
                        err
                )
            });
        unreachable!(
            "the forwarded call will never return since `tail_call` was set"
        );
        }
    }
}
