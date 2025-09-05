use candid::utils::ArgumentEncoder;
use candid::{CandidType, Principal};
use serde::de::DeserializeOwned;

use crate::client::CanisterClient;
use crate::{CanisterClientError, CanisterClientResult};

/// This client is used to interact with the IC canister.
#[derive(Debug, Clone)]
pub struct IcCanisterClient {
    /// The canister id of the Evm canister
    pub canister_id: Principal,
    // the call timeout
    timeout_seconds: Option<u32>,
}

impl IcCanisterClient {
    /// Creates a new instance of `IcCanisterClient`.
    ///
    /// # Parameters
    /// - `canister`: The Principal of the canister to interact with.
    /// - `timeout_seconds`: The timeout in seconds for calls to the canister.
    ///   If `Some`, a bounded call with the specified timeout will be used.
    ///   If `None`, an unbounded call will be used.
    pub fn new(canister: Principal, timeout_seconds: Option<u32>) -> Self {
        Self {
            canister_id: canister,
            timeout_seconds,
        }
    }

    async fn call<T, R>(&self, method: &str, args: T) -> CanisterClientResult<R>
    where
        T: ArgumentEncoder + Send,
        R: DeserializeOwned + CandidType,
    {
        let call = if let Some(timeout_seconds) = self.timeout_seconds {
            ic_cdk::call::Call::bounded_wait(self.canister_id, method)
                .change_timeout(timeout_seconds)
                .with_args(&args)
        } else {
            ic_cdk::call::Call::unbounded_wait(self.canister_id, method).with_args(&args)
        };

        let call_result = call
            .await
            .map_err(|e| CanisterClientError::CanisterError(e.into()))?
            .into_bytes();

        use candid::Decode;
        Decode!(&call_result, R).map_err(CanisterClientError::CandidError)
    }
}

impl CanisterClient for IcCanisterClient {
    async fn update<T, R>(&self, method: &str, args: T) -> CanisterClientResult<R>
    where
        T: ArgumentEncoder + Send + Sync,
        R: DeserializeOwned + CandidType + Send,
    {
        self.call(method, args).await
    }

    async fn query<T, R>(&self, method: &str, args: T) -> CanisterClientResult<R>
    where
        T: ArgumentEncoder + Send + Sync,
        R: DeserializeOwned + CandidType + Send,
    {
        self.call(method, args).await
    }
}
