use scrypto::prelude::*;

#[derive(NonFungibleData)]
pub struct PresentList {
    #[scrypto(mutable)]
    presents: Vec<String>
}

blueprint! {
    struct PresentListWithNFT {
        list_minter: Vault,
        list_def: ResourceAddress
    }

    impl PresentListWithNFT {
        pub fn new() -> ComponentAddress {
            // Create a badge that will allow the component to
            // mint new list NFTs
            let list_minter: Bucket = ResourceBuilder::new_fungible()
                                        .divisibility(DIVISIBILITY_NONE)
                                        .initial_supply(1);

            // Create the definition of the NFT
            let list_resource_def: ResourceAddress = ResourceBuilder::new_non_fungible()
                                                    .metadata("name", "Christmas List")
                                                    .mintable(rule!(require(list_minter.resource_address())), LOCKED)
                                                    .updateable_non_fungible_data(rule!(require(list_minter.resource_address())), LOCKED)
                                                    .no_initial_supply();

            // Store all required information info the component's state
            Self {
                list_minter: Vault::with_bucket(list_minter),
                list_def: list_resource_def
            }
            .instantiate().globalize()
        }
        
        // Allow the user to start a new christmas list.
        // It generates a list NFT that will contain the list items
        pub fn start_new_list(&mut self) -> Bucket {
            // Mint a new christmas list badge and return it to the caller
            self.list_minter.authorize(|| {
                borrow_resource_manager!(self.list_def)
                    .mint_non_fungible(&NonFungibleId::random(), PresentList { presents: Vec::new() })
            })
        }

        // Add a new present to the list
        pub fn add(&self, present_name: String, list: Proof) {
            assert!(list.resource_address() == self.list_def, "Wrong badge provided");
            let list_nft = list.non_fungible::<PresentList>();
            let mut list_data: PresentList = list_nft.data();

            // Make sure that the present is not already inside the user's list
            assert!(!list_data.presents.contains(&present_name), "Present already on the list !");

            list_data.presents.push(present_name);

            // Update the list with the newly added present
            self.list_minter.authorize(|| {
                borrow_resource_manager!(self.list_def)
                    .update_non_fungible_data(&list_nft.id(), list_data)
            });

            info!("Present added to your list !");
        }

        // Remove a present in the list
        pub fn remove(&self, present_name: String, list: Proof) {
            assert!(list.resource_address() == self.list_def, "Wrong badge provided");
            let list_nft = list.non_fungible::<PresentList>();
            let mut list_data: PresentList = list_nft.data();

            // Make sure that the present is not already inside the user's list
            assert!(list_data.presents.contains(&present_name), "Present not on the list !");
        
            // Find the index of the present to remove
            let index = list_data.presents.iter().position(|x| *x == present_name).unwrap();
            list_data.presents.remove(index);

            // Update the list with the present removed
            self.list_minter.authorize(|| {
                borrow_resource_manager!(self.list_def)
                    .update_non_fungible_data(&list_nft.id(), list_data);
            })
        }

        // Display the presents stored in the NFT
        pub fn display_list(&self, list: Proof) {
            assert!(list.resource_address() == self.list_def, "Wrong badge provided");
            let list_data: PresentList = list.non_fungible::<PresentList>().data();

            info!("==== Christmas list content");
            for item in list_data.presents.iter() {
                info!("{}", item);
            }
        }
    }
}