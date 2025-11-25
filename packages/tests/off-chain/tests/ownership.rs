use app_utils::tracing::tracing_init;
use off_chain_tests::client::{
    proxy::ProxyClient, service_handler::ServiceHandlerClient, AppClient,
};

#[tokio::test]
async fn service_handler_owns_proxy() {
    tracing_init();

    let app_client = AppClient::new("admin");
    let service_handler = ServiceHandlerClient::new(app_client.clone());
    let admins = vec![service_handler.address.clone()];

    let proxy = ProxyClient::new(app_client.clone(), admins.clone());

    let state = proxy.querier.state().await.unwrap();

    assert_eq!(state.admins, admins);
}
