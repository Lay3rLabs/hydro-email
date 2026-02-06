//! Local proxy contract message types and utilities.
//!
//! These types are duplicated from `hydro_proxy::msg` rather than imported because
//! the hydro submodule uses cosmwasm_std 2.x while this crate uses cosmwasm_std 3.x.
//! The Rust type system treats types from different crate versions as incompatible,
//! even if they have identical definitions. Since the JSON serialization format is
//! the same between versions, we can safely duplicate the types here and they will
//! serialize correctly when sent to the proxy contract.

use cosmwasm_schema::cw_serde;
use cosmwasm_std::{Coin, Uint128};

/// Execute messages for the proxy contract.
/// Mirrors `hydro_proxy::msg::ExecuteMsg`.
#[cw_serde]
pub enum ProxyExecuteMsg {
    ForwardToInflow {},
    WithdrawReceiptTokens { address: String, coin: Coin },
    WithdrawFunds { address: String, coin: Coin },
}

impl ProxyExecuteMsg {
    /// Convert to email subject string.
    /// - ForwardToInflow -> "deposit"
    /// - WithdrawFunds -> "withdraw ADDRESS DENOM AMOUNT"
    /// - WithdrawReceiptTokens -> "withdraw_receipt ADDRESS DENOM AMOUNT"
    pub fn to_email_subject(&self) -> String {
        match self {
            ProxyExecuteMsg::ForwardToInflow {} => "deposit".to_string(),
            ProxyExecuteMsg::WithdrawFunds { address, coin } => {
                format!("withdraw {} {} {}", address, coin.denom, coin.amount)
            }
            ProxyExecuteMsg::WithdrawReceiptTokens { address, coin } => {
                format!(
                    "withdraw_receipt {} {} {}",
                    address, coin.denom, coin.amount
                )
            }
        }
    }

    /// Parse email subject to ProxyExecuteMsg.
    ///
    /// Formats:
    /// - "forward" or "deposit" or empty -> ForwardToInflow
    /// - "withdraw ADDRESS DENOM AMOUNT" -> WithdrawFunds
    /// - "withdraw_receipt ADDRESS DENOM AMOUNT" -> WithdrawReceiptTokens
    ///
    /// Invalid formats fall back to ForwardToInflow.
    pub fn from_email_subject(subject: &str) -> Self {
        let text = subject.trim().to_lowercase();

        if text.is_empty() || text == "forward" || text == "deposit" {
            return ProxyExecuteMsg::ForwardToInflow {};
        }

        if let Some(rest) = text.strip_prefix("withdraw ") {
            if let Some(msg) = Self::parse_withdraw(rest, false) {
                return msg;
            }
        }

        if let Some(rest) = text.strip_prefix("withdraw_receipt ") {
            if let Some(msg) = Self::parse_withdraw(rest, true) {
                return msg;
            }
        }

        ProxyExecuteMsg::ForwardToInflow {}
    }

    fn parse_withdraw(rest: &str, is_receipt: bool) -> Option<Self> {
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
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_forward_variants() {
        assert!(matches!(
            ProxyExecuteMsg::from_email_subject(""),
            ProxyExecuteMsg::ForwardToInflow {}
        ));
        assert!(matches!(
            ProxyExecuteMsg::from_email_subject("forward"),
            ProxyExecuteMsg::ForwardToInflow {}
        ));
        assert!(matches!(
            ProxyExecuteMsg::from_email_subject("FORWARD"),
            ProxyExecuteMsg::ForwardToInflow {}
        ));
        assert!(matches!(
            ProxyExecuteMsg::from_email_subject("deposit"),
            ProxyExecuteMsg::ForwardToInflow {}
        ));
    }

    #[test]
    fn test_withdraw_funds() {
        let msg = ProxyExecuteMsg::from_email_subject("withdraw neutron1abc uatom 1000000");
        match msg {
            ProxyExecuteMsg::WithdrawFunds { address, coin } => {
                assert_eq!(address, "neutron1abc");
                assert_eq!(coin.denom, "uatom");
                assert_eq!(coin.amount.to_string(), "1000000");
            }
            _ => panic!("expected WithdrawFunds"),
        }
    }

    #[test]
    fn test_withdraw_receipt() {
        let msg = ProxyExecuteMsg::from_email_subject(
            "withdraw_receipt neutron1xyz factory/vault/share 500000",
        );
        match msg {
            ProxyExecuteMsg::WithdrawReceiptTokens { address, coin } => {
                assert_eq!(address, "neutron1xyz");
                assert_eq!(coin.denom, "factory/vault/share");
                assert_eq!(coin.amount.to_string(), "500000");
            }
            _ => panic!("expected WithdrawReceiptTokens"),
        }
    }

    #[test]
    fn test_invalid_falls_back_to_forward() {
        assert!(matches!(
            ProxyExecuteMsg::from_email_subject("withdraw"),
            ProxyExecuteMsg::ForwardToInflow {}
        ));
        assert!(matches!(
            ProxyExecuteMsg::from_email_subject("withdraw addr"),
            ProxyExecuteMsg::ForwardToInflow {}
        ));
        assert!(matches!(
            ProxyExecuteMsg::from_email_subject("withdraw addr denom notanumber"),
            ProxyExecuteMsg::ForwardToInflow {}
        ));
        assert!(matches!(
            ProxyExecuteMsg::from_email_subject("random subject"),
            ProxyExecuteMsg::ForwardToInflow {}
        ));
    }

    #[test]
    fn test_roundtrip_forward() {
        let msg = ProxyExecuteMsg::ForwardToInflow {};
        let subject = msg.to_email_subject();
        let parsed = ProxyExecuteMsg::from_email_subject(&subject);
        assert!(matches!(parsed, ProxyExecuteMsg::ForwardToInflow {}));
    }

    #[test]
    fn test_roundtrip_withdraw_funds() {
        let msg = ProxyExecuteMsg::WithdrawFunds {
            address: "neutron1abc".to_string(),
            coin: Coin {
                denom: "uatom".to_string(),
                amount: Uint128::from(1000000u128).into(),
            },
        };
        let subject = msg.to_email_subject();
        let parsed = ProxyExecuteMsg::from_email_subject(&subject);
        match parsed {
            ProxyExecuteMsg::WithdrawFunds { address, coin } => {
                assert_eq!(address, "neutron1abc");
                assert_eq!(coin.denom, "uatom");
                assert_eq!(coin.amount.to_string(), "1000000");
            }
            _ => panic!("expected WithdrawFunds"),
        }
    }

    #[test]
    fn test_roundtrip_withdraw_receipt() {
        let msg = ProxyExecuteMsg::WithdrawReceiptTokens {
            address: "neutron1xyz".to_string(),
            coin: Coin {
                denom: "factory/vault/share".to_string(),
                amount: Uint128::from(500000u128).into(),
            },
        };
        let subject = msg.to_email_subject();
        let parsed = ProxyExecuteMsg::from_email_subject(&subject);
        match parsed {
            ProxyExecuteMsg::WithdrawReceiptTokens { address, coin } => {
                assert_eq!(address, "neutron1xyz");
                assert_eq!(coin.denom, "factory/vault/share");
                assert_eq!(coin.amount.to_string(), "500000");
            }
            _ => panic!("expected WithdrawReceiptTokens"),
        }
    }
}
