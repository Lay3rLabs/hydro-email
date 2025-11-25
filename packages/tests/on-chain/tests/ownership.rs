use app_utils::tracing::tracing_init;
use on_chain_tests::client::{
    proxy::ProxyClient, service_handler::ServiceHandlerClient, AppClient,
};

#[tokio::test]
async fn service_handler_owns_proxy() {
    tracing_init();

    let app_client = AppClient::new().await;
    let service_handler = ServiceHandlerClient::new(app_client.clone(), None).await;

    let proxy = ProxyClient::new(app_client.clone(), vec![service_handler.address.clone()]).await;

    let state = proxy.querier.state().await.unwrap();

    assert_eq!(
        state.admins,
        vec![cosmwasm_std::Addr::try_from(service_handler.address).unwrap()]
    );
}
