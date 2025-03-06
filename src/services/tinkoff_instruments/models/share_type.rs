// src/services/tinkoff_instruments/models/share_type.rs
use crate::generate::tinkoff_public_invest_api_contract_v1::ShareType;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TinkoffShareTypeModel {
    pub raw: i32,
    pub name: String,
}

impl From<i32> for TinkoffShareTypeModel {
    fn from(raw: i32) -> Self {
        let name = ShareType::from_i32(raw)
            .map(|share_type| share_type.as_str_name().to_string())
            .unwrap_or_else(|| "UNKNOWN".to_string());

        Self { raw, name }
    }
}

/// Реализация From для enum ShareType
impl From<ShareType> for TinkoffShareTypeModel {
    fn from(share_type: ShareType) -> Self {
        Self {
            raw: share_type as i32,
            name: share_type.as_str_name().to_string(),
        }
    }
}
