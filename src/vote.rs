use crate::here::here::Here;
use scrypto::prelude::*;
#[blueprint]
mod aye {

    struct Aye {
        statement: String,
        ayes: KeyValueStore<Global<Account>, i64>,
        noes: KeyValueStore<Global<Account>, i64>,
        aye_votes: i64,
        no_votes: i64,
        end_time: i64,
        resource: ResourceAddress,
        creator: Global<Account>,
        this_component: ComponentAddress,
        home_component: Global<Here>,
    }

    impl Aye {
        pub fn instantiate_vote(
            end_time: i64,
            creator: Global<Account>,
            resource: ResourceAddress,
            statement: String,
            home_component: ComponentAddress,
            dapp_deff: GlobalAddress,
        ) -> Global<Aye> {
            let (address, component) = Runtime::allocate_component_address(Aye::blueprint_id());

            let home: Global<Here> = home_component.into();

            Self {
                statement,
                ayes: KeyValueStore::new(),
                noes: KeyValueStore::new(),
                aye_votes: 0,
                no_votes: 0,
                end_time,
                resource,
                creator,
                this_component: component,
                home_component: home,
            }
            .instantiate()
            .prepare_to_globalize(OwnerRole::None)
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
                    "icon_url" => Url::of(""), locked;
                }
            ))
            .globalize()
        }

        // This is a method, because it needs a reference to self.  Methods can only be called on components
        pub fn vote(&mut self, voter: Global<Account>, vote: bool, bucket: FungibleBucket) {
            {
                let voter_badge = voter.get_owner_role();

                Runtime::assert_access_rule(voter_badge.rule);
            }

            self.home_component.vote_fee(bucket);

            let current_time = Clock::current_time_rounded_to_seconds().seconds_since_unix_epoch;
            assert!(current_time < self.end_time, "Voting has ended");
            if self.ayes.get(&voter).is_some() || self.noes.get(&voter).is_some() {
                panic!("You have already voted");
            }
            if vote {
                self.ayes.insert(voter, current_time);
                self.aye_votes += 1;
            } else {
                self.noes.insert(voter, current_time);
                self.no_votes += 1;
            }
        }
    }
}
