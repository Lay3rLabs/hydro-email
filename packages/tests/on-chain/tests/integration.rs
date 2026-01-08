use std::sync::Arc;

use app_tests_common::shared_tests::integration::test_integration;
use app_utils::tracing::{env_init, tracing_init};
use cosmwasm_std::Uint128;
use on_chain_tests::client::{
    control_center::ControlCenterClient, proxy::ProxyClient, service_handler::ServiceHandlerClient,
    user_registry::UserRegistryClient, vault::VaultClient, AppClient,
};

#[tokio::test]
async fn integration() {
    tracing_init();
    env_init();

    let app_client = AppClient::new().await;
    let client = Arc::new(app_client.pool().get().await.unwrap());

    let control_center = ControlCenterClient::new(
        client.clone(),
        vec![client.addr.clone()],
        Uint128::new(1_000_000_000_000),
    )
    .await;

    let vault = VaultClient::new(
        client.clone(),
        control_center.address.clone(),
        "untrn".to_string(),
        vec![client.addr.clone()],
    )
    .await;

    control_center
        .add_subvault(&client, vault.address.clone())
        .await;

    let user_registry = UserRegistryClient::new(client.clone(), None).await;
    let service_handler =
        ServiceHandlerClient::new(client.clone(), user_registry.address, None).await;

    let proxy = ProxyClient::new(
        client.clone(),
        Some(vec![service_handler.address.clone()]),
        vec![control_center.address.clone()],
    )
    .await;
    let wrong_proxy = ProxyClient::new(
        client.clone(),
        Some(vec![service_handler.address.clone()]),
        vec![control_center.address.clone()],
    )
    .await;

    test_integration(service_handler, proxy, wrong_proxy).await;
}
