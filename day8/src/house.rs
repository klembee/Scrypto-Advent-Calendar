use scrypto::prelude::*;

blueprint! {
    struct House {
        key: ResourceAddress,
        milk: Vault,
        cookies: Vault,
        gifts: Vault
    }

    impl House {
        pub fn new(gift_resource: ResourceAddress) -> (ComponentAddress, Bucket) {
            // Create a key badge, allowing people to call methods on this component
            let key = ResourceBuilder::new_fungible()
                        .divisibility(DIVISIBILITY_NONE)
                        .metadata("name", "House Key")
                        .initial_supply(1);

            // Create the milk and cookie tokens
            let milk = ResourceBuilder::new_fungible()
                        .divisibility(DIVISIBILITY_NONE)
                        .metadata("name", "Milk")
                        .initial_supply(1);
            let cookies = ResourceBuilder::new_fungible()
                            .divisibility(DIVISIBILITY_NONE)
                            .metadata("name", "Cookie")
                            .initial_supply(3);

            let component = Self {
                // Store the resource definition of
                // the key badge to securise the methods
                key: key.resource_address(),
                milk: Vault::with_bucket(milk),
                cookies: Vault::with_bucket(cookies),
                gifts: Vault::new(gift_resource) // Instantiate empty gift vault
            }.instantiate();

            // Make sure only people presenting the `key` badge are
            // able to call the two methods
            let auth = AccessRules::new()
                .method("get_milk_and_cookie", auth!(require(key.resource_address())))
                .method("give_gift", auth!(require(key.resource_address())));

            (component.add_access_check(auth).globalize(), key)
        }

        pub fn get_milk_and_cookie(&mut self) -> (Bucket, Bucket) {
            // Give the cookies and milk
            (self.cookies.take_all(), self.milk.take_all())
        }

        pub fn give_gift(&mut self, gift: Bucket) {
            // Insert the gift in the component's vault
            self.gifts.put(gift);
        }
    }
}