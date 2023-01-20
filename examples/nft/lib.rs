#![cfg_attr(not(feature = "std"), no_std)]
#![feature(min_specialization)]
        
#[openbrush::contract]
pub mod nft {
    
    // imports from openbrush
	use openbrush::contracts::psp34::extensions::mintable::*;
	use openbrush::traits::Storage;

    #[ink(storage)]
    #[derive(Default, Storage)]
    pub struct Nft {
    	#[storage_field]
		psp34: psp34::Data,
    }
    
    // Section contains default implementation without any modifications
	impl PSP34 for Nft {}
    impl PSP34Mintable for Nft {}
     
    impl Nft {
        #[ink(constructor)]
        pub fn new() -> Self {
            let mut _instance = Self::default();
			_instance
        }
    }
}