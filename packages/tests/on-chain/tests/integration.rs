use std::sync::Arc;

use app_tests_common::shared_tests::integration::test_integration;
use app_utils::tracing::{env_init, tracing_init};
use on_chain_tests::client::{
    proxy::ProxyClient, service_handler::ServiceHandlerClient, user_registry::UserRegistryClient,
    AppClient,
};

#[tokio::test]
async fn integration() {
    tracing_init();
    env_init();

    let app_client = AppClient::new().await;
    let client = Arc::new(app_client.pool().get().await.unwrap());

    let user_registry = UserRegistryClient::new(client.clone(), None).await;
    let service_handler =
        ServiceHandlerClient::new(client.clone(), user_registry.address, None).await;

    let proxy = ProxyClient::new(client.clone(), Some(vec![service_handler.address.clone()])).await;
    let wrong_proxy = ProxyClient::new(client, Some(vec![service_handler.address.clone()])).await;

    test_integration(service_handler, proxy, wrong_proxy).await;
}
