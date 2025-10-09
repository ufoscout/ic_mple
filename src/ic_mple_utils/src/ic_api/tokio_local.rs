use std::time::SystemTime;

use candid::Principal;

use crate::ic_api::IcTrait;

/// An implementation of the IC API for local development
/// This runs on the host machine instead of the IC
/// This is useful for local development and testing
/// This should not be used in production as most of the returned data is fake
#[derive(Clone, Debug)]
pub struct TokioIcApi {
    canister_id: candid::Principal,
    canister_cycle_balance: u128,
}

impl Default for TokioIcApi {
    fn default() -> Self {
        Self { 
            canister_id: Principal::anonymous(), 
            canister_cycle_balance: Default::default() 
        }
    }
}

impl TokioIcApi {
    pub fn new(canister_id: Principal, canister_cycle_balance: u128) -> Self {
        Self { canister_id, canister_cycle_balance }
    }

    /// Sets the Principal of the canister to use when interacting with the IC API.
    pub fn set_canister_id(&mut self, canister_id: Principal) {
        self.canister_id = canister_id;
    }

    
    /// Sets the current cycle balance of the canister.
    pub fn set_canister_cycle_balance(&mut self, canister_cycle_balance: u128) {
        self.canister_cycle_balance = canister_cycle_balance;
    }
}

impl IcTrait for TokioIcApi {
    fn canister_self(&self) -> candid::Principal {
        self.canister_id
    }

    fn canister_cycle_balance(&self) -> u128 {
        self.canister_cycle_balance
    }

    fn time_ns(&self) -> u64 {
        self.current_system_time()
    .duration_since(std::time::SystemTime::UNIX_EPOCH)
    .expect("get current timestamp error")
    .as_secs()
    }

    fn time_secs(&self) -> u64 {
        self.current_system_time()
        .duration_since(std::time::SystemTime::UNIX_EPOCH)
        .expect("get current timestamp error")
        .as_nanos() as u64
    }

    fn current_system_time(&self) -> std::time::SystemTime {
        SystemTime::now()
    }

    fn spawn<F: 'static + Future<Output = ()>>(&self, future: F) {
        tokio::task::spawn_local(future);
    }

    fn print<S: std::convert::AsRef<str>>(&self, s: S) {
        println!("{}", s.as_ref())
    }
}