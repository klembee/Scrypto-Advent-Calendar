use scrypto::prelude::*;

import! {
    r#"
    {
      "package_address": "01ecb27f6b7977c3b588bf275375c7ee43eb340e4f65481d1ee7b3",
      "blueprint_name": "PriceOracle",
      "functions": [
        {
          "name": "new",
          "inputs": [],
          "output": {
            "type": "Custom",
            "name": "ComponentAddress",
            "generics": []
          }
        }
      ],
      "methods": [
        {
          "name": "get_price",
          "mutability": "Immutable",
          "inputs": [
            {
              "type": "Custom",
              "name": "ResourceAddress",
              "generics": []
            },
            {
              "type": "Custom",
              "name": "ResourceAddress",
              "generics": []
            }
          ],
          "output": {
            "type": "Option",
            "value": {
              "type": "Custom",
              "name": "Decimal",
              "generics": []
            }
          }
        },
        {
          "name": "get_usd_address",
          "mutability": "Immutable",
          "inputs": [],
          "output": {
            "type": "Custom",
            "name": "ResourceAddress",
            "generics": []
          }
        },
        {
          "name": "update_price",
          "mutability": "Immutable",
          "inputs": [
            {
              "type": "Custom",
              "name": "ResourceAddress",
              "generics": []
            },
            {
              "type": "Custom",
              "name": "ResourceAddress",
              "generics": []
            },
            {
              "type": "Custom",
              "name": "Decimal",
              "generics": []
            }
          ],
          "output": {
            "type": "Unit"
          }
        }
      ]
    }
    "#
    }

blueprint! {
    struct GiftExchange {
      // Will store the price oracle component
      price_oracle: ComponentAddress,
      // Keep track of the participants
      participants: Vec<ResourceAddress>,
      // Keep track of who should give to who
      who_to_who: HashMap<ResourceAddress, ResourceAddress>,
      // Indicates if the component decided who is going to give to who
      decided: bool,
      // Used to protect methods on this blueprint
      organizer_def: ResourceAddress,
    }

    impl GiftExchange {
        pub fn new(price_oracle_address: ComponentAddress) -> (ComponentAddress, Bucket) {
            // Create the organizer badge.
            // Used to protect the `add_participant` and `prepare_exchange` methods
            let organizer_badge = ResourceBuilder::new_fungible()
                                    .divisibility(DIVISIBILITY_NONE)
                                    .metadata("name", "Organizer Badge")
                                    .initial_supply(1);

            let component = Self {
                price_oracle: price_oracle_address,
                participants: Vec::new(),
                who_to_who: HashMap::new(),
                decided: false,
                organizer_def: organizer_badge.resource_address(),
            }
            .instantiate();

            let auth_rules = AccessRules::new()
              .method("add_participant", rule!(require(organizer_badge.resource_address())))
              .method("prepare_exchange", rule!(require(organizer_badge.resource_address())))
              .method("send_gift", rule!(allow_all));

            // Return the instantiated component and organizer's badge
            (component.add_access_check(auth_rules).globalize(), organizer_badge)
        }

        // As organizer, add a participant to the gift exchange
        pub fn add_participant(&mut self, address: ComponentAddress) {
            assert!(!self.decided, "Component already decided who would give presents to who !");

            // Create the participant's badge, used
            // as identification in `send_gift` method
            let participant_badge =  ResourceBuilder::new_fungible()
                                        .divisibility(DIVISIBILITY_NONE)
                                        .metadata("name", "Participant Badge")
                                        .metadata("account", format!("{}", address))
                                        .initial_supply(1);

            self.participants.push(participant_badge.resource_address());

            // Send the badge to the participant
            borrow_component!(address).call::<()>("deposit", vec![scrypto_encode(&participant_badge)]);
        }

        // Organizer can call this method after adding the participants
        // to decide who should give to who.
        pub fn prepare_exchange(&mut self) {
            assert!(self.participants.len() >= 2, "Add at least two participants first !");
            assert!(self.participants.len() % 2 == 0, "Need to have even number of participants !");
            assert!(!self.decided, "Component already decided who would give presents to who !");

            let amount_to_slice = self.participants.len() / 2;

            for i in 0..amount_to_slice {
                let from = self.participants.get(i).unwrap();
                let to = self.participants.get(i + amount_to_slice).unwrap();
                
                info!("{} is giving to {}", from, to);
                info!("{} is giving to {}", to, from);

                self.who_to_who.insert(*from, *to);
                self.who_to_who.insert(*to, *from);
            }

            // Set to true so that no one can call `make_exchange` and `add_participant` anymore
            self.decided = true;
        }

        // Allow participants to send their gift.
        // They only have to provide their badge. The destination is
        // fetched from the `who_to_who` map.
        pub fn send_gift(&self, gift: Bucket, your_badge: Proof) {
            assert!(self.participants.contains(&your_badge.resource_address()), "Invalid badge");
            assert!(self.decided, "You have to call `make_exchange` first to decide who should give to who.");
            assert!(self.who_to_who.contains_key(&your_badge.resource_address()), "Captain. What should we do? He's not on the list");

            let to_resource = borrow_resource_manager!(*self.who_to_who.get(&your_badge.resource_address()).unwrap());
            your_badge.drop();

            let oracle: PriceOracle = self.price_oracle.into();

            // Make sure the provided gift price is less than 20$
            match oracle.get_price(gift.resource_address(), oracle.get_usd_address()) {
                Some(price) => {
                    if price > dec!("20") {
                        info!("Gift is too expensive for the exchange ! Consider creating a YankeeSwap component instead");
                        std::process::abort();
                    }
                },
                None => {
                    info!("Price of {} unknown", borrow_resource_manager!(gift.resource_address()).metadata().get("name").unwrap());
                    std::process::abort();
                }
            };

            // Fetch the address from the metadata
            let to_address: ComponentAddress = ComponentAddress::from_str(to_resource.metadata().get("account").unwrap()).unwrap();

            // Deposit the gift into the recipient's account
            borrow_component!(to_address).call::<()>("deposit", vec![scrypto_encode(&gift)]);
        }
    }
}
