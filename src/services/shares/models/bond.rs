use serde::{Deserialize, Serialize};

use crate::generate::tinkoff_public_invest_api_contract_v1::Bond;

use super::{
    money_value::TinkoffMoneyValueModel, quotation::TinkoffQuotationModel,
    real_exchange::TinkoffRealExchangeModel, time_stamp::TinkoffTimestampModel,
    trading_status::TinkoffTradingStatusModel,
};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TinkoffBondModel {
    // Basic fields
    pub figi: String,
    pub ticker: String,
    pub class_code: String,
    pub isin: String,
    pub uid: String,
    pub position_uid: String,
    pub name: String,
    pub lot: i32,
    pub currency: String,
    pub exchange: String,

    // Flag fields
    pub short_enabled_flag: bool,
    pub otc_flag: bool,
    pub buy_available_flag: bool,
    pub sell_available_flag: bool,
    pub floating_coupon_flag: bool,
    pub perpetual_flag: bool,
    pub amortization_flag: bool,
    pub api_trade_available_flag: bool,
    pub for_iis_flag: bool,
    pub for_qual_investor_flag: bool,
    pub weekend_flag: bool,
    pub blocked_tca_flag: bool,
    pub subordinated_flag: bool,
    pub liquidity_flag: bool,

    // Enhanced enum fields
    pub trading_status: TinkoffTradingStatusModel,
    pub real_exchange: TinkoffRealExchangeModel,

    // Specific bond fields
    pub issue_size: i64,
    pub issue_size_plan: i64,
    pub nominal: Option<TinkoffMoneyValueModel>,
    pub initial_nominal: Option<TinkoffMoneyValueModel>,
    pub placement_price: Option<TinkoffMoneyValueModel>,
    pub aci_value: Option<TinkoffMoneyValueModel>,
    pub country_of_risk: String,
    pub country_of_risk_name: String,
    pub sector: String,
    pub issue_kind: String,
    pub coupon_quantity_per_year: i32,

    // Dates
    pub maturity_date: Option<TinkoffTimestampModel>,
    pub state_reg_date: Option<TinkoffTimestampModel>,
    pub placement_date: Option<TinkoffTimestampModel>,
    pub first_1min_candle_date: Option<TinkoffTimestampModel>,
    pub first_1day_candle_date: Option<TinkoffTimestampModel>,

    // Optional fields with enhanced types
    pub klong: Option<TinkoffQuotationModel>,
    pub kshort: Option<TinkoffQuotationModel>,
    pub dlong: Option<TinkoffQuotationModel>,
    pub dshort: Option<TinkoffQuotationModel>,
    pub dlong_min: Option<TinkoffQuotationModel>,
    pub dshort_min: Option<TinkoffQuotationModel>,
    pub min_price_increment: Option<TinkoffQuotationModel>,
    pub risk_level: Option<String>,
}

impl From<&Bond> for TinkoffBondModel {
    fn from(bond: &Bond) -> Self {
        TinkoffBondModel {
            figi: bond.figi.clone(),
            ticker: bond.ticker.clone(),
            class_code: bond.class_code.clone(),
            isin: bond.isin.clone(),
            uid: bond.uid.clone(),
            position_uid: bond.position_uid.clone(),
            name: bond.name.clone(),
            lot: bond.lot,
            currency: bond.currency.clone(),
            exchange: bond.exchange.clone(),

            short_enabled_flag: bond.short_enabled_flag,
            otc_flag: bond.otc_flag,
            buy_available_flag: bond.buy_available_flag,
            sell_available_flag: bond.sell_available_flag,
            floating_coupon_flag: bond.floating_coupon_flag,
            perpetual_flag: bond.perpetual_flag,
            amortization_flag: bond.amortization_flag,
            api_trade_available_flag: bond.api_trade_available_flag,
            for_iis_flag: bond.for_iis_flag,
            for_qual_investor_flag: bond.for_qual_investor_flag,
            weekend_flag: bond.weekend_flag,
            blocked_tca_flag: bond.blocked_tca_flag,
            subordinated_flag: bond.subordinated_flag,
            liquidity_flag: bond.liquidity_flag,

            trading_status: TinkoffTradingStatusModel::from(bond.trading_status),
            real_exchange: TinkoffRealExchangeModel::from(bond.real_exchange),

            issue_size: bond.issue_size,
            issue_size_plan: bond.issue_size_plan,
            nominal: bond.nominal.as_ref().map(TinkoffMoneyValueModel::from),
            initial_nominal: bond
                .initial_nominal
                .as_ref()
                .map(TinkoffMoneyValueModel::from),
            placement_price: bond
                .placement_price
                .as_ref()
                .map(TinkoffMoneyValueModel::from),
            aci_value: bond.aci_value.as_ref().map(TinkoffMoneyValueModel::from),
            country_of_risk: bond.country_of_risk.clone(),
            country_of_risk_name: bond.country_of_risk_name.clone(),
            sector: bond.sector.clone(),
            issue_kind: bond.issue_kind.clone(),
            coupon_quantity_per_year: bond.coupon_quantity_per_year,

            maturity_date: bond.maturity_date.as_ref().map(TinkoffTimestampModel::from),
            state_reg_date: bond
                .state_reg_date
                .as_ref()
                .map(TinkoffTimestampModel::from),
            placement_date: bond
                .placement_date
                .as_ref()
                .map(TinkoffTimestampModel::from),
            first_1min_candle_date: bond
                .first_1min_candle_date
                .as_ref()
                .map(TinkoffTimestampModel::from),
            first_1day_candle_date: bond
                .first_1day_candle_date
                .as_ref()
                .map(TinkoffTimestampModel::from),

            klong: bond.klong.as_ref().map(TinkoffQuotationModel::from),
            kshort: bond.kshort.as_ref().map(TinkoffQuotationModel::from),
            dlong: bond.dlong.as_ref().map(TinkoffQuotationModel::from),
            dshort: bond.dshort.as_ref().map(TinkoffQuotationModel::from),
            dlong_min: bond.dlong_min.as_ref().map(TinkoffQuotationModel::from),
            dshort_min: bond.dshort_min.as_ref().map(TinkoffQuotationModel::from),
            min_price_increment: bond
                .min_price_increment
                .as_ref()
                .map(TinkoffQuotationModel::from),
            risk_level: match bond.risk_level {
                0 => Some("RISK_LEVEL_HIGH".to_string()),
                1 => Some("RISK_LEVEL_MODERATE".to_string()),
                2 => Some("RISK_LEVEL_LOW".to_string()),
                _ => None,
            },
        }
    }
}
