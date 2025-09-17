use ic_mple_client::{CanisterClient, CanisterClientResult};


/// Client for the test canister
#[derive(Debug, Clone)]
pub struct TestCanisterClient<C>
where
    C: CanisterClient,
{
    client: C,
}

impl<C: CanisterClient> TestCanisterClient<C> {

    pub fn new(client: C) -> Self {
        Self { client }
    }
    
    pub async fn get_counter(&self) -> CanisterClientResult<u64> {
        self.client.query("get_counter", ()).await
    }

    pub async fn increment_counter(&self, amount: u64) -> CanisterClientResult<()> {
        self.client.update("increment_counter", (amount,)).await
    }

    pub async fn counter_of_other_canister(&self) -> CanisterClientResult<u64> {
        self.client.query("counter_of_other_canister", ()).await
    }
}