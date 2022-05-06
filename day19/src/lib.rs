use scrypto::prelude::*;

#[derive(NonFungibleData)]
struct SubscriptionData {
    amount: Decimal,
    recurrence: u64,
    last_payment_at: u64,
    destination: ComponentAddress
}

blueprint! {
    struct RecurrentPayment {
        admin_def: ResourceAddress,
        token_authority_badge: Vault,
        user_payments: Vault,
        payment_token_def: ResourceAddress,
        user_nft_def: ResourceAddress,
        nb_users: u64
    }

    impl RecurrentPayment {
        pub fn new() -> (ComponentAddress, Bucket) {
            // Admin badge used to protect methods
            let admin_badge = ResourceBuilder::new_fungible()
                                .divisibility(DIVISIBILITY_NONE)
                                .metadata("name", "RecurrentPayment Admin")
                                .initial_supply(1);

            // Badge used to manage the payment tokens and
            // the user NFT
            let token_authority_badge = ResourceBuilder::new_fungible()
                                    .divisibility(DIVISIBILITY_NONE)
                                    .initial_supply(1);


            // Create a payment token that this component can take from
            // the accounts
            let payment_token_def = ResourceBuilder::new_fungible()
                                        .metadata("name", "Payment Token")
                                        .mintable(rule!(require(token_authority_badge.resource_address())), LOCKED)
                                        .no_initial_supply();

            // NFT definition that will represent individual subscriptions                     
            let user_nft_def = ResourceBuilder::new_non_fungible()
                                .metadata("name", "RecurrentPayment user")
                                .mintable(rule!(require(token_authority_badge.resource_address())), LOCKED)
                                .burnable(rule!(require(token_authority_badge.resource_address())), LOCKED)
                                .updateable_non_fungible_data(rule!(require(token_authority_badge.resource_address())), LOCKED)
                                .no_initial_supply();

            let component = Self {
                admin_def: admin_badge.resource_address(),
                payment_token_def: payment_token_def,
                user_payments: Vault::new(RADIX_TOKEN),
                token_authority_badge: Vault::with_bucket(token_authority_badge),
                user_nft_def: user_nft_def,
                nb_users: 0
            }
            .instantiate();

            let access_rules = AccessRules::new()
                .method("take_payments", rule!(require(admin_badge.resource_address())))
                .default(rule!(allow_all));

            (component.add_access_check(access_rules).globalize(), admin_badge)
        }

        // Allow users to refill their Payment tokens
        pub fn buy_payment_tokens(&self, payment: Bucket) -> Bucket {
            assert!(payment.resource_address() == RADIX_TOKEN, "Payment must be XRD tokens");
            let amount = payment.amount();
            self.user_payments.put(payment);

            // Mint new payment tokens and return them to the user
            self.token_authority_badge.authorize(|| {
                borrow_resource_manager!(self.payment_token_def).mint(amount)
            })
        }

        // Allow a user to swap their Payment tokens to XRD
        pub fn sell_payment_tokens(&self, payment: Bucket) -> Bucket {
            assert!(payment.resource_address() == self.payment_token_def, "Payment must be Payment tokens");
            let amount = payment.amount();

            // Burn the Payment tokens
            self.token_authority_badge.authorize(|| {
                payment.burn();
            });

            // Return the same amount of XRD
            self.user_payments.take(amount)
        }

        pub fn take_payments(&self) {
            for n in 0..self.nb_users {
                let mut data: SubscriptionData = borrow_resource_manager!(self.user_nft_def).get_non_fungible_data(&NonFungibleId::from_u64(n));
                // Check if it's time to pay
                if data.last_payment_at + data.recurrence >= Runtime::current_epoch() {
                    // Take the payment (Payment tokens) from the user.
                    // Please note that it is not yet possible to recall tokens with Scrypto (18/12/2021)
                    // This is just a proof of concept.
                    let payment_tokens = user.recall(self.payment_token_def, data.amount);
                    let payment_xrd = self.user_payments.take(data.amount);

                    // Send the XRD tokens to the service 
                    borrow_component!(data.destination).call::<()>("deposit", vec![scrypto_encode(&payment_xrd)]);

                    // Burn the payment tokens and
                    // update the NFT last payment epoch
                    self.token_authority_badge.authorize(|| {
                        payment_tokens.burn();

                        data.last_payment_at = Runtime::current_epoch();
                        borrow_resource_manager!(self.user_nft_def).update_non_fungible_data(&NonFungibleId::from_u64(n), data);
                    })
                }
            }
        }

        // Allow services to create subscriptions on behalf of their users
        pub fn setup_subscription(&mut self, amount: Decimal, recurrence: u64, destination: ComponentAddress) -> Bucket {
            self.nb_users += 1;
            
            // Mint a new user NFT and return it to the caller
            self.token_authority_badge.authorize(|| {
                borrow_resource_manager!(self.user_nft_def).mint_non_fungible(&NonFungibleId::from_u64(self.nb_users), SubscriptionData{
                    amount: amount, 
                    recurrence: recurrence, 
                    last_payment_at: 0, 
                    destination: destination
                })
            })
        }
    }
}
