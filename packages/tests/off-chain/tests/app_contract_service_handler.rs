use app_contract_api::service_handler::msg::Email;
use app_utils::tracing::tracing_init;
use off_chain_tests::client::proxy::ProxyClient;
use off_chain_tests::client::service_handler::ServiceHandlerClient;
use off_chain_tests::client::AppClient;

#[tokio::test]
async fn get_admin() {
    tracing_init();

    let app_client = AppClient::new("admin");
    let proxy_code_id = ProxyClient::code_id(&app_client);
    let service_handler = ServiceHandlerClient::new(app_client.clone(), proxy_code_id);

    let admin = app_client.admin().to_string();

    app_tests_common::shared_tests::service_handler::get_admin(&service_handler.querier, &admin)
        .await;
}

#[tokio::test]
async fn pushes_email() {
    tracing_init();

    let app_client = AppClient::new("admin");
    let proxy_code_id = ProxyClient::code_id(&app_client);
    let service_handler = ServiceHandlerClient::new(app_client.clone(), proxy_code_id);
    ProxyClient::new(
        app_client,
        proxy_code_id,
        vec![service_handler.address.clone()],
    );

    app_tests_common::shared_tests::service_handler::push_email(
        service_handler,
        Email {
            from: "joe@example.com".to_string(),
            subject: "hello world!".to_string(),
        },
    )
    .await;
}
