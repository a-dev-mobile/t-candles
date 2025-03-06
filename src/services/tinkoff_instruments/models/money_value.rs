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

impl From<Option<&MoneyValue>> for TinkoffMoneyValueModel {
    fn from(opt_money: Option<&MoneyValue>) -> Self {
        match opt_money {
            Some(m) => Self::from(m),
            None => Self {
                currency: String::new(),
                units: 0,
                nano: 0,
                value: 0.0,
            },
        }
    }
}

// Helper function to extract money value fields safely
pub fn extract_money_value(money: &Option<MoneyValue>) -> (String, i64, i32) {
    money
        .as_ref()
        .map_or((String::new(), 0, 0), |m| (m.currency.clone(), m.units, m.nano))
}

// Helper function to convert money value to float
pub fn money_value_to_f64(units: i64, nano: i32) -> f64 {
    units as f64 + (nano as f64 / 1_000_000_000.0)
}