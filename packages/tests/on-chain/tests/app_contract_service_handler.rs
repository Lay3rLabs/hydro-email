use app_contract_api::service_handler::msg::Email;
use app_utils::tracing::{env_init, tracing_init};
use on_chain_tests::client::{
    proxy::ProxyClient, service_handler::ServiceHandlerClient, AppClient,
};

#[tokio::test]
async fn get_admin() {
    tracing_init();
    env_init();

    let app_client = AppClient::new().await;

    let proxy_creator = app_client.pool().get().await.unwrap();
    let proxy_address = ProxyClient::predict_address(&proxy_creator).await;
    let service_handler = ServiceHandlerClient::new(app_client.clone(), proxy_address, None).await;

    let admin = service_handler.querier.admin().await.unwrap().unwrap();

    app_tests_common::shared_tests::service_handler::get_admin(&service_handler.querier, &admin)
        .await;
}

#[tokio::test]
async fn push_email() {
    tracing_init();
    env_init();

    let app_client = AppClient::new().await;
    let proxy_creator = app_client.pool().get().await.unwrap();
    let proxy_address = ProxyClient::predict_address(&proxy_creator).await;
    let service_handler = ServiceHandlerClient::new(app_client.clone(), proxy_address, None).await;
    ProxyClient::new(proxy_creator, vec![service_handler.address.clone()]).await;

    app_tests_common::shared_tests::service_handler::push_email(
        service_handler,
        Email {
            from: "joe@example.com".to_string(),
            subject: "hello world!".to_string(),
        },
    )
    .await;
}
