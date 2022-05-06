use scrypto::prelude::*;
use crate::house::House;

blueprint! {
    struct Santa {
        // List of House components
        houses: Vec<ComponentAddress>,
        // Badges required to access the houses
        keys: Vec<Vault>,
        // Gift vault used to put gifts under the houses trees
        gifts: Vault
    }

    impl Santa {
        pub fn new() -> ComponentAddress {
            // Create the tokens that will represent the gifts
            let gifts = ResourceBuilder::new_fungible()
                            .divisibility(DIVISIBILITY_NONE)
                            .metadata("name", "Gift")
                            .initial_supply(8000);

            // Instantiate the 10 house components
            let mut houses: Vec<ComponentAddress> = Vec::new();
            let mut keys: Vec<Vault> = Vec::new();

            for _ in 0..10 {
                let (component, key) = House::new(gifts.resource_address());
                houses.push(component);
                keys.push(Vault::with_bucket(key));
            }

            Self {
                houses: houses,
                keys: keys,
                gifts: Vault::with_bucket(gifts)
            }
            .instantiate().globalize()
        }

        // Take the milk and cookies from the house at the specified index.
        // Then put a gift under the house's Christmas tree
        pub fn go_into_house(&mut self, house_index: usize) -> (Bucket, Bucket) {

            let (cookies, milk) = match self.houses.get(house_index) {
                Some(house_address) => {
                    let house: House = (*house_address).into();

                    // Get the key and use the authorize method.
                    // The authorize method takes a bucket from the key vault,
                    // create a proof of its content and make it available in the auth zone.
                    self.keys.get(house_index).unwrap().authorize(|| {
                        // Put gift under the tree
                        house.give_gift(self.gifts.take(1));

                        // Take and return the cookies and milk
                        house.get_milk_and_cookie()
                    })
                },
                None => {
                    // House not found with the provided house_index
                    info!("Invalid house index !");
                    std::process::abort();
                }
            };

            // Return the cookies and milk to the caller
            (cookies, milk)
        }
    }
}
