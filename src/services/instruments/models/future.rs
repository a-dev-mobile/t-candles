use serde::{Deserialize, Serialize};

use super::{
    money_value::TinkoffMoneyValueModel, quotation::TinkoffQuotationModel,
    real_exchange::TinkoffRealExchangeModel, time_stamp::TinkoffTimestampModel,
    trading_status::TinkoffTradingStatusModel,
};
use crate::generate::tinkoff_public_invest_api_contract_v1::Future;
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TinkoffFutureModel {
    // Basic fields
    pub figi: String,
    pub ticker: String,
    pub class_code: String,
    pub lot: i32,
    pub currency: String,
    pub uid: String,
    pub position_uid: String,
    pub name: String,
    pub exchange: String,

    // Flag fields
    pub short_enabled_flag: bool,
    pub otc_flag: bool,
    pub buy_available_flag: bool,
    pub sell_available_flag: bool,
    pub for_iis_flag: bool,
    pub for_qual_investor_flag: bool,
    pub weekend_flag: bool,
    pub blocked_tca_flag: bool,
    pub api_trade_available_flag: bool,

    // Enhanced enum fields
    pub trading_status: TinkoffTradingStatusModel,
    pub real_exchange: TinkoffRealExchangeModel,

    // Future specific fields
    pub first_trade_date: Option<TinkoffTimestampModel>,
    pub last_trade_date: Option<TinkoffTimestampModel>,
    pub futures_type: String,
    pub asset_type: String,
    pub basic_asset: String,
    pub basic_asset_size: Option<TinkoffQuotationModel>,
    pub country_of_risk: String,
    pub country_of_risk_name: String,
    pub sector: String,
    pub expiration_date: Option<TinkoffTimestampModel>,
    pub basic_asset_position_uid: String,

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

impl From<&Future> for TinkoffFutureModel {
    fn from(future: &Future) -> Self {
        TinkoffFutureModel {
            figi: future.figi.clone(),
            ticker: future.ticker.clone(),
            class_code: future.class_code.clone(),
            lot: future.lot,
            currency: future.currency.clone(),
            uid: future.uid.clone(),
            position_uid: future.position_uid.clone(),
            name: future.name.clone(),
            exchange: future.exchange.clone(),

            short_enabled_flag: future.short_enabled_flag,
            otc_flag: future.otc_flag,
            buy_available_flag: future.buy_available_flag,
            sell_available_flag: future.sell_available_flag,
            for_iis_flag: future.for_iis_flag,
            for_qual_investor_flag: future.for_qual_investor_flag,
            weekend_flag: future.weekend_flag,
            blocked_tca_flag: future.blocked_tca_flag,
            api_trade_available_flag: future.api_trade_available_flag,

            trading_status: TinkoffTradingStatusModel::from(future.trading_status),
            real_exchange: TinkoffRealExchangeModel::from(future.real_exchange),

            first_trade_date: future
                .first_trade_date
                .as_ref()
                .map(TinkoffTimestampModel::from),
            last_trade_date: future
                .last_trade_date
                .as_ref()
                .map(TinkoffTimestampModel::from),
            futures_type: future.futures_type.clone(),
            asset_type: future.asset_type.clone(),
            basic_asset: future.basic_asset.clone(),
            basic_asset_size: future
                .basic_asset_size
                .as_ref()
                .map(TinkoffQuotationModel::from),
            country_of_risk: future.country_of_risk.clone(),
            country_of_risk_name: future.country_of_risk_name.clone(),
            sector: future.sector.clone(),
            expiration_date: future
                .expiration_date
                .as_ref()
                .map(TinkoffTimestampModel::from),
            basic_asset_position_uid: future.basic_asset_position_uid.clone(),

            klong: future.klong.as_ref().map(TinkoffQuotationModel::from),
            kshort: future.kshort.as_ref().map(TinkoffQuotationModel::from),
            dlong: future.dlong.as_ref().map(TinkoffQuotationModel::from),
            dshort: future.dshort.as_ref().map(TinkoffQuotationModel::from),
            dlong_min: future.dlong_min.as_ref().map(TinkoffQuotationModel::from),
            dshort_min: future.dshort_min.as_ref().map(TinkoffQuotationModel::from),
            min_price_increment: future
                .min_price_increment
                .as_ref()
                .map(TinkoffQuotationModel::from),
            first_1min_candle_date: future
                .first_1min_candle_date
                .as_ref()
                .map(TinkoffTimestampModel::from),
            first_1day_candle_date: future
                .first_1day_candle_date
                .as_ref()
                .map(TinkoffTimestampModel::from),
        }
    }
}
