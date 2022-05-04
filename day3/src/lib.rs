use scrypto::prelude::*;

blueprint! {
    struct PresentDistributor {
        // Used to store the present tokens
        present_vault: Vault,
        // Used to store the coal tokens
        coal_vault: Vault,
        // Store the good kids addresses
        good_kids: Vec<ComponentAddress>,
        // Store the naughty kids addresses
        naughty_kids: Vec<ComponentAddress>
    }

    impl PresentDistributor {
        pub fn new() -> ComponentAddress {
            // Generate 1000 present tokens
            // let presents = ResourceBuilder::new_fungible()
            //             .divisibility(DIVISIBILITY_MAXIMUM)
            //             .metadata("name", "Present")
            //             .initial_supply(1000);

            info!("Before minting!");

            // // Generate 1000 coal tokens
            // let coal = ResourceBuilder::new_fungible()
            //             .divisibility(DIVISIBILITY_MAXIMUM)
            //             .metadata("name", "Coal")
            //             .initial_supply(1000);

            let coal: Bucket = ResourceBuilder::new_fungible()
                        .metadata("name", "HelloToken")
                        .metadata("symbol", "HT")
                        .initial_supply(1000);

            info!("After minting!");

            // Store the tokens in component's vaults
            Self {
                present_vault: Vault::new(RADIX_TOKEN),
                coal_vault: Vault::with_bucket(coal),
                good_kids: Vec::new(),
                naughty_kids: Vec::new()
            }
            .instantiate()
            .globalize()
        }

        // Add a kid to the list of good or naughty kids
        pub fn add_kid(&mut self, receiver: ComponentAddress, is_naughty: bool) {
            // Make sure the address was not already added
            assert!(!self.good_kids.contains(&receiver) && !self.naughty_kids.contains(&receiver), "Address already added to the list");

            if is_naughty {
                // Push the new kid to the naughty list
                self.naughty_kids.push(receiver);
            } else {
                // Push the new kid to the good list
                self.good_kids.push(receiver);
            }
        }

        // Distribute the presents and the coal to the kids
        pub fn distribute_gifts(&mut self) {
            // Make sure kids were added
            assert!(self.naughty_kids.len() > 0 || self.good_kids.len() > 0, "You need to add kids to the list !");

            // Distribute the presents
            for kid in self.good_kids.iter() {
                borrow_component!(*kid).call::<()>("deposit", vec![scrypto_encode(&self.present_vault.take(1))]);
            }

            // Distribute coal
            for kid in self.naughty_kids.iter() {
                borrow_component!(*kid).call::<()>("deposit", vec![scrypto_encode(&self.present_vault.take(1))]);
            }
        }
    }
}
