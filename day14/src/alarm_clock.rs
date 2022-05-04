use scrypto::prelude::*;
use crate::time_oracle::TimeOracle;

blueprint! {
    struct AlarmClock {
        // Admin badge of the TimeOracle
        // used to protect the "try_trigger" method
        admin_badge: ResourceAddress,
        // Reference to the TimeOracle component
        time_oracle_address: ComponentAddress,
        // Unix time when the AlarmClock should call the other component
        call_at: u64,

        // Component and method to call when the time is greater
        // or equal to the "call_at" variable
        component: ComponentAddress,
        method_to_call: String,

        // Specifies whether the AlarmClock already 
        // called the component or not.
        done: bool
    }

    impl AlarmClock {
        pub fn new(component_address: ComponentAddress, method_to_call: String, call_at: u64) -> (ComponentAddress, Bucket) {
            // Instantiate the TimeOracle component
            let (time_oracle_component, admin_badge): (ComponentAddress, Bucket) = TimeOracle::new(1);
            let time_oracle: TimeOracle = time_oracle_component.into();

            admin_badge.authorize(|| {
                // Set the time to 2021-12-24 00:00:00
                time_oracle.set_current_time(1640322000);
            });

            let component = Self{
                time_oracle_address: time_oracle_component,
                admin_badge: admin_badge.resource_address(),
                call_at: call_at,
                component: component_address.into(),
                method_to_call: method_to_call,
                done: false
            }.instantiate();

            let access_rules = AccessRules::new()
                .method("try_trigger", rule!(require(admin_badge.resource_address())));

            // We will use the same admin badge as the one used
            // in the TimeOracle component for simplicity.
            (component.add_access_check(access_rules).globalize(), admin_badge)
        }

        pub fn try_trigger(&mut self) {
            assert!(!self.done, "Already triggered !");
            let time_oracle: TimeOracle = self.time_oracle_address.into();
            let current_time = time_oracle.get_time();
            if current_time >= self.call_at {
                // Call the method of the specified component
                borrow_component!(self.component).call::<()>(&self.method_to_call, vec![]);
                self.done = true;
            } else {
                info!("Not ready yet !");
            }
        }
    }
}