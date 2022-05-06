use sbor::*;
use scrypto::prelude::*;

#[derive(TypeId, Decode, Encode, Describe, NonFungibleData)]
pub struct Vacine {
    name: String,
    epoch_taken: u64
}

#[derive(NonFungibleData)]
pub struct Passport {
    #[scrypto(mutable)]
    vacines: Vec<Vacine>
}

blueprint! {
    struct ChristmasParty {
        // Resource Definition of the passport NFT
        passport_nft_def: ResourceAddress
    }

    impl ChristmasParty {
        pub fn new(passport_nft_def: ResourceAddress) -> ComponentAddress {
            Self {
                passport_nft_def: passport_nft_def
            }.instantiate().globalize()
        }

        pub fn enter_party(&self, vacine_passport: Proof) {
            assert!(vacine_passport.resource_address() == self.passport_nft_def, "Wrong passport !");

            let resource_manager = borrow_resource_manager!(self.passport_nft_def);
            let data: Passport = resource_manager.get_non_fungible_data(&vacine_passport.non_fungible::<Passport>().id());

            if data.vacines.len() > 0 {
                info!("Come in !");
            } else {
                info!("You are not authorized to come in.")
            }
        }
    }
}
