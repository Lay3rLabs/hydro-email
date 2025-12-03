use cosmwasm_schema::cw_serde;
use cosmwasm_std::Addr;

use crate::user_registry::msg::UserId;

#[cw_serde]
pub struct UserRegisteredEvent {
    pub user_id: UserId,
    pub proxy_address: Addr,
}

impl UserRegisteredEvent {
    pub const EVENT_TYPE: &'static str = "user-registered";
    pub const EVENT_ATTR_KEY_USER_ID: &'static str = "user-id";
    pub const EVENT_ATTR_KEY_PROXY_ADDRESS: &'static str = "proxy-address";
}

impl From<UserRegisteredEvent> for cosmwasm_std::Event {
    fn from(src: UserRegisteredEvent) -> Self {
        cosmwasm_std::Event::new(UserRegisteredEvent::EVENT_TYPE)
            .add_attribute(
                UserRegisteredEvent::EVENT_ATTR_KEY_USER_ID,
                src.user_id.to_string(),
            )
            .add_attribute(
                UserRegisteredEvent::EVENT_ATTR_KEY_PROXY_ADDRESS,
                src.proxy_address.to_string(),
            )
    }
}

impl TryFrom<&cosmwasm_std::Event> for UserRegisteredEvent {
    type Error = anyhow::Error;

    fn try_from(event: &cosmwasm_std::Event) -> Result<Self, Self::Error> {
        if event.ty != Self::EVENT_TYPE && event.ty != format!("wasm-{}", Self::EVENT_TYPE) {
            return Err(anyhow::anyhow!(
                "Expected event type {}, found {}",
                Self::EVENT_TYPE,
                event.ty
            ));
        }

        let mut user_id = None;
        let mut proxy_address = None;

        for attr in event.attributes.iter() {
            match attr.key.as_str() {
                Self::EVENT_ATTR_KEY_USER_ID => {
                    user_id = Some(UserId::new_raw(attr.value.to_string()))
                }
                Self::EVENT_ATTR_KEY_PROXY_ADDRESS => {
                    proxy_address = Some(Addr::unchecked(attr.value.to_string()))
                }
                _ => {}
            }
        }

        match (user_id, proxy_address) {
            (Some(user_id), Some(proxy_address)) => Ok(Self {
                user_id,
                proxy_address,
            }),
            (user_id, proxy_address) => {
                let mut missing_attrs = Vec::new();
                if user_id.is_none() {
                    missing_attrs.push(Self::EVENT_ATTR_KEY_USER_ID);
                }
                if proxy_address.is_none() {
                    missing_attrs.push(Self::EVENT_ATTR_KEY_PROXY_ADDRESS);
                }
                Err(anyhow::anyhow!(
                    "Missing required attributes in UserRegisteredEvent: {:?}",
                    missing_attrs
                ))
            }
        }
    }
}
