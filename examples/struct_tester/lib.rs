#![cfg_attr(not(feature = "std"), no_std)]

use ink_lang as ink;

#[ink::contract]
mod struct_tester {
    use ink_prelude::string::String;
    use ink_storage::traits::{SpreadAllocate, PackedLayout, SpreadLayout};
    use ink_storage::Mapping;

    #[derive(PackedLayout, SpreadLayout, scale::Encode, scale::Decode, Clone)]
    #[cfg_attr(feature = "std", derive(::scale_info::TypeInfo))]
    pub struct DataStruct {
        byte_vaulue: u8,
        string_value: String,
        int_value: u32,
        bool_value: bool,
        address_value: AccountId
    }


    #[ink(storage)]
    #[derive(SpreadAllocate)]
    pub struct StructTester {
        values: Mapping<AccountId, DataStruct>,
        caller: AccountId,
    }

    use ink_lang::utils::initialize_contract;
    impl StructTester {
        /// Constructor that initializes the `bool` value to the given `init_value`.
        #[ink(constructor)]
        pub fn new(init_value: DataStruct) -> Self {
            initialize_contract(|contract: &mut Self| {
                let caller = Self::env().caller();
                contract.values.insert(&caller, &init_value);
                contract.caller = caller;
            })
            // Self { value: init_value }
        }

        #[ink(message)]
        pub fn get_value(&self) -> DataStruct {
            let value = self.values.get(self.caller).clone();
            return match value {
                Some(x) => x,
                None => DataStruct{
                    byte_vaulue: 0,
                    string_value: String::from(""),
                    int_value: 0,
                    bool_value: false,
                    address_value: self.caller
                },
            }
        }
    }

}
