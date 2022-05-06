use scrypto::prelude::*;
use crate::membership_system::*;

blueprint! {
    struct ElectionSystem {
        membership_admin_badge: Vault,
        membership_system_address: ComponentAddress,
        membership_nft_def: ResourceAddress,
        current_leader_id: Option<NonFungibleId>,
        votes: HashMap<NonFungibleId, u64>,
        election_duration: u64,
        election_deadline: u64,
        who_voted: Vec<NonFungibleId>,
        election_decided: bool
    }

    impl ElectionSystem {
    
        pub fn new(election_duration: u64) -> ComponentAddress {
            // Setup the membership system component
            let (membership_system_address, admin_badge): (ComponentAddress, Bucket) = MembershipSystem::new();
            let membership_system: MembershipSystem = membership_system_address.into();
            let member_nft_def = membership_system.get_member_nft_def();

            let access_rules = AccessRules::new()
                .method("start_election", rule!(require(member_nft_def)))
                .method("close_election", rule!(require(member_nft_def)))
                .default(rule!(allow_all));

            Self {
                membership_admin_badge: Vault::with_bucket(admin_badge),
                membership_system_address: membership_system_address,
                membership_nft_def: member_nft_def.into(),
                current_leader_id: None,
                votes: HashMap::new(),
                election_duration: election_duration,
                election_deadline: 0,
                who_voted: Vec::new(),
                election_decided: true
            }
            .instantiate().add_access_check(access_rules).globalize()
        }

        // Start a new election
        pub fn start_election(&mut self) {
            assert!(Runtime::current_epoch() >= self.election_deadline && self.election_decided, "An election is already ongoing");
            self.election_deadline = Runtime::current_epoch() + self.election_duration;
            self.election_decided = false;
        }

        // As a member of the DAO, vote for who
        // should be the leader
        pub fn vote(&mut self, member_id: NonFungibleId, auth: Proof) {
            assert_eq!(auth.resource_address(), self.membership_nft_def, "Wrong proof provided!");
            let auth_nft = auth.non_fungible::<MemberData>();

            assert!(Runtime::current_epoch() < self.election_deadline, "Election not yet started !");
            assert!(!self.who_voted.contains(&auth_nft.id()), "You already voted !");

            // Make sure NFT with member_id exists
            borrow_resource_manager!(self.membership_nft_def).get_non_fungible_data::<MemberData>(&member_id);

            // Increase the number of votes by one
            let existing_votes = *self.votes.entry(member_id.clone()).or_insert(0);
            self.votes.insert(member_id, existing_votes + 1);

            self.who_voted.push(auth_nft.id());
        }

        // Close the election and find the member with the
        // highest vote
        pub fn close_election(&mut self) {
            assert!(Runtime::current_epoch() >= self.election_deadline  && !self.election_decided, "The election has not ended yet.");
            // Find who won
            let mut highest_votes = -1;
            let mut highest_votes_member_id: Option<NonFungibleId> = None;

            for (id, nb_votes) in self.votes.iter() {
                if *nb_votes as i128 > highest_votes {
                    highest_votes = *nb_votes as i128;
                    highest_votes_member_id = Some(id.clone());
                }
            }

            self.current_leader_id = highest_votes_member_id;

            // Clear data for next election
            self.who_voted.clear();
            self.votes.clear();
            self.election_decided = true;
        }

        // Display the current leader of the DAO
        pub fn get_current_leader(&self) {
            let member_data: MemberData = match self.current_leader_id.clone() {
                Some(id) => borrow_resource_manager!(self.membership_nft_def).get_non_fungible_data(&id),
                None => {
                    info!("No current leader !");
                    std::process::abort();
                }
            };
            
            info!("Current leader: {}", member_data.name);
        }
    }
}
