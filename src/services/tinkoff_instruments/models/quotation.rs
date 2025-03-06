use serde::{Deserialize, Serialize};


use crate::generate::tinkoff_public_invest_api_contract_v1::Quotation;
/// Human-readable Quotation model
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TinkoffQuotationModel {
    pub units: i64,
    pub nano: i32,
    pub value: f64,
}

impl From<&Quotation> for TinkoffQuotationModel {
    fn from(q: &Quotation) -> Self {
        let units = q.units;
        let nano = q.nano;
        let value = units as f64 + (nano as f64 / 1_000_000_000.0);

        Self { units, nano, value }
    }
}

impl From<Option<&Quotation>> for TinkoffQuotationModel {
    fn from(opt_quotation: Option<&Quotation>) -> Self {
        match opt_quotation {
            Some(q) => Self::from(q),
            None => Self { units: 0, nano: 0, value: 0.0 },
        }
    }
}

// Helper function to extract quotation fields safely
pub fn extract_quotation(quotation: &Option<Quotation>) -> (i64, i32) {
    quotation.as_ref().map_or((0, 0), |q| (q.units, q.nano))
}

// Helper function to convert quotation to float
pub fn quotation_to_f64(units: i64, nano: i32) -> f64 {
    units as f64 + (nano as f64 / 1_000_000_000.0)
}