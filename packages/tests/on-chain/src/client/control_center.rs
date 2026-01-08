use app_client::executor::SigningClientWrapper;
use cosmwasm_std::Uint128;
use hydro_interface::inflow_control_center::ExecuteMsg;
use layer_climb::prelude::*;

use crate::code_ids::CodeId;

// Local InstantiateMsg since the hydro control-center msg.rs is not exposed via interface
#[cosmwasm_schema::cw_serde]
pub struct InstantiateMsg {
    pub deposit_cap: Uint128,
    pub whitelist: Vec<String>,
    pub subvaults: Vec<String>,
}

#[derive(Clone)]
pub struct ControlCenterClient {
    pub address: Address,
}

impl ControlCenterClient {
    pub async fn new(
        client: SigningClientWrapper,
        whitelist: Vec<Address>,
        deposit_cap: Uint128,
    ) -> Self {
        let msg = InstantiateMsg {
            deposit_cap,
            whitelist: whitelist.iter().map(|a| a.to_string()).collect(),
            subvaults: vec![],
        };

        let (address, _) = client
            .contract_instantiate(
                None,
                CodeId::new_control_center().await,
                "Control Center",
                &msg,
                vec![],
                None,
            )
            .await
            .unwrap();

        Self { address }
    }

    pub async fn add_subvault(&self, client: &SigningClientWrapper, vault: Address) {
        let msg = ExecuteMsg::AddSubvault {
            address: vault.to_string(),
        };

        client
            .contract_execute(&self.address, &msg, vec![])
            .await
            .unwrap();
    }
}
