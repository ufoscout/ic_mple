use std::time::SystemTime;

use candid::Principal;
use ic_cdk::{api::{canister_cycle_balance, canister_self}, futures::spawn};

#[cfg(all(not(target_family = "wasm"), feature = "tokio"))]
pub mod tokio_local;

const E_9: u64 = 1_000_000_000;

/// Returns the IC API, or the TokioIcApi if non in wasm and the tokio feature is enabled
pub fn ic() -> IcApi {
    IcApi::default()
}

#[cfg(target_family = "wasm")]
pub type IcApi = IcPlatform;

#[cfg(all(not(target_family = "wasm"), feature = "tokio"))]
pub type IcApi = crate::ic_api::tokio_local::TokioIcApi;

#[cfg(all(not(target_family = "wasm"), not(feature = "tokio")))]
pub type IcApi = IcPlatform;

/// A wrapper trait for the IC API.
/// It allows us to use a mock or non-wasm-based implementation.
pub trait IcTrait {

    /// Gets canister's own identity.
    fn canister_self(&self) -> Principal;

    /// Gets the current cycle balance of the canister.
    fn canister_cycle_balance(&self) -> u128;

    /// Gets current timestamp, in nanoseconds since the epoch (1970-01-01)
    fn time_ns(&self) -> u64;

    /// Gets current timestamp, in seconds since the epoch (1970-01-01)
    fn time_secs(&self) -> u64;

    /// Returns the current SystemTime
    fn current_system_time(&self) -> SystemTime;

    /// Spawn an asynchronous task to run in the background.
    fn spawn<F: 'static + Future<Output = ()>>(&self, future: F);

    fn print<S: std::convert::AsRef<str>>(&self, s: S);

}

/// The default implementation of the IC API
#[derive(Clone, Debug, Default)]
pub struct IcPlatform;

impl IcTrait for IcPlatform {

    fn canister_self(&self) -> Principal {
        canister_self()
    }

    fn time_ns(&self) -> u64 {
        ic_cdk::api::time()
    }

    fn time_secs(&self) -> u64 {
        ic_cdk::api::time() / E_9
    }

    fn spawn<F: 'static + Future<Output = ()>>(&self, future: F) {
        spawn(future)
    }
    
    fn canister_cycle_balance(&self) -> u128 {
        canister_cycle_balance()
    }
    
    fn current_system_time(&self) -> SystemTime {
        let timestamp_in_nanos = self.time_ns();
        std::time::SystemTime::UNIX_EPOCH + std::time::Duration::from_nanos(timestamp_in_nanos)
    }

    fn print<S: std::convert::AsRef<str>>(&self, s: S) {
        ic_cdk::api::debug_print(s)
    }

}