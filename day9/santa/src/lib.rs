use scrypto::prelude::*;

import! {
    r#"
    {
      "package_address": "01ae4fdaf0894d2a22987009d2eab9f524c77e7c224d471c054c4e",
      "blueprint_name": "PresentList",
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
          "name": "start_new_list",
          "mutability": "Mutable",
          "inputs": [],
          "output": {
            "type": "Custom",
            "name": "Bucket",
            "generics": []
          }
        },
        {
          "name": "add",
          "mutability": "Mutable",
          "inputs": [
            {
              "type": "String"
            },
            {
              "type": "Custom",
              "name": "Proof",
              "generics": []
            }
          ],
          "output": {
            "type": "Unit"
          }
        },
        {
          "name": "remove",
          "mutability": "Mutable",
          "inputs": [
            {
              "type": "String"
            },
            {
              "type": "Custom",
              "name": "Proof",
              "generics": []
            }
          ],
          "output": {
            "type": "Unit"
          }
        },
        {
          "name": "display_list",
          "mutability": "Immutable",
          "inputs": [
            {
              "type": "Custom",
              "name": "Proof",
              "generics": []
            }
          ],
          "output": {
            "type": "Unit"
          }
        },
        {
          "name": "get_lists",
          "mutability": "Immutable",
          "inputs": [],
          "output": {
            "type": "HashMap",
            "key": {
              "type": "Custom",
              "name": "ResourceAddress",
              "generics": []
            },
            "value": {
              "type": "Vec",
              "element": {
                "type": "String"
              }
            }
          }
        }
      ]
    }
    "#
    }

blueprint! {
    struct Santa {
        present_list: ComponentAddress,
        presents: HashMap<ResourceAddress, Vec<Vault>>
    }

    impl Santa {
        pub fn new(present_list_component: ComponentAddress) -> ComponentAddress {
            Self {
                present_list: present_list_component,
                presents: HashMap::new()
            }.instantiate().globalize()
        }

        // Create tokens for every present in the list
        // and associate them with the recipient's badge
        pub fn prepare_gifts(&mut self) {
            let present_list_component: PresentList = self.present_list.into();
            let lists: HashMap<ResourceAddress, Vec<String>> = present_list_component.get_lists();
            for (badge_address, gifts) in lists {
                // Retrieve the list of vaults for that particular badge's address.
                // If not present, create entry with empty vec
                let vaults = self.presents.entry(badge_address).or_insert(Vec::new());

                // Create the tokens that will act as gifts
                for gift in gifts {
                    let resource = ResourceBuilder::new_fungible()
                                    .divisibility(DIVISIBILITY_NONE)
                                    .metadata("name", format!("{}", gift))
                                    .initial_supply(1);
                    vaults.push(Vault::with_bucket(resource));
                }
            }
        }

        // Allow people to withdraw their gifts.
        // They use the same badge as the one for their present list
        pub fn withdraw_gifts(&mut self, badge: Proof) -> Vec<Bucket> {
            let mut buckets: Vec<Bucket> = Vec::new();
            match self.presents.get_mut(&badge.resource_address()) {
                Some(gifts) => {
                    for gift in gifts {
                        buckets.push(gift.take_all())
                    }
                },
                None => {
                    info!("Badge is invalid !");
                    std::process::abort();
                }
            };

            buckets
        }
    }
}
