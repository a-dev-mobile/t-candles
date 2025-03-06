use serde::{Deserialize, Serialize};

use crate::generate::tinkoff_public_invest_api_contract_v1::ShareType;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TinkoffShareTypeModel {
    pub raw: i32,
    pub name: String,
}

impl From<i32> for TinkoffShareTypeModel {
    fn from(raw: i32) -> Self {
        let name = match raw {
            0 => "SHARE_TYPE_UNSPECIFIED",
            1 => "SHARE_TYPE_COMMON",
            2 => "SHARE_TYPE_PREFERRED",
            3 => "SHARE_TYPE_ADR",
            4 => "SHARE_TYPE_GDR",
            5 => "SHARE_TYPE_MLP",
            6 => "SHARE_TYPE_NY_REG_SHRS",
            7 => "SHARE_TYPE_CLOSED_END_FUND",
            8 => "SHARE_TYPE_REIT",
            _ => "UNKNOWN",
        };

        Self {
            raw,
            name: name.to_string(),
        }
    }
}

/// Реализация From для enum ShareType
impl From<ShareType> for TinkoffShareTypeModel {
    fn from(share_type: ShareType) -> Self {
        Self::from(share_type as i32)
    }
}
