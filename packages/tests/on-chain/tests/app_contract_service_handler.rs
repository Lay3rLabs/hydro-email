use app_contract_api::service_handler::msg::Email;
use app_utils::tracing::tracing_init;
use on_chain_tests::client::{service_handler::ServiceHandlerClient, AppClient};

#[tokio::test]
async fn get_admin() {
    tracing_init();

    let app_client = AppClient::new().await;
    let service_handler = ServiceHandlerClient::new(app_client.clone(), None).await;

    let admin = service_handler.querier.admin().await.unwrap().unwrap();

    app_tests_common::shared_tests::service_handler::get_admin(&service_handler.querier, &admin)
        .await;
}

#[tokio::test]
async fn push_email() {
    tracing_init();

    let app_client = AppClient::new().await;
    let service_handler = ServiceHandlerClient::new(app_client.clone(), None).await;

    app_tests_common::shared_tests::service_handler::push_email(
        &service_handler.querier,
        &service_handler.executor,
        Email {
            from: "joe@example.com".to_string(),
            subject: "hello world!".to_string(),
        },
    )
    .await;
}
