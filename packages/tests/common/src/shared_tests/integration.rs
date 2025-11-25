use app_client::contracts::{proxy::ProxyContract, service_handler::ServiceHandlerContract};
use app_contract_api::{
    proxy::state::{ActionState, State},
    service_handler::msg::Email,
};

pub async fn test_integration(
    service_handler: impl Into<ServiceHandlerContract>,
    proxy: impl Into<ProxyContract>,
) {
    let proxy = proxy.into();
    let service_handler = service_handler.into();

    let state = proxy.querier.state().await.unwrap();

    assert_eq!(
        state,
        State {
            admins: vec![cosmwasm_std::Addr::from(service_handler.address)],
            last_action: ActionState::default(),
        }
    );

    let email = Email {
        from: "joe@example.com".to_string(),
        subject: "hello world!".to_string(),
    };

    service_handler
        .executor
        .push_email(email.clone())
        .await
        .unwrap();

    let state = proxy.querier.state().await.unwrap();

    assert_eq!(state.last_action, ActionState::Forwarded);
}
