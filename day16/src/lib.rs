use scrypto::prelude::*;
use sbor::*;
use std::fmt;

// Define the attributes of the NFT
#[derive(TypeId, Encode, Decode, Describe, Debug)]
pub enum Head {
    None,
    Cap,
    ChristmasHat,
    Crown
}

#[derive(TypeId, Encode, Decode, Describe, Debug)]
pub enum Clothing {
    WhiteShirt,
    Hoodie,
    WinterCoat,
    SantaCoat
}

#[derive(TypeId, Encode, Decode, Describe, Debug)]
pub enum Mouth {
    Smile,
    TongueOut,
    Sad,
    GoldenTeeths,
    DiamondTeeths
}

#[derive(TypeId, Encode, Decode, Describe, Debug)]
pub enum Nose {
    Regular,
    Clown,
    Runny
}

#[derive(TypeId, Encode, Decode, Describe, Debug)]
pub enum Eyewear {
    None,
    ReadingGlasses,
    SunGlasses,
    EyePatch
}

#[derive(TypeId, Encode, Decode, Describe, Debug)]
pub enum Background {
    White,
    Blue,
    Gold,
    Diamond,
    Space
}

#[derive(NonFungibleData)]
pub struct DegenerateElf {
    head: Head,
    clothing: Clothing,
    mouth: Mouth,
    nose: Nose,
    eye_wear: Eyewear,
    background: Background,
    color: usize
}

impl fmt::Display for DegenerateElf {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Head: {:?}\nClothing: {:?}\nMouth: {:?}\nNose: {:?}\nEyes: {:?}\nBackground: {:?}\nColor: {:?}\n", self.head, self.clothing, self.mouth, self.nose, self.eye_wear, self.background, self.color)
    }
}

blueprint! {
    struct DegenerateElves {
        // Store the badge that allows
        // the component to mint new elves
        mint_badge: Vault,
        // Resource definition of the elf NFT
        elf_def: ResourceAddress,
        // Vault to store the payments
        payment_vault: Vault,
        // Cost of minting one NFT
        mint_price: Decimal,
        // Maximum supply of the nfts
        max_supply: u64,
        // Keep track of the number of elves minted
        nb_minted: u64,
    }

    impl DegenerateElves {
        // Instantiate a new DegenerateElves component with a
        // minting cost and a max supply
        pub fn new(mint_price: Decimal, max_supply: u64) -> ComponentAddress {
            // Create the elf minting badge
            let mint_badge = ResourceBuilder::new_fungible()
                                .divisibility(DIVISIBILITY_NONE)
                                .metadata("name", "DegenerateElf Minting Badge")
                                .initial_supply(1);

            // Create the elf NFT definition
            let elf_def = ResourceBuilder::new_non_fungible()
                            .metadata("name", "Degenerate Elves")
                            .mintable(rule!(require(mint_badge.resource_address())), LOCKED)
                            .no_initial_supply();

            Self {
                mint_badge: Vault::with_bucket(mint_badge),
                elf_def: elf_def,
                payment_vault: Vault::new(RADIX_TOKEN),
                mint_price: mint_price,
                max_supply: max_supply,
                nb_minted: 0,
            }.instantiate().globalize()
        }

        // Mint a new elf NFT. Requires a payment
        // Return the NFT and the change (if payment > mint_cost)
        pub fn mint(&mut self, mut payment: Bucket) -> (Bucket, Bucket) {
            assert!(payment.amount() >= self.mint_price, "Minting costs {}", self.mint_price);
            assert!(self.nb_minted <= self.max_supply, "Max supply reached !");

            self.payment_vault.put(payment.take(self.mint_price));

            // Mint a random Elf
            let elf_attributes = DegenerateElf{
                head: self.random_head(),
                clothing: self.random_clothing(),
                eye_wear: self.random_eye(),
                background: self.random_background(),
                nose: self.random_nose(),
                mouth: self.random_mouth(),
                color: self.random_color()
            };

            let elf = self.mint_badge.authorize(|| {
                borrow_resource_manager!(self.elf_def).mint_non_fungible(&NonFungibleId::from_u64(self.nb_minted), elf_attributes)
            });

            self.nb_minted += 1;

            // Return the change and NFT back
            (payment, elf)
        }

        // Used to display information about your elf NFT
        pub fn display_info(&self, elves: Proof) {
            assert!(elves.resource_address() == self.elf_def, "NFT definition not matching");

            for non_fungible in elves.non_fungibles() {
                let data: DegenerateElf = non_fungible.data();
                info!("========");
                info!("{}", data)
            }
            
            elves.drop();
        }

        // The following methods are used to randomly generate an elf

        fn random_head(&mut self) -> Head {
            match self.random_number(0, 3) {
                0 => Head::Cap,
                1 => Head::ChristmasHat,
                2 => Head::Crown,
                3 => Head::None,
                _ => panic!()
            }
        }

        fn random_clothing(&mut self) -> Clothing {
            match self.random_number(0, 3) {
                0 => Clothing::Hoodie,
                1 => Clothing::SantaCoat,
                2 => Clothing::WhiteShirt,
                3 => Clothing::WinterCoat,
                _ => panic!()
            }
        }

        fn random_mouth(&mut self) -> Mouth {
            match self.random_number(0, 4) {
                0 => Mouth::Smile,
                1 => Mouth::Sad,
                2 => Mouth::TongueOut,
                3 => Mouth::GoldenTeeths,
                4 => Mouth::DiamondTeeths,
                _ => panic!()
            }
        }

        fn random_nose(&mut self) -> Nose {
            match self.random_number(0, 2) {
                0 => Nose::Regular,
                1 => Nose::Clown,
                2 => Nose::Runny,
                _ => panic!()
            }
        }

        fn random_eye(&mut self) -> Eyewear {
            match self.random_number(0, 3) {
                0 => Eyewear::None,
                1 => Eyewear::EyePatch,
                2 => Eyewear::ReadingGlasses,
                3 => Eyewear::SunGlasses,
                _ => panic!()
            }
        }

        fn random_background(&mut self) -> Background {
            match self.random_number(0, 4) {
                0 => Background::White,
                1 => Background::Blue,
                2 => Background::Gold,
                3 => Background::Space,
                4 => Background::Diamond,
                _ => panic!()
            }
        }

        fn random_color(&mut self) -> usize {
            self.random_number(0, 16777215)
        }

        // Generate a random number
        // WARNING: DON'T USE THIS IN PRODUCTION !
        fn random_number(&mut self, min: u64, max: u64) -> usize {
            let mut random_number = Runtime::generate_uuid() as u64;
            random_number = (random_number / u64::MAX) * (max - min) + min;
            random_number as usize
        }
    }
}