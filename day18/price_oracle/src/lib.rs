use scrypto::prelude::*;

// PriceOrable component. Taken from https://github.com/radixdlt/radixdlt-scrypto examples.
// I removed the admin badge for simplicity.
blueprint! {
    struct PriceOracle {
        /// Last price of each resource pair
        prices: LazyMap<(ResourceAddress, ResourceAddress), Decimal>,
        usd: Vault
    }

    impl PriceOracle {
        /// Creates a PriceOracle component, along with admin badges.
        pub fn new() -> ComponentAddress {
            // Create usd tokens
            let usd = ResourceBuilder::new_fungible()
                        .metadata("name", "USD")
                        .initial_supply(100000);

            Self {
                prices: LazyMap::new(),
                usd: Vault::with_bucket(usd)
            }
            .instantiate().globalize()
        }

        /// Returns the current price of a resource pair BASE/QUOTE.
        pub fn get_price(&self, base: ResourceAddress, quote: ResourceAddress) -> Option<Decimal> {
            self.prices.get(&(base, quote))
        }

        // Return the address of USD token
        pub fn get_usd_address(&self) -> ResourceAddress {
            self.usd.resource_address()
        }

        /// Updates the price of a resource pair BASE/QUOTE and its inverse.
        pub fn update_price(&self, base: ResourceAddress, quote: ResourceAddress, price: Decimal) {
            self.prices.insert((base, quote), price);
            self.prices.insert((quote, base), Decimal::one() / price);
        }
    }
}
