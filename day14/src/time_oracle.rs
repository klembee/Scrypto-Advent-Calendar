use scrypto::prelude::*;

blueprint! {
    struct TimeOracle {
        // Badge used to update the time
        admin_badge: ResourceAddress,
        second_since_unix: u64
    }

    impl TimeOracle {
        pub fn new(nb_admins: u32) -> (ComponentAddress, Bucket) {
            // Create the admin badges
            let admin_badges = ResourceBuilder::new_fungible()
                                .divisibility(DIVISIBILITY_NONE)
                                .metadata("name", "UTCTimeOracle Admin Badge")
                                .initial_supply(nb_admins);

            let component = Self {
                admin_badge: admin_badges.resource_address(),
                second_since_unix: 0
            }
            .instantiate();

            let access_rules = AccessRules::new()
                .method("set_current_time", rule!(require(admin_badges.resource_address())))
                .default(rule!(allow_all));

            // Return the component and the admin badges
            (component.add_access_check(access_rules).globalize(), admin_badges)
        }

        pub fn set_current_time(&mut self, second_since_unix: u64) {
            self.second_since_unix = second_since_unix;
        }

        pub fn get_time(&self) -> u64 {
            // Return the datetime
            self.second_since_unix
        }
    }
}
