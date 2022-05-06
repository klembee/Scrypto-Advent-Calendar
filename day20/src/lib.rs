use scrypto::prelude::*;

#[derive(NonFungibleData)]
struct MemberData {
    name: String,
    #[scrypto(mutable)]
    good_member_points: Decimal,
    #[scrypto(mutable)]
    is_banned: bool
}

blueprint! {
    struct MembershipSystem {
        minter: Vault,
        contributions: Vault,
        member_nft_def: ResourceAddress,
        nb_members: u64,
    }

    impl MembershipSystem {
        pub fn new() -> ComponentAddress {
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
                                .updateable_non_fungible_data(rule!(require(minter.resource_address())), LOCKED)
                                .no_initial_supply();

            Self {
                minter: Vault::with_bucket(minter),
                contributions: Vault::new(RADIX_TOKEN),
                member_nft_def: member_nft_def,
                nb_members: 0
            }
            .instantiate().globalize()
        }

        // Allow anyone to become a member of the DAO.
        // The component mints a badge representing the user.
        pub fn become_member(&mut self, name: String) -> Bucket {
            self.nb_members += 1;

            self.minter.authorize(|| {
                borrow_resource_manager!(self.member_nft_def).mint_non_fungible(&NonFungibleId::from_u64(self.nb_members), MemberData{
                    name: name, 
                    good_member_points: Decimal::zero(),
                    is_banned: false
                })
            })
        }

        // Allow members to contribute XRD to the
        // DAO's vault. The member receive points based on
        // how much they give
        pub fn contribute(&mut self, payment: Bucket, auth: Proof) {
            assert!(auth.resource_address() == self.member_nft_def, "Wrong badge!");
            let auth_nft = auth.non_fungible::<MemberData>();    

            let points = payment.amount();
            self.contributions.put(payment);

            // Add points to the nft metadata
            let mut nft_data: MemberData = auth.non_fungible().data();
            assert!(!nft_data.is_banned, "You are banned from the DAO !");  
            nft_data.good_member_points += points;

            self.minter.authorize(|| {
                borrow_resource_manager!(self.member_nft_def).update_non_fungible_data(&auth_nft.id(), nft_data);
            });

            info!("Thank you ! You received {} points !", points);
        }

        // Allow members with more than 10000 points
        // to ban another member
        pub fn ban_member(&mut self, nft_id: NonFungibleId, auth: Proof) {
            assert!(auth.resource_address() == self.member_nft_def, "Wrong badge!");      
            let nft_data: MemberData = auth.non_fungible().data();
            assert!(!nft_data.is_banned, "You are banned from the DAO !");
            assert!(nft_data.good_member_points >= dec!("10000"), "You do not have enough points to ban another member !");

            let mut other_member_nft_data: MemberData = borrow_resource_manager!(self.member_nft_def).get_non_fungible_data(&nft_id);
            other_member_nft_data.is_banned = true;
            self.minter.authorize(|| {
                borrow_resource_manager!(self.member_nft_def).update_non_fungible_data(&nft_id, other_member_nft_data);
            });
        }

        // Will be used by other components of the DAO to
        // know if a member is banned
        pub fn is_banned(&self, nft: Proof) -> bool {
            assert!(nft.resource_address() == self.member_nft_def, "Wrong nft");

            let data: MemberData = nft.non_fungible().data();
            data.is_banned
        }

        // Will be used by other components of the DAO to
        // get the member NFT resource definition
        pub fn get_member_nft_def(&self) -> ResourceAddress {
            self.member_nft_def
        }
    }
}
