use app_client::contracts::service_handler::{ServiceHandlerContract, ServiceHandlerQuerier};
use app_contract_api::service_handler::{event::EmailEvent, msg::Email};
use layer_climb::events::CosmosTxEvents;

pub async fn get_admin(querier: &ServiceHandlerQuerier, expected: &str) {
    let admin = querier.admin().await.unwrap().unwrap();
    assert_eq!(admin, expected);
}

pub async fn push_email(service_handler: impl Into<ServiceHandlerContract>, email: Email) {
    let ServiceHandlerContract {
        querier, executor, ..
    } = service_handler.into();
    // Register user to receive payments
    let response = executor.push_email(email.clone()).await.unwrap();

    let event = {
        let events = CosmosTxEvents::from(&response);
        let event = events.event_first_by_type(EmailEvent::EVENT_TYPE).unwrap();
        EmailEvent::try_from(&cosmwasm_std::Event::from(event)).unwrap()
    };

    let emails = querier.all_emails_from(&email.from).await.unwrap();

    let found_email = emails
        .into_iter()
        .find_map(|(email, id)| {
            if id == event.pagination_id {
                Some(email)
            } else {
                None
            }
        })
        .expect("Pushed email not found in query results");

    assert_eq!(found_email.subject, email.subject);
}
