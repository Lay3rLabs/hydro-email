use cosmwasm_std::{Coin, Uint128};
use hydro_proxy::msg::ExecuteMsg as ProxyExecuteMsg;

/// Formats:
/// - "forward" or "deposit" or empty → ForwardToInflow
/// - "withdraw ADDRESS DENOM AMOUNT" → WithdrawFunds
/// - "withdraw_receipt ADDRESS DENOM AMOUNT" → WithdrawReceiptTokens
pub fn parse_email_action(subject: &str) -> ProxyExecuteMsg {
    let text = subject.trim().to_lowercase();

    if text.is_empty() || text == "forward" || text == "deposit" {
        return ProxyExecuteMsg::ForwardToInflow {};
    }

    // "withdraw ADDRESS DENOM AMOUNT"
    if let Some(rest) = text.strip_prefix("withdraw ") {
        if let Some(msg) = parse_withdraw(rest, false) {
            return msg;
        }
    }

    // "withdraw_receipt ADDRESS DENOM AMOUNT"
    if let Some(rest) = text.strip_prefix("withdraw_receipt ") {
        if let Some(msg) = parse_withdraw(rest, true) {
            return msg;
        }
    }

    ProxyExecuteMsg::ForwardToInflow {}
}

fn parse_withdraw(rest: &str, is_receipt: bool) -> Option<ProxyExecuteMsg> {
    let parts: Vec<&str> = rest.split_whitespace().collect();
    if parts.len() >= 3 {
        let address = parts[0].to_string();
        let denom = parts[1].to_string();
        let amount: Uint128 = parts[2].parse().ok()?;
        let coin = Coin {
            denom,
            amount: amount.into(),
        };

        return Some(if is_receipt {
            ProxyExecuteMsg::WithdrawReceiptTokens { address, coin }
        } else {
            ProxyExecuteMsg::WithdrawFunds { address, coin }
        });
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;
    use cosmwasm_std::Uint256;

    #[test]
    fn test_forward_variants() {
        assert!(matches!(
            parse_email_action(""),
            ProxyExecuteMsg::ForwardToInflow {}
        ));
        assert!(matches!(
            parse_email_action("forward"),
            ProxyExecuteMsg::ForwardToInflow {}
        ));
        assert!(matches!(
            parse_email_action("FORWARD"),
            ProxyExecuteMsg::ForwardToInflow {}
        ));
        assert!(matches!(
            parse_email_action("deposit"),
            ProxyExecuteMsg::ForwardToInflow {}
        ));
    }

    #[test]
    fn test_withdraw_funds() {
        let msg = parse_email_action("withdraw neutron1abc uatom 1000000");
        match msg {
            ProxyExecuteMsg::WithdrawFunds { address, coin } => {
                assert_eq!(address, "neutron1abc");
                assert_eq!(coin.denom, "uatom");
                assert_eq!(coin.amount, Uint256::from(1000000u128));
            }
            _ => panic!("expected WithdrawFunds"),
        }
    }

    #[test]
    fn test_withdraw_receipt() {
        let msg = parse_email_action("withdraw_receipt neutron1xyz factory/vault/share 500000");
        match msg {
            ProxyExecuteMsg::WithdrawReceiptTokens { address, coin } => {
                assert_eq!(address, "neutron1xyz");
                assert_eq!(coin.denom, "factory/vault/share");
                assert_eq!(coin.amount, Uint256::from(500000u128));
            }
            _ => panic!("expected WithdrawReceiptTokens"),
        }
    }

    #[test]
    fn test_invalid_falls_back_to_forward() {
        assert!(matches!(
            parse_email_action("withdraw"),
            ProxyExecuteMsg::ForwardToInflow {}
        ));
        assert!(matches!(
            parse_email_action("withdraw addr"),
            ProxyExecuteMsg::ForwardToInflow {}
        ));
        assert!(matches!(
            parse_email_action("withdraw addr denom notanumber"),
            ProxyExecuteMsg::ForwardToInflow {}
        ));
        assert!(matches!(
            parse_email_action("random subject"),
            ProxyExecuteMsg::ForwardToInflow {}
        ));
    }
}
