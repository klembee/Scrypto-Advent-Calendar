use scrypto::prelude::*;
use sbor::*;

// Used to keep track of the user's stake
#[derive(Debug, TypeId, Encode, Decode, Describe, PartialEq, Eq)]
pub struct StakerData {
    // Define when the user staked
    started_at: u64,
    // Defines the amount that the user staked
    amount: Decimal
}

blueprint! {
    struct CoalYieldFarming {
        // Will hold a badge allowing the component to
        // mint Coal tokens and burn staker badges
        minter: Vault,
        
        // Will hold the staked Coal tokens
        stake_pool: Vault,

        stakers: HashMap<ResourceAddress, StakerData>
    }

    impl CoalYieldFarming {
        pub fn new() -> ComponentAddress {
            // Create the minter badge.
            // this badge will be owned by the component and will
            // allow it to mint new coal tokens and burn staker's badge
            let minter = ResourceBuilder::new_fungible()
                                .divisibility(DIVISIBILITY_NONE)
                                .metadata("name", "Coal Minter Badge")
                                .initial_supply(1);

            // Define the coal resource
            let coal = ResourceBuilder::new_fungible()
                        .metadata("name", "Coal")
                        .mintable(rule!(require(minter.resource_address())), LOCKED)
                        .no_initial_supply();

            Self {
                minter: Vault::with_bucket(minter),
                stake_pool: Vault::new(coal),
                stakers: HashMap::new()
            }.instantiate().globalize()
        }

        // Allow caller to stake their coal tokens.
        // This method sends a badge allowing the user to withdraw their funds later
        pub fn stake(&mut self, coal: Bucket) -> Bucket {
            assert!(coal.resource_address() == self.stake_pool.resource_address(), "You can only stake coal !");

            // Create the badge used to withdraw the tokens in the future
            let staker_badge = ResourceBuilder::new_fungible()
                    .divisibility(DIVISIBILITY_NONE)
                    .metadata("name", "Coal Staker Badge")
                    .mintable(rule!(require(self.minter.resource_address())), LOCKED)
                    .burnable(rule!(require(self.minter.resource_address())), LOCKED)
                    .initial_supply(1);

            // Save the stake's data on the component's state
            self.stakers.insert(staker_badge.resource_address(), StakerData { started_at: Runtime::current_epoch(), amount: coal.amount() });
            self.stake_pool.put(coal);

            // Return the staker badge to the caller
            staker_badge
        }

        // Withdraw the staked tokens and rewards received.
        pub fn withdraw(&mut self, staker_badge: Bucket) -> (Bucket, Bucket) {
            let staker_data = match self.stakers.get(&staker_badge.resource_address()) {
                Some(staker) => staker,
                None => {
                    info!("No entries found for this badge !");
                    std::process::abort();
                }
            };

            // Burn the staker badge so that it cannot be used again
            self.minter.authorize(|| {
                staker_badge.burn()
            });

            // Mint coal depending on how long the user staked
            let reward = self.minter.authorize(|| {
                let epochs_staked = Runtime::current_epoch() - staker_data.started_at;
                borrow_resource_manager!(self.stake_pool.resource_address()).mint(10 * epochs_staked)
            });
            
            // Return the staked amount + newly minted tokens
            (self.stake_pool.take(staker_data.amount), reward)
        }

        // Send 1000 Coal tokens to the caller
        // to help you test this component
        pub fn faucet(&self) -> Bucket {
            self.minter.authorize(|| {
                borrow_resource_manager!(self.stake_pool.resource_address()).mint(1000)
            })
        }
    }
}
