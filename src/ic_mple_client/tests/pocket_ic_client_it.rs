#![cfg(feature = "pocket-ic")]

use candid::Principal;
use ic_mple_client::PocketIcClient;
use test_canister::client::TestCanisterClient;
use utils::pocket_ic_test_context::with_pocket_ic_context;

mod utils;

#[tokio::test]
async fn ic_mple_client_should_call_query_endpoint() {
    with_pocket_ic_context::<_, ()>(async move |ctx| {
        // Arrange
        let client = PocketIcClient::from_client(
            ctx.client.clone(),
            ctx.canister_a_principal,
            Principal::anonymous(),
        );
        let client = TestCanisterClient::new(client);

        // Act
        let counter = client.get_counter().await.unwrap();

        // Assert
        assert_eq!(counter, 0);

        Ok(())
    })
    .await
    .unwrap();
}

#[tokio::test]
async fn ic_mple_client_should_call_update_endpoint() {
    with_pocket_ic_context::<_, ()>(async move |ctx| {
        // Arrange
        let client = PocketIcClient::from_client(
            ctx.client.clone(),
            ctx.canister_a_principal,
            Principal::anonymous(),
        );
        let client = TestCanisterClient::new(client);

        // Act
        let counter_0 = client.get_counter().await.unwrap();
        client.increment_counter(10).await.unwrap();
        let counter_1 = client.get_counter().await.unwrap();
        client.increment_counter(11).await.unwrap();
        let counter_2 = client.get_counter().await.unwrap();

        // Assert
        assert_eq!(counter_0, 0);
        assert_eq!(counter_1, 10);
        assert_eq!(counter_2, 21);

        Ok(())
    })
    .await
    .unwrap();
}

#[tokio::test]
async fn ic_mple_client_should_perform_an_intercanister_call() {
    with_pocket_ic_context::<_, ()>(async move |ctx| {
        // Arrange
        let client_a = TestCanisterClient::new(PocketIcClient::from_client(
            ctx.client.clone(),
            ctx.canister_a_principal,
            Principal::anonymous(),
        ));
        let client_b = TestCanisterClient::new(PocketIcClient::from_client(
            ctx.client.clone(),
            ctx.canister_b_principal,
            Principal::anonymous(),
        ));

        // Act
        let counter_a_0 = client_a.get_counter().await.unwrap();
        let counter_other_0 = client_a.counter_of_other_canister().await.unwrap();
        client_b.increment_counter(10).await.unwrap();
        let counter_a_1 = client_a.get_counter().await.unwrap();
        let counter_other_1 = client_a.counter_of_other_canister().await.unwrap();

        // Assert
        assert_eq!(counter_a_0, 0);
        assert_eq!(counter_other_0, 0);
        assert_eq!(counter_a_1, 0);
        assert_eq!(counter_other_1, 10);

        Ok(())
    })
    .await
    .unwrap();
}
