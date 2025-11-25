use app_tests_common::shared_tests::integration::test_integration;
use app_utils::tracing::tracing_init;
use off_chain_tests::client::{
    proxy::ProxyClient, service_handler::ServiceHandlerClient, AppClient,
};

#[tokio::test]
async fn integration() {
    tracing_init();

    let app_client = AppClient::new("admin");
    let proxy_code_id = ProxyClient::code_id(&app_client);
    let service_handler = ServiceHandlerClient::new(app_client.clone(), proxy_code_id);
    let proxy = ProxyClient::new(
        app_client.clone(),
        proxy_code_id,
        vec![service_handler.address.clone()],
    );

    test_integration(service_handler, proxy).await;
}
