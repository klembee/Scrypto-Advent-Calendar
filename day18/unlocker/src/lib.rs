use sbor::*;
use scrypto::prelude::*;

import! {
r#"
{
  "package_address": "01232a1e751e830c96908eafaf2607b3b20295e2c483aba40235de",
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

#[derive(Debug, TypeId, Encode, Decode, Describe, PartialEq, Eq)]
struct PercentageData {
  percentage: Decimal,
  nb_admin_approved: u32,
}

#[derive(NonFungibleData)]
struct RecipientData {
  amount: Decimal,
  #[scrypto(mutable)]
  percentage_unlocked: Decimal,
}

blueprint! {
  struct PriceBasedUnlockScheduler {
    // Used to have a reference to the price oracle
    // to get the price of the token
    price_oracle_address: ComponentAddress,
    // Keep track of the different unlock steps
    token_percentage_unlocked: HashMap<Decimal, PercentageData>,
    // Badge definition used to protect methods on the component
    admin_def: ResourceAddress,
    // Vault to store the tokens
    tokens: Vault,
    // Maps recipient badges to the amount left to withdraw
    recipients: HashMap<ResourceAddress, Decimal>,
    minter_badge: Vault,
    recipient_def: ResourceAddress,
    percentage_unlocked: Decimal
  }

  impl PriceBasedUnlockScheduler {
    pub fn new(token_def: ResourceAddress, price_oracle_address: ComponentAddress) -> (ComponentAddress, Bucket) {
      // Create an admin badge used for
      // authorization to call methods on the component
      let admin_badge = ResourceBuilder::new_fungible()
                          .divisibility(DIVISIBILITY_NONE)
                          .metadata("name", "Unlock Scheduler")
                          .initial_supply(1);

      // Define a minter badge, used to mint recipient NFTs
      // and update their individual metadata
      let minter_badge = ResourceBuilder::new_fungible()
                          .divisibility(DIVISIBILITY_NONE)
                          .metadata("name", "Unlock Scheduler")
                          .initial_supply(1);

      // Define the recipient NFT, used to keep track
      // of how many tokens a user have left to unlock
      let recipient_def = ResourceBuilder::new_non_fungible()
                            .metadata("name", "Recipient Data")
                            .mintable(rule!(require(minter_badge.resource_address())), LOCKED)
                            .updateable_non_fungible_data(rule!(require(minter_badge.resource_address())), LOCKED)
                            .no_initial_supply();

      // Define the different unlocking steps
      let mut token_percentage_unlocked = HashMap::new();
      token_percentage_unlocked.insert(dec!("10"), PercentageData { percentage: dec!("10"), nb_admin_approved: 0 }); // At 10$, unlock 10% of the supply
      token_percentage_unlocked.insert(dec!("20"), PercentageData { percentage: dec!("30"), nb_admin_approved: 0 }); // At 20$, unlock 30% of the supply
      token_percentage_unlocked.insert(dec!("50"), PercentageData { percentage: dec!("60"), nb_admin_approved: 0 }); // At 50$, unlock 60% of the supply
      token_percentage_unlocked.insert(dec!("60"), PercentageData { percentage: dec!("100"), nb_admin_approved: 0 }); // At 60$, unlock 100% of the supply

      // Store all required information on the component's state
      let component = Self {
        price_oracle_address: price_oracle_address,
        token_percentage_unlocked: token_percentage_unlocked,
        admin_def: admin_badge.resource_address(),
        tokens: Vault::new(token_def),
        recipients: HashMap::new(),
        minter_badge: Vault::with_bucket(minter_badge),
        recipient_def: recipient_def,
        percentage_unlocked: Decimal::zero()
      }.instantiate();

      let access_rules = AccessRules::new()
        .method("add_recipient", rule!(require(admin_badge.resource_address())))
        .method("do_unlock", rule!(require(admin_badge.resource_address())))
        .default(rule!(allow_all));

      // Return the component and admin badge to the caller
      (component.add_access_check(access_rules).globalize(), admin_badge)
    }

    pub fn add_recipient(&mut self, recipient: ComponentAddress, tokens: Bucket) {
      // Mint a new NFT for the recipient
      let recipient_nft = self.minter_badge.authorize(|| {
        // Keep track of how much that account owns by
        // inserting it in the NFT metadata
        borrow_resource_manager!(self.recipient_def)
          .mint_non_fungible(&NonFungibleId::random(), RecipientData {amount: tokens.amount(), percentage_unlocked: Decimal::zero()})
      });

      // Store the user's token in the component's vault
      self.tokens.put(tokens);

      // Send the NFT to the account
      borrow_component!(recipient).call::<()>("deposit", vec![scrypto_encode(&recipient_nft)]);
    }

    pub fn do_unlock(&mut self) {
      // Get the current price of the asset
      let price_oracle: PriceOracle = self.price_oracle_address.into();
      let current_price = match price_oracle.get_price(self.tokens.resource_address(), price_oracle.get_usd_address()) {
        Some(price) => price,
        None => {
          info!("No price found for {}", borrow_resource_manager!(self.tokens.resource_address()).metadata().get("name").unwrap());
          std::process::abort();
        }
      };

      // Update the percentage unlocked
      for (price, data) in self.token_percentage_unlocked.iter() {
        if *price <= current_price && data.percentage > self.percentage_unlocked {
          self.percentage_unlocked = data.percentage;
        }
      }

      info!("percentage_unlocked: {}", self.percentage_unlocked);
    }

    // Allow a recipient to withdraw the unlocked
    // tokens that have not been withdrawn yet.
    pub fn withdraw(&mut self, recipient_nft: Proof) -> Bucket {
      let nft = recipient_nft.non_fungible::<RecipientData>();
      assert!(recipient_nft.resource_address() == self.recipient_def, "Wrong token");

      // Fetch the metadata of the NFT
      let mut nft_data: RecipientData = nft.data();
      let amount = nft_data.amount;
      recipient_nft.drop();

      let to_unlock = self.percentage_unlocked - nft_data.percentage_unlocked;

      // Set the total_unlocked on the NFT.
      // This is necessesary to make sure the recipient
      // can only withdraw what they haven't withdrawn yet
      nft_data.percentage_unlocked = self.percentage_unlocked;

      // Insert the new metadata on the NFT
      self.minter_badge.authorize(|| {
        borrow_resource_manager!(self.recipient_def).update_non_fungible_data(&nft.id(), nft_data);
      });

      self.tokens.take(amount * (to_unlock / dec!("100")))
    }
  }
}
