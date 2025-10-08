use std::cell::RefCell;

use candid::{CandidType, Deserialize, Principal};
use ic_cdk::{query, update};
use ic_mple_client::IcCanisterClient;

use crate::client::TestCanisterClient;

pub mod client;

thread_local! {
    static COUNTER: RefCell<u64> = const { RefCell::new(0) };
    static DROP_COUNTER: RefCell<u64> = const { RefCell::new(0) };
    static CONFIG: RefCell<Config> = RefCell::new(Config::default());
}

#[derive(Default)]
struct Config {
    pub other_canister: Option<Principal>,
}

#[derive(Debug, Clone, CandidType, Deserialize)]
pub struct InitArgs {
    pub other_canister: Option<Principal>,
}

#[ic_cdk::init]
fn init(arg: InitArgs) {
    CONFIG.with(|c| {
        c.replace(Config {
            other_canister: arg.other_canister,
        })
    });
}

#[query]
fn get_counter() -> u64 {
    COUNTER.with(|c| *c.borrow())
}

#[update]
fn increment_counter(amount: u64) {
    COUNTER.with(|counter| *counter.borrow_mut() += amount);
}

#[query(composite)]
async fn counter_of_other_canister() -> u64 {
    let other_canister = CONFIG.with(|config| config.borrow().other_canister.unwrap());

    // Use IcCanisterClient to perform an intercanister call
    let client = IcCanisterClient::new(other_canister, Some(10));
    let client = TestCanisterClient::new(client);
    client.get_counter().await.unwrap()
}

// Enable Candid export
ic_cdk::export_candid!();
