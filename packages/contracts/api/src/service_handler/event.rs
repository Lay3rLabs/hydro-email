use cosmwasm_schema::cw_serde;

use crate::service_handler::msg::Email;

#[cw_serde]
pub struct EmailEvent {
    pub email: Email,
    pub pagination_id: u64,
}

impl EmailEvent {
    pub const EVENT_TYPE: &'static str = "email";
    pub const EVENT_ATTR_KEY_EMAIL_FROM: &'static str = "email-from";
    pub const EVENT_ATTR_KEY_EMAIL_SUBJECT: &'static str = "email-subject";
    pub const EVENT_ATTR_KEY_PAGINATION_ID: &'static str = "pagination-id";
}

impl From<EmailEvent> for cosmwasm_std::Event {
    fn from(src: EmailEvent) -> Self {
        cosmwasm_std::Event::new(EmailEvent::EVENT_TYPE)
            .add_attribute(EmailEvent::EVENT_ATTR_KEY_EMAIL_FROM, src.email.from)
            .add_attribute(EmailEvent::EVENT_ATTR_KEY_EMAIL_SUBJECT, src.email.subject)
            .add_attribute(
                EmailEvent::EVENT_ATTR_KEY_PAGINATION_ID,
                src.pagination_id.to_string(),
            )
    }
}

impl TryFrom<&cosmwasm_std::Event> for EmailEvent {
    type Error = anyhow::Error;

    fn try_from(event: &cosmwasm_std::Event) -> Result<Self, Self::Error> {
        if event.ty != Self::EVENT_TYPE && event.ty != format!("wasm-{}", Self::EVENT_TYPE) {
            return Err(anyhow::anyhow!(
                "Expected event type {}, found {}",
                Self::EVENT_TYPE,
                event.ty
            ));
        }

        let mut email_from = None;
        let mut email_subject = None;
        let mut pagination_id = None;

        for attr in event.attributes.iter() {
            match attr.key.as_str() {
                Self::EVENT_ATTR_KEY_EMAIL_FROM => email_from = Some(attr.value.to_string()),
                Self::EVENT_ATTR_KEY_EMAIL_SUBJECT => email_subject = Some(attr.value.to_string()),
                Self::EVENT_ATTR_KEY_PAGINATION_ID => {
                    pagination_id = Some(attr.value.parse::<u64>()?)
                }
                _ => {}
            }
        }

        match (email_from, email_subject, pagination_id) {
            (Some(from), Some(subject), Some(id)) => Ok(Self {
                email: Email { from, subject },
                pagination_id: id,
            }),
            (from, subject, id) => {
                let mut missing_attrs = Vec::new();
                if from.is_none() {
                    missing_attrs.push(Self::EVENT_ATTR_KEY_EMAIL_FROM);
                }
                if subject.is_none() {
                    missing_attrs.push(Self::EVENT_ATTR_KEY_EMAIL_SUBJECT);
                }
                if id.is_none() {
                    missing_attrs.push(Self::EVENT_ATTR_KEY_PAGINATION_ID);
                }
                Err(anyhow::anyhow!(
                    "Missing required attributes in EmailEvent: {:?}",
                    missing_attrs
                ))
            }
        }
    }
}
