

use std::collections::HashMap;
use std::convert::TryInto;

use access::{Access, Condition, Relationship};
use bloom_filter::{Bloom, WrappedHash};
use events::Event;
use near_sdk::borsh::{self, BorshDeserialize, BorshSerialize};
use near_sdk::json_types::{Base58CryptoHash, U128, U64};
use near_sdk::serde::{Serialize, Deserialize};
use near_sdk::serde_json::{json, self};
use near_sdk::{env, near_bindgen, AccountId, log, bs58, PanicOnDefault, Promise, BlockHeight, CryptoHash, assert_one_yocto};
use near_sdk::collections::{LookupMap, UnorderedMap, Vector, LazyOption, UnorderedSet};
use drip::Drip;
use post::Report;
use role::Role;
use utils::{check_args, verify, check_encrypt_args, refund_extra_storage_deposit, set_content};
use crate::post::Hierarchy;
use std::convert::TryFrom;
use role::Permission;


pub mod utils;
pub mod signature;
pub mod bloom_filter;
pub mod access;
pub mod post;
pub mod resolver;
pub mod owner;
pub mod drip;
pub mod view;
pub mod events;
pub mod role;



#[near_bindgen]
#[derive(BorshDeserialize, BorshSerialize, PanicOnDefault)]
pub struct Community {
    owner_id: AccountId,
    public_key: String,
    public_bloom_filter: Bloom,
    encryption_bloom_filter: Bloom,
    relationship_bloom_filter: Bloom,
    access: Option<Access>,
    reports: UnorderedMap<AccountId, UnorderedMap<Base58CryptoHash, Report>>,
    drip: Drip,
    roles: UnorderedMap<String, Role>
}

#[derive(BorshDeserialize, BorshSerialize, PanicOnDefault)]
pub struct OldCommunity {
    owner_id: AccountId,
    public_key: String,
    public_bloom_filter: Bloom,
    encryption_bloom_filter: Bloom,
    access: Option<Access>,
    members: UnorderedMap<AccountId, u32>,
}



const MAX_LEVEL: usize = 3;


#[near_bindgen]
impl Community {

    #[init]
    pub fn new(owner_id: AccountId, public_key: String) -> Self {
        let mut this = Self {
            owner_id: owner_id.clone(),
            public_key: public_key,
            public_bloom_filter: Bloom::new_for_fp_rate_with_seed(1000000, 0.1, "public".to_string()),
            encryption_bloom_filter: Bloom::new_for_fp_rate_with_seed(1000000, 0.1, "encrypt".to_string()),
            relationship_bloom_filter: Bloom::new_for_fp_rate_with_seed(1000000, 0.1, "relationship".to_string()),
            access: None,
            reports: UnorderedMap::new(b'r'),
            drip: Drip::new(),
            roles: UnorderedMap::new("roles".as_bytes())
        };
        this.drip.join(owner_id);
        this
    }

    #[init(ignore_state)]
    pub fn migrate() -> Self {

        let mut prev: OldCommunity = env::state_read().expect("ERR_NOT_INITIALIZED");
        // let success = env::storage_remove(b"r");
        // log!("{:?}", success);
        assert_eq!(
            env::predecessor_account_id(),
            prev.owner_id,
            "Only owner"
        );

        prev.members.clear();
        
        let this = Community {
            owner_id: prev.owner_id,
            public_key: prev.public_key,
            public_bloom_filter: prev.public_bloom_filter,
            encryption_bloom_filter: prev.encryption_bloom_filter,
            access: prev.access,
            relationship_bloom_filter: Bloom::new_for_fp_rate_with_seed(1000000, 0.1, "relationship".to_string()),
            reports: UnorderedMap::new(b'r'),
            drip: Drip::new(),
            roles: UnorderedMap::new("roles".as_bytes())
        };
        this
    }
    
    #[payable]
    pub fn join(&mut self) {
        let initial_storage_usage = env::storage_usage();
        let sender_id = env::predecessor_account_id();
        match &mut self.access {
            Some(v) => v.check_permission(sender_id),
            None => {
                self.drip.join(sender_id);
            }
        }
        refund_extra_storage_deposit(env::storage_usage() - initial_storage_usage, 0)
    }

    #[payable]
    pub fn quit(&mut self) {
        assert_one_yocto();
        let sender_id = env::predecessor_account_id();
        self.drip.quit(sender_id);
    }

    #[payable]
    pub fn collect_drip(&mut self) -> U128 {
        assert_one_yocto();
        let sender_id = env::signer_account_id();
        self.drip.get_and_clear_drip(sender_id)
    }
    
}





#[cfg(test)]
mod tests {


}