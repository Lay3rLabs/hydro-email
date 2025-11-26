use app_tests_common::shared_tests::integration::test_integration;
use app_utils::tracing::{env_init, tracing_init};
use on_chain_tests::client::{
    proxy::ProxyClient, service_handler::ServiceHandlerClient, AppClient,
};

#[tokio::test]
async fn integration() {
    tracing_init();
    env_init();

    let app_client = AppClient::new().await;
    let proxy_creator = app_client.pool().get().await.unwrap();
    let proxy_address = ProxyClient::predict_address(&proxy_creator).await;
    let service_handler = ServiceHandlerClient::new(app_client.clone(), proxy_address, None).await;
    let proxy = ProxyClient::new(proxy_creator, vec![service_handler.address.clone()]).await;

    test_integration(service_handler, proxy).await;
}
