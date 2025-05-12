use serde::{Deserialize, Serialize};

use crate::generate::tinkoff_public_invest_api_contract_v1::{Bond, Etf};

use super::{
    money_value::TinkoffMoneyValueModel, quotation::TinkoffQuotationModel,
    real_exchange::TinkoffRealExchangeModel, time_stamp::TinkoffTimestampModel,
    trading_status::TinkoffTradingStatusModel,
};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TinkoffEtfModel {
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
    pub api_trade_available_flag: bool,
    pub for_iis_flag: bool,
    pub for_qual_investor_flag: bool,
    pub weekend_flag: bool,
    pub blocked_tca_flag: bool,
    pub liquidity_flag: bool,

    // Enhanced enum fields
    pub trading_status: TinkoffTradingStatusModel,
    pub real_exchange: TinkoffRealExchangeModel,

    // ETF specific fields
    pub fixed_commission: Option<TinkoffQuotationModel>,
    pub focus_type: String,
    pub released_date: Option<TinkoffTimestampModel>,
    pub num_shares: Option<TinkoffQuotationModel>,
    pub country_of_risk: String,
    pub country_of_risk_name: String,
    pub sector: String,
    pub rebalancing_freq: String,

    // Optional fields with enhanced types
    pub klong: Option<TinkoffQuotationModel>,
    pub kshort: Option<TinkoffQuotationModel>,
    pub dlong: Option<TinkoffQuotationModel>,
    pub dshort: Option<TinkoffQuotationModel>,
    pub dlong_min: Option<TinkoffQuotationModel>,
    pub dshort_min: Option<TinkoffQuotationModel>,
    pub min_price_increment: Option<TinkoffQuotationModel>,
    pub first_1min_candle_date: Option<TinkoffTimestampModel>,
    pub first_1day_candle_date: Option<TinkoffTimestampModel>,
}

impl From<&Etf> for TinkoffEtfModel {
    fn from(etf: &Etf) -> Self {
        TinkoffEtfModel {
            figi: etf.figi.clone(),
            ticker: etf.ticker.clone(),
            class_code: etf.class_code.clone(),
            isin: etf.isin.clone(),
            uid: etf.uid.clone(),
            position_uid: etf.position_uid.clone(),
            name: etf.name.clone(),
            lot: etf.lot,
            currency: etf.currency.clone(),
            exchange: etf.exchange.clone(),

            short_enabled_flag: etf.short_enabled_flag,
            otc_flag: etf.otc_flag,
            buy_available_flag: etf.buy_available_flag,
            sell_available_flag: etf.sell_available_flag,
            api_trade_available_flag: etf.api_trade_available_flag,
            for_iis_flag: etf.for_iis_flag,
            for_qual_investor_flag: etf.for_qual_investor_flag,
            weekend_flag: etf.weekend_flag,
            blocked_tca_flag: etf.blocked_tca_flag,
            liquidity_flag: etf.liquidity_flag,

            trading_status: TinkoffTradingStatusModel::from(etf.trading_status),
            real_exchange: TinkoffRealExchangeModel::from(etf.real_exchange),

            fixed_commission: etf
                .fixed_commission
                .as_ref()
                .map(TinkoffQuotationModel::from),
            focus_type: etf.focus_type.clone(),
            released_date: etf.released_date.as_ref().map(TinkoffTimestampModel::from),
            num_shares: etf.num_shares.as_ref().map(TinkoffQuotationModel::from),
            country_of_risk: etf.country_of_risk.clone(),
            country_of_risk_name: etf.country_of_risk_name.clone(),
            sector: etf.sector.clone(),
            rebalancing_freq: etf.rebalancing_freq.clone(),

            klong: etf.klong.as_ref().map(TinkoffQuotationModel::from),
            kshort: etf.kshort.as_ref().map(TinkoffQuotationModel::from),
            dlong: etf.dlong.as_ref().map(TinkoffQuotationModel::from),
            dshort: etf.dshort.as_ref().map(TinkoffQuotationModel::from),
            dlong_min: etf.dlong_min.as_ref().map(TinkoffQuotationModel::from),
            dshort_min: etf.dshort_min.as_ref().map(TinkoffQuotationModel::from),
            min_price_increment: etf
                .min_price_increment
                .as_ref()
                .map(TinkoffQuotationModel::from),
            first_1min_candle_date: etf
                .first_1min_candle_date
                .as_ref()
                .map(TinkoffTimestampModel::from),
            first_1day_candle_date: etf
                .first_1day_candle_date
                .as_ref()
                .map(TinkoffTimestampModel::from),
        }
    }
}
