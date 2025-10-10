use std::time::SystemTime;

use candid::{CandidType, Principal};
use ic_cdk::{
    api::{canister_cycle_balance, canister_self},
    futures::spawn,
};
use serde::Deserialize;

pub mod mock;

const E_9: u64 = 1_000_000_000;

/// Returns the IC API, or the TokioIcApi if non in wasm and the ic_mock feature is enabled
pub fn ic() -> IcApi {
    IcApi::default()
}

#[cfg(target_family = "wasm")]
pub type IcApi = IcPlatform;

#[cfg(not(target_family = "wasm"))]
pub type IcApi = crate::ic_api::mock::IcMock;

/// A wrapper trait for the IC API.
/// It allows us to use a mock or non-wasm-based implementation.
pub trait IcTrait: Clone {
    /// Gets canister's own identity.
    fn canister_self(&self) -> Principal;

    /// Gets the current cycle balance of the canister.
    fn canister_cycle_balance(&self) -> u128;

    /// Gets current timestamp, in nanoseconds since the epoch (1970-01-01)
    fn time_nanos(&self) -> u64;

    /// Gets current timestamp, in seconds since the epoch (1970-01-01)
    fn time_secs(&self) -> u64 { self.time_nanos() / E_9 }

    /// Returns the current SystemTime
    fn current_system_time(&self) -> SystemTime {
        let timestamp_in_nanos = self.time_nanos();
        std::time::SystemTime::UNIX_EPOCH + std::time::Duration::from_nanos(timestamp_in_nanos)
    }

    /// Spawn an asynchronous task to run in the background.
    fn spawn<F: 'static + Future<Output = ()>>(&self, future: F);

    /// Spawn an asynchronous task to run in the background.
    /// If this task panicks it does not cause the launching task to be rolled back by IC.
    /// This is achieved by executing the task in zero-delayed dedicated timer
    fn spawn_detached<F: 'static + Future<Output = ()>>(&self, future: F);

    fn print<S: std::convert::AsRef<str>>(&self, s: S);
}

/// The default implementation of the IC API
#[derive(Clone, Debug, Default, CandidType, Deserialize, PartialEq, Eq)]
pub struct IcPlatform;

impl IcTrait for IcPlatform {
    fn canister_self(&self) -> Principal {
        canister_self()
    }

    fn time_nanos(&self) -> u64 {
        ic_cdk::api::time()
    }

    fn spawn<F: 'static + Future<Output = ()>>(&self, future: F) {
        spawn(future)
    }

    fn canister_cycle_balance(&self) -> u128 {
        canister_cycle_balance()
    }

    fn print<S: std::convert::AsRef<str>>(&self, s: S) {
        ic_cdk::api::debug_print(s)
    }
    
    fn spawn_detached<F: 'static + Future<Output = ()>>(&self, future: F) {
        ic_cdk_timers::set_timer(std::time::Duration::from_millis(0), || {
            spawn(future);
        });
    }
}
