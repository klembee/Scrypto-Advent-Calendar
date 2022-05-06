use sbor::*;
use scrypto::prelude::*;

#[derive(TypeId, Decode, Encode, Describe, NonFungibleData)]
pub struct Vaccine {
    name: String,
    epoch_taken: u64
}

#[derive(NonFungibleData)]
pub struct Passport {
    #[scrypto(mutable)]
    vaccines: Vec<Vaccine>
}

blueprint! {
    struct VaccinePassport {
        admin_def: ResourceAddress,
        // Token that the this component use to mint and update
        // vaccine passports NFTs
        passport_manager_badge: Vault,
        // Resource definition of the NFTs
        passport_def: ResourceAddress,
        // Number of passports minted
        nb_passports: u64
    }

    impl VaccinePassport {
        pub fn new() -> (ComponentAddress, Bucket) {
            let passport_manager_badge: Bucket = ResourceBuilder::new_fungible()
                                                    .divisibility(DIVISIBILITY_NONE)
                                                    .metadata("name", "Vaccine Passport Manager")
                                                    .initial_supply(1);

            // Define the admin badge
            let admin_badge: Bucket = ResourceBuilder::new_fungible()
                .divisibility(DIVISIBILITY_NONE)
                .metadata("name", "VaccinePassport Admin Badge")
                .initial_supply(1);

            // Define the VaccinePassport NFT.
            // Specify that the admin_badge can mint, burn and update the metadata of the tokens
            let passport: ResourceAddress = ResourceBuilder::new_non_fungible()
                            .metadata("name", "Vaccine Passport")
                            .mintable(rule!(require(passport_manager_badge.resource_address())), LOCKED)
                            .burnable(rule!(require(passport_manager_badge.resource_address())), LOCKED)
                            .updateable_non_fungible_data(rule!(require(passport_manager_badge.resource_address())), LOCKED)
                            .no_initial_supply();

            let component = Self {
                admin_def: admin_badge.resource_address(),
                passport_manager_badge: Vault::with_bucket(passport_manager_badge),
                passport_def: passport,
                nb_passports: 0
            }.instantiate();

            (component.globalize(), admin_badge)
        }

        // Allow people to create a new empty vaccine passport
        pub fn get_new_passport(&mut self) -> Bucket {
            // Mint a new NFT with empty array of vaccines
            let passport = self.passport_manager_badge.authorize(|| {
                borrow_resource_manager!(self.passport_def)
                    .mint_non_fungible(&NonFungibleId::from_u64(self.nb_passports), Passport{vaccines: Vec::new()})
            });
            
            self.nb_passports += 1;

            passport
        }

        // Update the provided passport NFT with the vaccine data
        pub fn get_vaccine(&self, passport: Proof) {
            // Make sure the passed bucket is valid
            assert!(passport.resource_address() == self.passport_def, "Wrong passport. Create one with `get_new_passport`");

            // Add the vaccine data to the passport
            let mut data: Passport = passport.non_fungible::<Passport>().data();

            data.vaccines.push(Vaccine{
                name: "ScryptoZeneca".to_owned(),
                epoch_taken: Runtime::current_epoch()
            });

            // Update the NFT data with the new array of vaccines
            self.passport_manager_badge.authorize(|| {
                borrow_resource_manager!(self.passport_def)
                    .update_non_fungible_data(&passport.non_fungible::<Passport>().id(), data)
            });
        }

        // Display the information on the taken vaccines
        pub fn display_vaccine_data(&self, passport: Proof) {
            // Make sure the passed bucket is valid
            assert!(passport.resource_address() == self.passport_def, "Wrong passport. Create one with `get_new_passport`");

            let data: Passport = passport.non_fungible::<Passport>().data();
            passport.drop();

            info!("Vaccines you have taken:");
            for vaccine in data.vaccines {
                info!("Vaccine {} taken on epoch {}", vaccine.name, vaccine.epoch_taken);
            }
        }
    }
}
