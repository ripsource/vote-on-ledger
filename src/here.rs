use crate::vote::aye::Aye;
use scrypto::prelude::*;

#[derive(ScryptoSbor, ScryptoEvent)]
pub struct VoteCreated {
    pub vote_id: i64,
    pub creator: ComponentAddress,
    pub statement: String,
    pub end_time: i64,
}

#[blueprint]
mod here {
    struct Here {
        votes: KeyValueStore<i64, (String, ResourceAddress, i64, ComponentAddress)>,
        votes_id: i64,
        here_component: ComponentAddress,
        rev: FungibleVault,
        owner: ResourceAddress,
        cost: Decimal,
        vote_cost: Decimal,
    }

    impl Here {
        pub fn instantiate_here(
            owner: ResourceAddress,
            dapp_deff: ComponentAddress,
        ) -> Global<Here> {
            let (address, component) = Runtime::allocate_component_address(Here::blueprint_id());
            Self {
                votes: KeyValueStore::new(),
                votes_id: 0,
                here_component: component,
                rev: FungibleVault::new(XRD),
                owner,
                cost: dec!(69),
                vote_cost: dec!(6.9),
            }
            .instantiate()
            .prepare_to_globalize(OwnerRole::Fixed(rule!(require(owner))))
            .with_address(address)
            .metadata(metadata! (
                roles {
                    metadata_setter => rule!(deny_all);
                    metadata_setter_updater => rule!(deny_all);
                    metadata_locker => rule!(deny_all);
                    metadata_locker_updater => rule!(deny_all);
                },
                init {
                    "name" => "Here Here".to_owned(), locked;
                    "description" => "Will the ayes or the noes have it?".to_owned(), locked;
                    "dapp_definition" => dapp_deff, locked;
                    "icon_url" => Url::of("https://e3.365dm.com/23/09/2048x1152/skynews-john-bercow-speaker_6293239.jpg"), locked;
                }
            ))
            .globalize()
        }

        pub fn withdraw_fee(&mut self, proof: Proof) -> FungibleBucket {
            proof.check(self.owner);

            self.rev.take_all()
        }

        pub fn update_cost(&mut self, proof: Proof, new_cost: Decimal) {
            proof.check(self.owner);

            self.cost = new_cost;
        }

        pub fn vote_fee(&mut self, fee: FungibleBucket) {
            assert!(fee.resource_address() == XRD, "Payment must be in XRD");

            assert!(
                fee.amount() >= self.vote_cost,
                "Payment must be at least 6.9 XRD"
            );

            self.rev.put(fee);
        }

        // This is a method, because it needs a reference to self.  Methods can only be called on components
        pub fn create_vote(
            &mut self,
            creator: Global<Account>,
            statement: String,
            end: i64,
            resource: ResourceAddress,
            mut payment: FungibleBucket,
        ) -> FungibleBucket {
            assert!(payment.resource_address() == XRD, "Payment must be in XRD");

            assert!(
                payment.amount() >= self.cost,
                "Payment must be at least 500 XRD"
            );

            let fee = payment.take(self.cost);

            self.rev.put(fee);

            let dapp_def_account =
                Blueprint::<Account>::create_advanced(OwnerRole::Updatable(rule!(allow_all)), None); // will reset owner role after dapp def metadata has been set
            dapp_def_account.set_metadata("account_type", String::from("dapp definition"));
            dapp_def_account.set_metadata("name", "Here Here".to_string());
            dapp_def_account.set_metadata(
                "description",
                "Will the ayes or the noes have it?".to_string(),
            );

            dapp_def_account.set_metadata(
                "icon_url",
                Url::of(
                    "https://e3.365dm.com/23/09/2048x1152/skynews-john-bercow-speaker_6293239.jpg",
                ),
            );

            let dapp_def_address = GlobalAddress::from(dapp_def_account.address());

            self.votes_id += 1;

            let vote_component = Aye::instantiate_vote(
                end,
                creator,
                resource,
                statement.clone(),
                self.here_component,
                dapp_def_address,
            );

            dapp_def_account.set_metadata(
                "claimed_entities",
                vec![GlobalAddress::from(vote_component.address())],
            );
            dapp_def_account.set_owner_role(rule!(require(self.owner)));

            let vote_address = vote_component.address();

            self.votes
                .insert(self.votes_id, (statement, resource, end, vote_address));

            payment
        }
    }
}
