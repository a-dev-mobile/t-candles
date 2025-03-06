use serde::{Deserialize, Serialize};

use crate::generate::tinkoff_public_invest_api_contract_v1::RealExchange;
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TinkoffRealExchangeModel {
    pub raw: i32,
    pub name: String,
}

impl From<i32> for TinkoffRealExchangeModel {
    fn from(raw: i32) -> Self {
        let name = match raw {
            0 => "REAL_EXCHANGE_UNSPECIFIED",
            1 => "REAL_EXCHANGE_MOEX",
            2 => "REAL_EXCHANGE_RTS",
            3 => "REAL_EXCHANGE_OTC",
            _ => "UNKNOWN",
        };

        Self {
            raw,
            name: name.to_string(),
        }
    }
}

/// Реализация From для enum RealExchange
impl From<RealExchange> for TinkoffRealExchangeModel {
    fn from(real_exchange: RealExchange) -> Self {
        Self::from(real_exchange as i32)
    }
}
