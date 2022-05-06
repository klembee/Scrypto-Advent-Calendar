use scrypto::prelude::*;

blueprint! {
    struct CoffeeMachine {}

    impl CoffeeMachine {
        pub fn new() -> ComponentAddress {
            Self{}.instantiate().globalize()
        }

        pub fn make_coffee() {
            info!("Brewing coffee !");
        }
    }
}