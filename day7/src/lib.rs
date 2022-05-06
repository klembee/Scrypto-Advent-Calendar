use scrypto::prelude::*;

// ElfWorkshop component. 
// People can register as elf to receive a badge.
// They can then use the badge to create new toys and the component
// keeps track of the amount of toys each elf created.
blueprint! {
    struct ElfWorkshop {
        // Vault that will contain the badge allowing this component to mint new elf_badges
        elf_badge_minter: Vault,
        // Resource definition of the elf badges
        elf_badge: ResourceAddress,
        // Maps elf's badge to an hashmap mapping toy name to quantity
        toys: HashMap<ResourceAddress, HashMap<String, u32>>
    }

    impl ElfWorkshop {
        pub fn new() -> ComponentAddress {
            // Create a badge allowing this component to mint new elf badges
            let elf_badge_minter: Bucket = ResourceBuilder::new_fungible()
                .divisibility(DIVISIBILITY_NONE)
                .metadata("name", "Elf badge minter")
                .initial_supply(1);

            // Define a mintable resource representing the elf badges
            // Only people presenting the elf_badge_minter badge can mint this resource.
            // The LOCKED flag makes sure that we cannot update this authorization rule.
            let elf_badges: ResourceAddress = ResourceBuilder::new_fungible()
                .divisibility(DIVISIBILITY_NONE)
                .metadata("name", "Elf Badge")
                .mintable(auth!(require(elf_badge_minter.resource_address())), LOCKED)
                .no_initial_supply();

            // Instantiate the component
            Self {
                elf_badge_minter: Vault::with_bucket(elf_badge_minter),
                elf_badge: elf_badges,
                toys: HashMap::new()
            }
            .instantiate().globalize()
        }

        pub fn become_elf(&mut self) -> Bucket {
            info!("Welcome to the factory, here is your badge");

            // Mint a new badge and send it to the caller
            // Vault.authorize takes the badge from the vault and puts it
            // on the component's auth zone.
            self.elf_badge_minter.authorize(|| {
                borrow_resource_manager!(self.elf_badge).mint(1)
            })
        }

        pub fn create_toy(&mut self, name: String, badge: Proof) {
            assert!(badge.resource_address() == self.elf_badge, "That's not a valid bage !");
            
            // The badge's address is used to identify the elf
            let elf_id = badge.resource_address();

            // We always need to drop bucket refs or else we get an error !
            badge.drop();

            // Insert the toy in the hashmap
            let elf_toys = self.toys.entry(elf_id).or_insert(HashMap::new());
            let old_count = *elf_toys.entry(name.clone()).or_insert(0);
            elf_toys.insert(name.clone(), old_count + 1);

            info!("The total amount of {} you created is {}", name, old_count + 1)
        }
        
    }
}
