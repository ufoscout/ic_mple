use std::sync::Arc;

use candid::{CandidType, Encode, Principal};
use ic_mple_client_integration_tests::InitArgs;
use ic_mple_pocket_ic::get_pocket_ic_client;
use ic_mple_pocket_ic::pocket_ic::nonblocking::PocketIc;

use crate::utils::wasm::get_test_canister_bytecode;

pub struct PocketIcTestContext {
    pub client: Arc<PocketIc>,
    pub canister_a_principal: Principal,
    pub canister_b_principal: Principal,
}

pub async fn with_pocket_ic_context<F, E>(f: F) -> Result<(), E>
where
    F: AsyncFnOnce(&PocketIcTestContext) -> Result<(), E>,
{
    let client: Arc<PocketIc> = Arc::new(get_pocket_ic_client().await.build_async().await);

    let canister_b_args = InitArgs {
        other_canister: None,
    };
    let canister_b_principal =
        deploy_canister(&client, get_test_canister_bytecode(), &canister_b_args).await;

    let canister_a_args = InitArgs {
        other_canister: Some(canister_b_principal),
    };
    let canister_a_principal =
        deploy_canister(&client, get_test_canister_bytecode(), &canister_a_args).await;

    let result = f(&PocketIcTestContext {
        client: client.clone(),
        canister_a_principal,
        canister_b_principal,
    })
    .await;

    if let Ok(client) = Arc::try_unwrap(client) {
        client.drop().await
    }

    result
}

async fn deploy_canister<T: CandidType>(
    client: &PocketIc,
    bytecode: Vec<u8>,
    args: &T,
) -> Principal {
    let args = Encode!(args).unwrap();
    let canister = client.create_canister().await;
    client.add_cycles(canister, 10_u128.pow(12)).await;
    client
        .install_canister(canister, bytecode, args, None)
        .await;
    canister
}
