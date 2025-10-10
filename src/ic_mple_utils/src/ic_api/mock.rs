use std::{cell::RefCell, rc::Rc, time::{Duration, SystemTime}};

use candid::{CandidType, Principal};

use crate::ic_api::{IcTrait, E_9};

/// An implementation of the IC API for local development
/// This runs on the host machine instead of the IC
/// This is useful for local development and testing
/// This should not be used in production as most of the returned data is fake
#[derive(Clone, Debug, CandidType, PartialEq, Eq)]
pub struct MockIcApi {
    canister_id: Rc<RefCell<candid::Principal>>,
    canister_cycle_balance: Rc<RefCell<u128>>,
    time_nanos: Rc<RefCell<u64>>,
}

impl Default for MockIcApi {
    fn default() -> Self {
        Self {
            canister_id: Rc::new(RefCell::new(Principal::anonymous())),
            canister_cycle_balance: Rc::new(RefCell::new(Default::default())),
            time_nanos: Rc::new(RefCell::new(0)),
        }
    }
}

impl MockIcApi {

    /// Sets the Principal of the canister to use when interacting with the IC API.
    pub fn set_canister_id(&self, canister_id: Principal) {
        *self.canister_id.borrow_mut() = canister_id;
    }

    /// Sets the current cycle balance of the canister.
    pub fn set_canister_cycle_balance(&mut self, canister_cycle_balance: u128) {
        *self.canister_cycle_balance.borrow_mut() = canister_cycle_balance;
    }

    /// Sets the current time of the canister.
    pub fn set_time_nanos(&self, time: u64) {
        *self.time_nanos.borrow_mut() = time;
    }
}

impl IcTrait for MockIcApi {
    fn canister_self(&self) -> candid::Principal {
        self.canister_id.borrow().clone()
    }

    fn canister_cycle_balance(&self) -> u128 {
        self.canister_cycle_balance.borrow().clone()
    }

    fn time_nanos(&self) -> u64 {
        self.time_nanos.borrow().clone()
    }

    fn time_secs(&self) -> u64 {
        self.time_nanos() / E_9
    }

    fn current_system_time(&self) -> std::time::SystemTime {
        SystemTime::UNIX_EPOCH + Duration::from_nanos(self.time_nanos())
    }

    fn spawn<F: 'static + Future<Output = ()>>(&self, _future: F) {
        
    }

    fn print<S: std::convert::AsRef<str>>(&self, s: S) {
        println!("{}", s.as_ref())
    }
    
    fn spawn_detached<F: 'static + Future<Output = ()>>(&self, future: F) {
        self.spawn(future);
    }
}
