use app_client::contracts::{
    proxy::ProxyContract, service_handler::ServiceHandlerContract,
    user_registry::UserRegistryContract,
};
use app_contract_api::{
    proxy::state::{ActionState, State},
    service_handler::msg::Email,
};

pub async fn test_integration(
    service_handler: impl Into<ServiceHandlerContract>,
    proxy: impl Into<ProxyContract>,
    wrong_proxy: impl Into<ProxyContract>,
) {
    let service_handler = service_handler.into();
    let proxy = proxy.into();
    let wrong_proxy = wrong_proxy.into();

    let user_registry = UserRegistryContract::new(
        service_handler.querier.inner.clone(),
        service_handler.executor.inner.clone(),
        service_handler
            .querier
            .user_registry_address()
            .await
            .unwrap(),
    );

    let email = Email {
        from: "alice@example.com".to_string(),
        subject: "hello world!".to_string(),
    };

    // An email from a different user to verify we get the right proxy address
    let not_this_email = Email {
        from: "bob@example.com".to_string(),
        subject: "hello world!".to_string(),
    };

    // no proxy address yet
    user_registry
        .querier
        .proxy_address_email(&email.from)
        .await
        .unwrap_err();

    // register the user's proxy address
    user_registry
        .executor
        .register_user_email(&email.from, proxy.address.clone())
        .await
        .unwrap();

    // register a different user's proxy address to verify we get the right one
    user_registry
        .executor
        .register_user_email(&not_this_email.from, wrong_proxy.address.clone())
        .await
        .unwrap();

    // got the right one
    assert_eq!(
        user_registry
            .querier
            .proxy_address_email(&email.from)
            .await
            .unwrap(),
        proxy.address
    );

    // Proxy is idle at first
    let state = proxy.querier.state().await.unwrap();

    assert_eq!(
        state,
        State {
            admins: vec![cosmwasm_std::Addr::from(service_handler.address)],
            last_action: ActionState::Idle
        }
    );

    // Send an email through the service handler
    service_handler
        .executor
        .push_email(email.clone())
        .await
        .unwrap();

    // Service handler should get the right proxy address and update the state
    let state = proxy.querier.state().await.unwrap();
    assert_eq!(state.last_action, ActionState::Forwarded);

    // wrong proxy is still idle
    let state = wrong_proxy.querier.state().await.unwrap();
    assert_eq!(state.last_action, ActionState::Idle);
}
