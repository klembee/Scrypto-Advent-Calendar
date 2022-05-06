use scrypto::prelude::*;

// Introduction to badges and how to switch users in resim
blueprint! {
    struct House {
        santa_badge: ResourceAddress,
        owner_badge: ResourceAddress
    }

    impl House {
        pub fn new() -> (ComponentAddress, Vec<Bucket>) {
            // Create a new santa badge
            // new_badge_fixed returns a bucket containing the
            // generated badge.
            let santa_badge = ResourceBuilder::new_fungible()
                .divisibility(DIVISIBILITY_NONE)
                .metadata("name", "Santa's Badge")
                .initial_supply(1);
            
            // Create a new owner badge
            let owner_badge = ResourceBuilder::new_fungible()
                .divisibility(DIVISIBILITY_NONE)
                .metadata("name", "Owner's Badge")
                .initial_supply(1);

            // Store both badge's resource_def in the component's state.
            // We will need them for authentification
            let component = Self {
                santa_badge: santa_badge.resource_address(),
                owner_badge: owner_badge.resource_address()
            }.instantiate();

            // Return back the component and both badges
            (component.globalize(), vec![santa_badge, owner_badge])
        }

        pub fn enter(&self, key: Proof) {
            // === Note on Proof
            // In this method, we are accepting a proof to identify the user.
            // Proofs are like Buckets whose ownership are not passed to the component.
            // This component can't store the content of the provided proof in its vaults or send it to someone else.
            if key.resource_address() == self.owner_badge {
                info!("Welcome home !");
            } else if key.resource_address() == self.santa_badge {
                info!("Hello ! Please take some cookies and milk !");
            }
        }
    }
}
