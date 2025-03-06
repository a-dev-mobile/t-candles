use serde::{Deserialize, Serialize};


use crate::generate::tinkoff_public_invest_api_contract_v1::MoneyValue;


/// Human-readable MoneyValue model
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TinkoffMoneyValueModel {
    pub currency: String,
    pub units: i64,
    pub nano: i32,
    pub value: f64,
}

impl From<&MoneyValue> for TinkoffMoneyValueModel {
    fn from(m: &MoneyValue) -> Self {
        let units = m.units;
        let nano = m.nano;
        let value = units as f64 + (nano as f64 / 1_000_000_000.0);

        Self {
            currency: m.currency.clone(),
            units,
            nano,
            value,
        }
    }
}
