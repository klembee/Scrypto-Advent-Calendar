use scrypto::prelude::*;
use crate::membership_system::*;

#[derive(NonFungibleData)]
struct ProposalData {
    created_by_id: NonFungibleId,
    title: String,
    description: String,
    created_at: u64,
    #[scrypto(mutable)]
    voted_by: Vec<ResourceAddress>
}

blueprint! {
    struct ProposalVoting {
        proposals: Vault,
        proposal_minter: Vault,
        proposal_def: ResourceAddress,
        nb_proposals: u64,
        membership_admin: Vault,
        membership_system_address: ComponentAddress,
        member_nft_def: ResourceAddress
    }

    impl ProposalVoting {
        pub fn new() -> ComponentAddress {
            // Badge allowed to mint new proposal NFTs
            let proposal_minter = ResourceBuilder::new_fungible()
                                    .divisibility(DIVISIBILITY_NONE)
                                    .initial_supply(1);

            // Create proposal NFT definition
            let proposal_definition = ResourceBuilder::new_non_fungible()
                                        .metadata("name", "Proposal")
                                        .mintable(rule!(require(proposal_minter.resource_address())), LOCKED)
                                        .updateable_non_fungible_data(rule!(require(proposal_minter.resource_address())), LOCKED)
                                        .no_initial_supply();

            // Instantiate the membership system component
            let (membership_system_address, admin_badge): (ComponentAddress, Bucket) = MembershipSystem::new();
            let membership_system: MembershipSystem = membership_system_address.into();
            let member_nft_def = membership_system.get_member_nft_def();

            Self {
                proposals: Vault::new(proposal_definition),
                proposal_minter: Vault::with_bucket(proposal_minter),
                proposal_def: proposal_definition,
                nb_proposals: 0,
                membership_admin: Vault::with_bucket(admin_badge),
                membership_system_address: membership_system_address,
                member_nft_def: member_nft_def
            }
            .instantiate().globalize()
        }

        // As a member, create a new proposal with
        // provided title and description
        pub fn create_proposal(&mut self, title: String, description: String, auth: Proof) {
            assert_eq!(auth.resource_address(), self.member_nft_def, "Wrong badge provided!");

            let proposal =self.proposal_minter.authorize(|| {
                borrow_resource_manager!(self.proposal_def).mint_non_fungible(&NonFungibleId::from_u64(self.nb_proposals), ProposalData {
                    created_by_id: auth.non_fungible::<MemberData>().id(),
                    title: title, 
                    description: description, 
                    voted_by: Vec::new(),
                    created_at: Runtime::current_epoch()
                })
            });

            self.nb_proposals += 1;
            self.proposals.put(proposal);
        }

        // As a member, vote for a proposal with
        // provided id
        pub fn vote_on_proposal(&self, proposal_id: NonFungibleId, auth: Proof) {
            assert_eq!(auth.resource_address(), self.member_nft_def, "Wrong badge provided!");
            let mut nft_data: ProposalData = borrow_resource_manager!(self.proposal_def).get_non_fungible_data(&proposal_id);

            // Make sure that the member voting is not he
            // one that created the proposal and that they have not already
            // voted on it.
            assert!(nft_data.created_by_id != auth.non_fungible::<MemberData>().id(), "You can't vote on your own proposal");
            assert!(!nft_data.voted_by.contains(&auth.resource_address()), "Already voted for that proposal !");

            // Add the member id to the list of votes
            nft_data.voted_by.push(auth.resource_address());

            // Update the NFT's data
            self.proposal_minter.authorize(|| {
                borrow_resource_manager!(self.proposal_def).update_non_fungible_data(&proposal_id, nft_data);
            })
        }

        // List all the proposals and the amount of 
        // votes they have
        pub fn list_proposals(&self) {
            info!("==== Proposals =====");
            let proposal_resource_manager = borrow_resource_manager!(self.proposal_def);

            for i in 0..self.nb_proposals {
                let data: ProposalData = proposal_resource_manager.get_non_fungible_data(&NonFungibleId::from_u64(i));
                info!("Title: {}", data.title);
                info!("Description: {}", data.description);
                info!("Nb votes: {}", data.voted_by.len());
            }
        }
    }
}
