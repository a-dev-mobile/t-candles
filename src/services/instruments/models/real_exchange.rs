// src/services/tinkoff_instruments/models/real_exchange.rs
use crate::generate::tinkoff_public_invest_api_contract_v1::RealExchange;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TinkoffRealExchangeModel {
    pub raw: i32,
    pub name: String,
}

impl From<i32> for TinkoffRealExchangeModel {
    fn from(raw: i32) -> Self {
        let name = RealExchange::from_i32(raw)
            .map(|exchange| exchange.as_str_name().to_string())
            .unwrap_or_else(|| "UNKNOWN".to_string());

        Self { raw, name }
    }
}

/// Реализация From для enum RealExchange
impl From<RealExchange> for TinkoffRealExchangeModel {
    fn from(real_exchange: RealExchange) -> Self {
        Self {
            raw: real_exchange as i32,
            name: real_exchange.as_str_name().to_string(),
        }
    }
}
