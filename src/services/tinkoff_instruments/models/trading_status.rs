// src/services/tinkoff_instruments/models/trading_status.rs
use crate::generate::tinkoff_public_invest_api_contract_v1::SecurityTradingStatus;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TinkoffTradingStatusModel {
    pub raw: i32,
    pub value: String,
}

impl From<i32> for TinkoffTradingStatusModel {
    fn from(raw: i32) -> Self {
        let value = SecurityTradingStatus::from_i32(raw)
            .map(|status| status.as_str_name().to_string())
            .unwrap_or_else(|| "UNKNOWN".to_string());

        Self { raw, value }
    }
}

impl From<SecurityTradingStatus> for TinkoffTradingStatusModel {
    fn from(status: SecurityTradingStatus) -> Self {
        Self {
            raw: status as i32,
            value: status.as_str_name().to_string(),
        }
    }
}
