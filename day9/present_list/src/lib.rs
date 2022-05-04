use scrypto::prelude::*;

blueprint! {
    struct PresentList {
        // Used to store the presents in the list for every account
        lists: HashMap<ResourceAddress, Vec<String>>,
    }

    impl PresentList {
        pub fn new() -> ComponentAddress {
            // Store all required information info the component's state
            Self {
                lists: HashMap::new()
            }
            .instantiate().globalize()
        }
        
        // Allow the user to start a new christmas list.
        // It generates a badge that will allow users to add and remove
        // presents associated with it.
        pub fn start_new_list(&mut self) -> Bucket {
            // Mint a new christmas list badge
            let list_bucket = ResourceBuilder::new_fungible()
                .divisibility(DIVISIBILITY_NONE)
                .metadata("name", format!("Christmas List ID #{}", self.lists.len() + 1))
                .initial_supply(1);

            // Store an empty list for the badge's address in the lists map
            self.lists.insert(list_bucket.resource_address(), vec![]);

            // Return the list badge to the caller
            list_bucket
        }

        // Add a new present to the list
        pub fn add(&mut self, present_name: String, list_badge: Proof) {
            let list_address = list_badge.resource_address();
            assert!(self.lists.contains_key(&list_address), "Invalid badge provided");

            let list = self.lists.get(&list_address).unwrap();

            // Make sure that the present is not already inside the user's list
            assert!(!list.contains(&present_name), "Present already on the list !");

            let mut presents = list.clone();
            presents.push(present_name);

            // Update the list with the newly added present
            self.lists.insert(list_address, presents);
            info!("Present added to your list !");
        }

        // Remove a present in the list
        pub fn remove(&mut self, present_name: String, list_badge: Proof) {
            let list_address = list_badge.resource_address();
            assert!(self.lists.contains_key(&list_address), "Invalid badge provided!");

            let list = self.lists.get(&list_address).unwrap();

            // Make sure that the present is not already inside the user's list
            assert!(list.contains(&present_name), "Present not on the list !");

            let mut presents = list.clone();
        
            // Find the index of the present to remove
            let index = presents.iter().position(|x| *x == present_name).unwrap();
            presents.remove(index);

            // Update the list with the present removed
            self.lists.insert(list_address, presents);
        }

        // Display the presents stored in the list
        // associated with the list badge
        pub fn display_list(&self, list_badge: Proof) {
            let list_address = list_badge.resource_address();
            assert!(self.lists.contains_key(&list_address), "Invalid badge provided!");

            let list = self.lists.get(&list_address).unwrap();

            info!("==== Christmas list content");
            for item in list.iter() {
                info!("{}", item);
            }
        }

        // This method is used to retrieve the lists from the Santa component
        pub fn get_lists(&self) -> HashMap<ResourceAddress, Vec<String>> {
            self.lists.clone()
        }
    }
}