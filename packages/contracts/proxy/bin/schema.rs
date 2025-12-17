use app_contract_api::proxy::msg::QueryMsg;
use cosmwasm_schema::write_api;
use hydro_proxy::msg::{ExecuteMsg, InstantiateMsg};

fn main() {
    write_api! {
        instantiate: InstantiateMsg,
        execute: ExecuteMsg,
        query: QueryMsg,
    };
}
