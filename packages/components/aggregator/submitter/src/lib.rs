use layer_climb::prelude::CosmosAddr;

use crate::wavs::aggregator::aggregator::{CosmosAddress, CosmosSubmitAction, SubmitAction};

wit_bindgen::generate!({
    path: "../../../../wit-definitions/aggregator/wit",
    world: "aggregator-world",
    generate_all,
    with: {
        "wasi:io/poll@0.2.0": wasip2::io::poll
    },
    features: ["tls"]
});

struct Component;

impl Guest for Component {
    fn process_packet(_packet: Packet) -> Result<Vec<AggregatorAction>, String> {
        let chain = host::config_var("CHAIN").ok_or("CHAIN config var is required")?;
        let service_handler_addr = host::config_var("SERVICE_HANDLER_CONTRACT_ADDRESS")
            .ok_or("SERVICE_HANDLER_CONTRACT_ADDRESS config var is required")?;

        host::get_cosmos_chain_config(&chain)
            .ok_or(format!("failed to get chain config for {}", chain))?;

        let service_handler_addr =
            CosmosAddr::new_str(&service_handler_addr, None).map_err(|e| e.to_string())?;

        Ok(vec![AggregatorAction::Submit(SubmitAction::Cosmos(
            CosmosSubmitAction {
                chain: chain.to_string(),
                address: CosmosAddress {
                    bech32_addr: service_handler_addr.to_string(),
                    prefix_len: service_handler_addr.prefix().len() as u32,
                },
                gas_price: None,
            },
        ))])
    }

    fn handle_timer_callback(_packet: Packet) -> Result<Vec<AggregatorAction>, String> {
        Ok(vec![])
    }

    fn handle_submit_callback(
        _packet: Packet,
        _tx_result: Result<AnyTxHash, String>,
    ) -> Result<(), String> {
        Ok(())
    }
}

export!(Component);
