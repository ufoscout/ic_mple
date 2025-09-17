#![cfg(feature = "ic-agent")]

use candid::Principal;
use ic_mple_client::ic_agent::{agent::AgentBuilder, export::reqwest::Url};
use ic_mple_client::{CanisterClient, CanisterClientResult, IcAgentClient};

// Address of ckUSDC canister
const CKUSDC_ADDRESS: &str = "xevnm-gaaaa-aaaar-qafnq-cai";

#[derive(Debug, Clone)]
pub struct CkUsdcClient<C>
where
    C: CanisterClient,
{
    client: C,
}

impl<C: CanisterClient> CkUsdcClient<C> {
    pub async fn icrc1_symbol(&self) -> CanisterClientResult<String> {
        self.client.query("icrc1_symbol", ()).await
    }
}

#[tokio::test]
async fn agent_client_should_call_the_ckusdc_canister() {
    // Arrange
    let url = Url::parse("https://ic0.app").unwrap();
    let agent = AgentBuilder::default().with_url(url).build().unwrap();
    let client = IcAgentClient::with_agent(Principal::from_text(CKUSDC_ADDRESS).unwrap(), agent);
    let ckusdc_client = CkUsdcClient { client };

    // Act
    let symbol = ckusdc_client.icrc1_symbol().await.unwrap();

    // Assert
    assert_eq!(symbol, "ckUSDC");
}
