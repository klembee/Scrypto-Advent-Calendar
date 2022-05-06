use scrypto::prelude::*;
use crate::membership_system::*;

blueprint! {
    struct FundsSplitter {
        admin_badge: Vault,
        membership_system_address: ComponentAddress,
        member_nft_def: ResourceAddress,
        funds: Vault
    }

    impl FundsSplitter {
        pub fn new() -> ComponentAddress {
            // Instantiate the membership system component
            let (membership_system_address, admin_badge): (ComponentAddress, Bucket) = MembershipSystem::new();
            let membership_system: MembershipSystem = membership_system_address.into();

            let member_nft_def = membership_system.get_member_nft_def();

            Self {
                admin_badge: Vault::with_bucket(admin_badge),
                membership_system_address: membership_system_address,
                member_nft_def: member_nft_def.into(),
                funds: Vault::new(RADIX_TOKEN)
            }
            .instantiate().globalize()
        }

        // Add funds to the DAO and split the amount between 
        // all members
        pub fn add_funds(&mut self, payment: Bucket) {
            let membership_system: MembershipSystem = self.membership_system_address.into();
            let nb_members = membership_system.get_nb_members();
            assert!(nb_members > 0, "No members to give the funds to !");

            // Split the funds equally between all members
            for i in 1..=nb_members {
                let mut nft_data: MemberData = borrow_resource_manager!(self.member_nft_def).get_non_fungible_data(&NonFungibleId::from_u64(i));

                // Update the shares on the NFT
                nft_data.fund_share += payment.amount() / nb_members;
                self.admin_badge.authorize(|| {
                    borrow_resource_manager!(self.member_nft_def).update_non_fungible_data(&NonFungibleId::from_u64(i), nft_data);
                });
            }

            // Store payment in DAO's fund vault
            self.funds.put(payment);
        }

        // Allow members to withdraw their share of the funds
        pub fn withdraw(&mut self, auth: Proof) -> Bucket {
            assert_eq!(auth.resource_address(), self.member_nft_def, "Wrong badge provided!");
            let auth_nft = auth.non_fungible();

            // Fetch data on the NFT
            let mut nft_data: MemberData = auth_nft.data();

            // Make a bucket with the XRD to return to the caller
            let shares_to_return: Bucket = self.funds.take(nft_data.fund_share);

            // Set the shares to 0 on the NFT
            nft_data.fund_share = Decimal::zero();
            self.admin_badge.authorize(|| { 
                borrow_resource_manager!(self.member_nft_def)
                    .update_non_fungible_data(&auth_nft.id(), nft_data);
            });

            shares_to_return
        }
    }
}
