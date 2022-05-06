use scrypto::prelude::*;
use crate::membership_system::*;

#[derive(Decode, Encode, TypeId, Describe, Clone)]
pub struct Review {
    service: String,
    stars: u8,
    text: String,
    offered_on: u64,
    reviewed_on: u64
}

blueprint! {
    struct RatingSystem {
        membership_nft_admin: Vault,
        membership_system_address: ComponentAddress,
        membership_nft_def: ResourceAddress,
        reviews: HashMap<NonFungibleId, Vec<Review>>
    }

    impl RatingSystem {
        pub fn new() -> ComponentAddress {
            let (membership_system_address, admin_badge): (ComponentAddress, Bucket) = MembershipSystem::new();
            let membership_system: MembershipSystem = membership_system_address.into();
            let member_nft_def = membership_system.get_member_nft_def();

            Self {
                membership_nft_admin: Vault::with_bucket(admin_badge),
                membership_nft_def: member_nft_def.into(),
                membership_system_address: membership_system_address,
                reviews: HashMap::new()
            }
            .instantiate().globalize()
        }

        // As a member of the DAO
        // create a new service that you are offering
        pub fn create_service(&mut self, service: String, auth: Proof){
            assert_eq!(auth.resource_address(), self.membership_nft_def, "Wrong proof provided!");
            let auth_nft = auth.non_fungible();

            let mut nft_data: MemberData = auth_nft.data();
            assert!(!nft_data.services.contains(&service), "You are already offering that service.");

            // Update the list of services on the member NFT
            nft_data.services.push(service);

            // Insert empty review list for that member
            self.reviews.insert(auth_nft.id(), Vec::new());

            self.membership_nft_admin.authorize(|| {
                borrow_resource_manager!(self.membership_nft_def).update_non_fungible_data(&auth_nft.id(), nft_data);
            });
        }

        // As a user, review services received from a
        // member of the DAO.
        pub fn review_service(&mut self, member_id: NonFungibleId, service_name: String, offered_on: u64, stars: u8, review: String) {
            assert!(stars <= 5, "stars param must be between 0 and 5");
            assert!(offered_on <= Runtime::current_epoch(), "The service must have been offered in the past !");

            let member_nft_data: MemberData = borrow_resource_manager!(self.membership_nft_def).get_non_fungible_data(&member_id);
            assert!(member_nft_data.services.contains(&service_name), "The member is not offering that service");

            // Insert the review on the member's nft and increase their good member points
            let mut review_list = match self.reviews.get(&member_id) {
                Some(reviews) => reviews.clone(),
                None => {
                    info!("Internal error");
                    std::process::abort();
                }
            };

            review_list.push(Review {
                service: service_name,
                stars: stars,
                offered_on: offered_on,
                text: review,
                reviewed_on: Runtime::current_epoch()
            });

            self.reviews.insert(member_id, review_list.clone());
        }

        // Display the services and ratings for a particular member
        pub fn display_ratings(&self, member_id: NonFungibleId) {
            let member_data: MemberData = borrow_resource_manager!(self.membership_nft_def).get_non_fungible_data(&member_id);

            info!("Service: ");
            info!("=====");
            for service in member_data.services {
                info!("{}", service);
            }

            let reviews = self.reviews.get(&member_id);
            if reviews.is_none() {
                return;
            }

            info!("Reviews");
            info!("======");
            for review in reviews.unwrap() {
                info!("For service: {}", review.service);
                info!("Stars: {}/5", review.stars);
                info!("message: {}", review.text);
            }
        }
    }
}
