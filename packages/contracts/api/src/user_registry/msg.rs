use cosmwasm_schema::{cw_serde, QueryResponses};
use cosmwasm_std::Addr;
use cw_storage_plus::NewTypeKey;
use sha2::{Digest, Sha256};

#[cw_serde]
pub struct InstantiateMsg {
    pub admins: Vec<String>,
}

#[cw_serde]
pub enum ExecuteMsg {
    RegisterUser {
        user_id: UserId,
        proxy_address: String,
    },
}

#[cw_serde]
#[derive(QueryResponses)]
pub enum QueryMsg {
    #[returns(ProxyAddressResponse)]
    ProxyAddress { user_id: UserId },
}

#[cw_serde]
pub struct ProxyAddressResponse {
    pub address: Addr,
}

#[cw_serde]
#[derive(NewTypeKey)]
pub struct UserId(String);

impl std::fmt::Display for UserId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl UserId {
    pub const SALT: [u8; 12] = *b"hydro-email!";

    pub fn new_raw(user_id: String) -> Self {
        Self(user_id)
    }

    pub fn new_email_address(email: &str) -> Self {
        let email = match extract_email(email) {
            Some(e) => e,
            None => email.to_string(),
        };

        let mut hasher = Sha256::new();
        hasher.update(&email);
        hasher.update(&Self::SALT);

        Self(const_hex::encode(hasher.finalize()))
    }
}

fn extract_email(input: &str) -> Option<String> {
    let parsed = mailparse::addrparse(input).ok()?;
    // addrparse returns a vector; usually you want the first entry
    let addr = parsed.get(0)?;

    match addr {
        mailparse::MailAddr::Single(single) => Some(single.addr.clone()),
        mailparse::MailAddr::Group(group) => {
            // Rare, but groups exist in RFC 5322
            group.addrs.first().map(|a| a.addr.clone())
        }
    }
}
