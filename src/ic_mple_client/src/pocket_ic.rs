use std::sync::Arc;

use candid::utils::ArgumentEncoder;
use candid::{CandidType, Decode, Principal};
use pocket_ic::nonblocking::*;
use serde::de::DeserializeOwned;

use crate::{CanisterClient, CanisterClientResult};

/// A client for interacting with a canister inside dfinity's PocketIc test framework.
#[derive(Clone)]
pub struct PocketIcClient {
    client: Option<Arc<PocketIc>>,
    pub canister: Principal,
    pub caller: Principal,
}

impl PocketIcClient {
    /// Creates a new instance of a PocketIcClient.
    /// The new instance is independent and have no access to canisters of other instances.
    pub async fn new(canister: Principal, caller: Principal) -> Self {
        Self::from_client(PocketIc::new().await, canister, caller)
    }

    /// Crates new instance of PocketIcClient from an existing client instance.
    pub fn from_client<P: Into<Arc<PocketIc>>>(
        client: P,
        canister: Principal,
        caller: Principal,
    ) -> Self {
        Self {
            client: Some(client.into()),
            canister,
            caller,
        }
    }

    /// Returns the PocketIC client for the canister.
    pub fn client(&self) -> &PocketIc {
        self.client
            .as_ref()
            .expect("PocketIC client is not available")
    }

    /// Performs update call with the given arguments.
    pub async fn update<T, R>(&self, method: &str, args: T) -> CanisterClientResult<R>
    where
        T: ArgumentEncoder + Send + Sync,
        R: DeserializeOwned + CandidType,
    {
        let args = candid::encode_args(args)?;

        let call_result = self
            .client()
            .update_call(self.canister, self.caller, method, args)
            .await?;

        let decoded = Decode!(&call_result, R)?;
        Ok(decoded)
    }

    /// Performs query call with the given arguments.
    pub async fn query<T, R>(&self, method: &str, args: T) -> CanisterClientResult<R>
    where
        T: ArgumentEncoder + Send + Sync,
        R: DeserializeOwned + CandidType,
    {
        let args = candid::encode_args(args)?;

        let call_result = self
            .client()
            .query_call(self.canister, self.caller, method, args)
            .await?;

        let decoded = Decode!(&call_result, R)?;
        Ok(decoded)
    }
}

impl CanisterClient for PocketIcClient {
    async fn update<T, R>(&self, method: &str, args: T) -> CanisterClientResult<R>
    where
        T: ArgumentEncoder + Send + Sync,
        R: DeserializeOwned + CandidType + Send,
    {
        PocketIcClient::update(self, method, args).await
    }

    async fn query<T, R>(&self, method: &str, args: T) -> CanisterClientResult<R>
    where
        T: ArgumentEncoder + Send + Sync,
        R: DeserializeOwned + CandidType + Send,
    {
        PocketIcClient::query(self, method, args).await
    }
}
