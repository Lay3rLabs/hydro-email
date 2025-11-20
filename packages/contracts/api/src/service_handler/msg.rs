use cosmwasm_schema::{cw_serde, QueryResponses};
use wavs_types::contracts::cosmwasm::service_handler::{
    ServiceHandlerExecuteMessages, ServiceHandlerQueryMessages,
};

#[cw_serde]
pub struct InstantiateMsg {
    pub auth: Auth,
}

#[cw_serde]
pub enum Auth {
    /// Implement ServiceHandler interface, validate signatures with the ServiceManager
    ServiceManager(String),
    /// Used for tests. One account is authorized to execute the privileged methods normally reserved for WAVS
    Admin(String),
}

#[cw_serde]
#[schemaifier(mute_warnings)]
#[derive(QueryResponses)]
#[query_responses(nested)]
#[serde(untagged)]
pub enum QueryMsg {
    Custom(CustomQueryMsg),
    Wavs(ServiceHandlerQueryMessages),
}

#[cw_serde]
#[derive(QueryResponses)]
pub enum CustomQueryMsg {
    #[returns(EmailAddrsResponse)]
    EmailAddrs {
        /// Max number of emails to return
        limit: Option<u32>,
        /// Optional exclusive start of the range (last key from previous page)
        start_after: Option<String>,
    },
    #[returns(EmailsFromResponse)]
    EmailsFrom {
        /// Email address
        from: String,
        /// Max number of emails to return
        limit: Option<u32>,
        /// Optional exclusive start of the range (last key from previous page)
        start_after: Option<u64>,
    },
    #[returns(EmailsResponse)]
    Emails {
        /// Max number of emails to return
        limit: Option<u32>,
        /// Optional exclusive start of the range (last key from previous page)
        start_after: Option<u64>,
    },
    #[returns(AdminResponse)]
    Admin {},
}

#[cw_serde]
pub struct EmailsFromResponse {
    /// List of (email, pagination key)
    pub emails: Vec<(EmailMessageOnly, u64)>,
}

#[cw_serde]
pub struct EmailAddrsResponse {
    /// List of email addresses
    pub email_addrs: Vec<String>,
}

#[cw_serde]
pub struct EmailsResponse {
    /// List of (email, pagination key)
    pub emails: Vec<(Email, u64)>,
}

#[cw_serde]
pub struct EmailMessageOnly {
    pub subject: String,
}

impl From<&Email> for EmailMessageOnly {
    fn from(src: &Email) -> Self {
        Self {
            subject: src.subject.clone(),
        }
    }
}

#[cw_serde]
pub struct Email {
    pub from: String,
    pub subject: String,
}

#[cw_serde]
#[schemaifier(mute_warnings)]
#[serde(untagged)]
pub enum ExecuteMsg {
    Custom(CustomExecuteMsg),
    Wavs(ServiceHandlerExecuteMessages),
}

#[cw_serde]
pub enum CustomExecuteMsg {
    /// Got an email
    Email(Email),
}

impl CustomExecuteMsg {
    pub fn encode(&self) -> cosmwasm_std::StdResult<Vec<u8>> {
        cosmwasm_std::to_json_vec(self)
    }

    pub fn decode(bytes: impl AsRef<[u8]>) -> cosmwasm_std::StdResult<Self> {
        cosmwasm_std::from_json(bytes)
    }
}

#[cw_serde]
pub struct AdminResponse {
    pub admin: Option<String>,
}

#[cw_serde]
pub struct MigrateMsg {}
