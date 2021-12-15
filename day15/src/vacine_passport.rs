use sbor::*;
use scrypto::prelude::*;

#[derive(TypeId, Decode, Encode, Describe, NftData)]
pub struct Vacine {
    name: String,
    epoch_taken: u64
}

#[derive(NftData)]
pub struct Passport {
    #[scrypto(mutable)]
    vacines: Vec<Vacine>
}

blueprint! {
    struct VacinePassport {
        // Token that the this component use to mint and update
        // vacine passports NFTs
        admin_badge: Vault,
        // Resource definition of the NFTs
        passport_def: ResourceDef,
        // Number of passports minted
        nb_passports: u128
    }

    impl VacinePassport {
        pub fn new() -> Component {
            // Define the admin badge
            let admin_badge: Bucket = ResourceBuilder::new_fungible(DIVISIBILITY_NONE)
                                        .metadata("name", "VacinePassport Admin Badge")
                                        .initial_supply_fungible(1);

            // Define the VacinePassport NFT.
            // Specify that the admin_badge can mint, burn and update the metadata
            // of the tokens
            let passport: ResourceDef = ResourceBuilder::new_non_fungible()
                            .metadata("name", "Vacine Passport")
                            .flags(MINTABLE | BURNABLE | INDIVIDUAL_METADATA_MUTABLE)
                            .badge(
                                admin_badge.resource_def(),
                                MAY_MINT | MAY_BURN | MAY_CHANGE_INDIVIDUAL_METADATA
                            )
                            .no_initial_supply();

            Self {
                admin_badge: Vault::with_bucket(admin_badge),
                passport_def: passport,
                nb_passports: 0
            }.instantiate()
        }

        // Allow people to create a new empty vacine passport
        pub fn get_new_passport(&mut self) -> Bucket {
            // Mint a new NFT with empty array of vacines
            let passport = self.admin_badge.authorize(|badge| {
                self.passport_def.mint_nft(self.nb_passports, Passport{vacines: Vec::new()}, badge)
            });
            
            self.nb_passports += 1;

            passport
        }

        // Update the provided passport NFT with the vacine data
        pub fn get_vacine(&self, passport: Bucket) -> Bucket {
            // Make sure the passed bucket is valid
            assert!(passport.amount() > Decimal::zero(), "Missing passport");
            assert!(passport.resource_def() == self.passport_def, "Wrong passport. Create one with `get_new_passport`");

            // Add the vacine data to the passport
            let mut data: Passport = passport.get_nft_data(passport.get_nft_id());
            data.vacines.push(Vacine{
                name: "ScryptoZeneca".to_owned(),
                epoch_taken: Context::current_epoch()
            });

            // Update the NFT data with the new array of vacines
            self.admin_badge.authorize(|badge| {
                passport.update_nft_data(passport.get_nft_id(), data, badge);
            });

            // Return the passport back to the caller
            passport
        }

        // Display the information on the taken vacines
        pub fn display_vacine_data(&self, passport: BucketRef) {
            // Make sure the passed bucket is valid
            assert!(passport.amount() > Decimal::zero(), "Missing passport");
            assert!(passport.resource_def() == self.passport_def, "Wrong passport. Create one with `get_new_passport`");

            let data: Passport = self.passport_def.get_nft_data(passport.get_nft_id());
            passport.drop();

            info!("Vacines you have taken:");
            for vacine in data.vacines {
                info!("Vacine {} taken on epoch {}", vacine.name, vacine.epoch_taken);
            }
        }
    }
}
