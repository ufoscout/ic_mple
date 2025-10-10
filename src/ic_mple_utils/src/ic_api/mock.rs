use std::{
    sync::{Arc, Mutex},
    time::SystemTime,
};

use candid::{CandidType, Deserialize, Principal};

use crate::ic_api::IcTrait;

/// The time strategy to use for the mocked IC API
#[derive(Clone, Debug, CandidType, Deserialize, PartialEq, Eq)]
pub enum TimeStrategy {
    /// Fixed time
    Fixed { timestamp_nanos: u64 },
    /// Current system time
    System,
}

/// An mocked implementation of the IC API for local development
/// This runs on the host machine instead of the IC
/// This is useful for local development and testing
/// This should not be used in production as most of the returned data is fake
#[derive(Clone, Debug)]
pub struct IcMock {
    canister_id: Arc<Mutex<candid::Principal>>,
    canister_cycle_balance: Arc<Mutex<u128>>,
    time_strategy: Arc<Mutex<TimeStrategy>>,
}

impl Default for IcMock {
    fn default() -> Self {
        Self {
            canister_id: Arc::new(Mutex::new(Principal::anonymous())),
            canister_cycle_balance: Default::default(),
            time_strategy: Arc::new(Mutex::new(TimeStrategy::System)),
        }
    }
}

impl IcMock {
    pub fn new(canister_id: Principal, canister_cycle_balance: u128) -> Self {
        Self {
            canister_id: Arc::new(Mutex::new(canister_id)),
            canister_cycle_balance: Arc::new(Mutex::new(canister_cycle_balance)),
            time_strategy: Arc::new(Mutex::new(TimeStrategy::System)),
        }
    }

    /// Sets the Principal of the canister to use when interacting with the IC API.
    pub fn set_canister_id(&mut self, canister_id: Principal) {
        *self.canister_id.lock().unwrap() = canister_id;
    }

    /// Sets the current cycle balance of the canister.
    pub fn set_canister_cycle_balance(&mut self, canister_cycle_balance: u128) {
        *self.canister_cycle_balance.lock().unwrap() = canister_cycle_balance;
    }

    /// Sets the time strategy to use for the IC API.
    pub fn set_time_strategy(&mut self, time_strategy: TimeStrategy) {
        *self.time_strategy.lock().unwrap() = time_strategy;
    }
}

impl IcTrait for IcMock {
    fn canister_self(&self) -> candid::Principal {
        *self.canister_id.lock().unwrap()
    }

    fn canister_cycle_balance(&self) -> u128 {
        *self.canister_cycle_balance.lock().unwrap()
    }

    fn time_nanos(&self) -> u64 {
        match *self.time_strategy.lock().unwrap() {
            TimeStrategy::Fixed { timestamp_nanos } => timestamp_nanos,
            TimeStrategy::System => SystemTime::now()
                .duration_since(std::time::SystemTime::UNIX_EPOCH)
                .expect("get current timestamp error")
                .as_nanos() as u64,
        }
    }

    fn spawn<F: 'static + Future<Output = ()>>(&self, _future: F) {
        #[cfg(feature = "tokio")]
        tokio::task::spawn_local(_future);

        #[cfg(not(feature = "tokio"))]
        {
            println!(
                "WARNING: spawn was called on the IcMockApi but tokio feature is not enabled so it will be ignored. To allow spawn to work, enable the tokio feature of ic_mple_utils"
            );
        }
    }

    fn print<S: std::convert::AsRef<str>>(&self, s: S) {
        println!("{}", s.as_ref())
    }

    fn spawn_detached<F: 'static + Future<Output = ()>>(&self, future: F) {
        self.spawn(future);
    }
}
