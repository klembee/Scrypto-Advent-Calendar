use scrypto::prelude::*;

#[derive(NonFungibleData, Decode, Encode, TypeId, Describe)]
pub struct MemberData {
    name: String,
    #[scrypto(mutable)]
    good_member_points: Decimal,
    #[scrypto(mutable)]
    pub is_banned: bool,
    #[scrypto(mutable)]
    pub fund_share: Decimal
}

blueprint! {
    struct MembershipSystem {
        admin_def: ResourceAddress,
        minter: Vault,
        contributions: Vault,
        member_nft_def: ResourceAddress,
        nb_members: u64,
    }

    impl MembershipSystem {
        pub fn new() -> (ComponentAddress, Bucket) {
            // Create an admin badge
            let admin: Bucket = ResourceBuilder::new_fungible()
                                .divisibility(DIVISIBILITY_NONE)
                                .initial_supply(1);

            // Minter badge, kept by the component
            // to mint/burn/update new member NFTs
            let minter = ResourceBuilder::new_fungible()
                            .divisibility(DIVISIBILITY_NONE)
                            .initial_supply(1);

            // Create the definition of the member NFT.
            // Declare the NFT as mintable, burnable, recallable and updatable by 
            // the minter
            let member_nft_def = ResourceBuilder::new_non_fungible()
                                .metadata("name", "Member NFT")
                                .mintable(rule!(require(minter.resource_address())), LOCKED)
                                .burnable(rule!(require(minter.resource_address())), LOCKED)
                                .updateable_non_fungible_data(rule!(require(minter.resource_address()) || require(admin.resource_address())), LOCKED)
                                .no_initial_supply();

            let component = Self {
                minter: Vault::with_bucket(minter),
                contributions: Vault::new(RADIX_TOKEN),
                member_nft_def: member_nft_def,
                nb_members: 0,
                admin_def: admin.resource_address()
            }
            .instantiate();

            (component.globalize(), admin)
        }

        // Allow anyone to become a member of the DAO.
        // The component mints a badge representing the user.
        pub fn become_member(&mut self, name: String) -> Bucket {
            self.nb_members += 1;

            self.minter.authorize(|| {
                borrow_resource_manager!(self.member_nft_def).mint_non_fungible(&NonFungibleId::from_u64(self.nb_members), MemberData{
                    name: name, 
                    good_member_points: Decimal::zero(),
                    is_banned: false,
                    fund_share: Decimal::zero()
                })
            })
        }

        // Allow members to contribute XRD to the
        // DAO's vault. The member receive points based on
        // how much they give
        pub fn contribute(&mut self, payment: Bucket, auth: Proof) {
            assert_eq!(auth.resource_address(), self.member_nft_def, "Wrong badge provided!");
            let auth_nft = auth.non_fungible();

            let points = payment.amount();
            self.contributions.put(payment);

            // Add points to the nft metadata
            let mut nft_data: MemberData = auth_nft.data();
            assert!(!nft_data.is_banned, "You are banned from the DAO !");  
            nft_data.good_member_points += points;

            self.minter.authorize(|| {
                borrow_resource_manager!(self.member_nft_def).update_non_fungible_data(&auth_nft.id(), nft_data);
            });

            info!("Thank you ! You received {} points !", points);
        }

        // Allow members with more than 10000 points
        // to ban another member
        pub fn ban_member(&mut self, nft_id: u64, auth: Proof) {
            assert_eq!(auth.resource_address(), self.member_nft_def, "Wrong badge provided!");     
            let auth_nft = auth.non_fungible();
            
            let nft_data: MemberData = auth_nft.data();
            assert!(!nft_data.is_banned, "You are banned from the DAO !");
            assert!(nft_data.good_member_points >= dec!("10000"), "You do not have enough points to ban another member !");

            let member_resource_manager = borrow_resource_manager!(self.member_nft_def);

            let mut other_member_nft_data: MemberData = member_resource_manager.get_non_fungible_data(&NonFungibleId::from_u64(nft_id));
            other_member_nft_data.is_banned = true;
            self.minter.authorize(|| {
                member_resource_manager.update_non_fungible_data(&NonFungibleId::from_u64(nft_id), other_member_nft_data);
            });
        }

        // Will be used by other components of the DAO to
        // get the member NFT resource definition
        pub fn get_member_nft_def(&self) -> ResourceAddress {
            self.member_nft_def
        }

        pub fn get_nb_members(&self) -> u64 {
            self.nb_members
        }
    }
}
