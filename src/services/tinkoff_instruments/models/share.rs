use serde::{Deserialize, Serialize};

use crate::generate::tinkoff_public_invest_api_contract_v1::Share;

use super::{
    money_value::TinkoffMoneyValueModel, quotation::TinkoffQuotationModel,
    real_exchange::TinkoffRealExchangeModel, share_type::TinkoffShareTypeModel,
    time_stamp::TinkoffTimestampModel, trading_status::TinkoffTradingStatusModel,
};
/// Complete human-readable share model
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DbTinkoffShare {
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
    pub div_yield_flag: bool,
    pub api_trade_available_flag: bool,
    pub for_iis_flag: bool,
    pub for_qual_investor_flag: bool,
    pub weekend_flag: bool,
    pub blocked_tca_flag: bool,
    pub liquidity_flag: bool,

    // Enhanced enum fields
    pub trading_status: TinkoffTradingStatusModel,
    pub share_type: TinkoffShareTypeModel,
    pub real_exchange: TinkoffRealExchangeModel,

    // Other fields
    pub issue_size: i64,
    pub issue_size_plan: i64,
    pub country_of_risk: String,
    pub country_of_risk_name: String,
    pub sector: String,

    // Optional fields with enhanced types
    pub klong: Option<TinkoffQuotationModel>,
    pub kshort: Option<TinkoffQuotationModel>,
    pub dlong: Option<TinkoffQuotationModel>,
    pub dshort: Option<TinkoffQuotationModel>,
    pub dlong_min: Option<TinkoffQuotationModel>,
    pub dshort_min: Option<TinkoffQuotationModel>,
    pub min_price_increment: Option<TinkoffQuotationModel>,
    pub nominal: Option<TinkoffMoneyValueModel>,
    pub ipo_date: Option<TinkoffTimestampModel>,
    pub first_1min_candle_date: Option<TinkoffTimestampModel>,
    pub first_1day_candle_date: Option<TinkoffTimestampModel>,
}

impl From<&Share> for DbTinkoffShare {
    fn from(share: &Share) -> Self {
        DbTinkoffShare {
            figi: share.figi.clone(),
            ticker: share.ticker.clone(),
            class_code: share.class_code.clone(),
            isin: share.isin.clone(),
            uid: share.uid.clone(),
            position_uid: share.position_uid.clone(),
            name: share.name.clone(),
            lot: share.lot,
            currency: share.currency.clone(),
            exchange: share.exchange.clone(),

            short_enabled_flag: share.short_enabled_flag,
            otc_flag: share.otc_flag,
            buy_available_flag: share.buy_available_flag,
            sell_available_flag: share.sell_available_flag,
            div_yield_flag: share.div_yield_flag,
            api_trade_available_flag: share.api_trade_available_flag,
            for_iis_flag: share.for_iis_flag,
            for_qual_investor_flag: share.for_qual_investor_flag,
            weekend_flag: share.weekend_flag,
            blocked_tca_flag: share.blocked_tca_flag,
            liquidity_flag: share.liquidity_flag,

            trading_status: TinkoffTradingStatusModel::from(share.trading_status),
            share_type: TinkoffShareTypeModel::from(share.share_type),
            real_exchange: TinkoffRealExchangeModel::from(share.real_exchange),

            issue_size: share.issue_size,
            issue_size_plan: share.issue_size_plan,
            country_of_risk: share.country_of_risk.clone(),
            country_of_risk_name: share.country_of_risk_name.clone(),
            sector: share.sector.clone(),

            klong: share.klong.as_ref().map(TinkoffQuotationModel::from),
            kshort: share.kshort.as_ref().map(TinkoffQuotationModel::from),
            dlong: share.dlong.as_ref().map(TinkoffQuotationModel::from),
            dshort: share.dshort.as_ref().map(TinkoffQuotationModel::from),
            dlong_min: share.dlong_min.as_ref().map(TinkoffQuotationModel::from),
            dshort_min: share.dshort_min.as_ref().map(TinkoffQuotationModel::from),
            min_price_increment: share
                .min_price_increment
                .as_ref()
                .map(TinkoffQuotationModel::from),
            nominal: share.nominal.as_ref().map(TinkoffMoneyValueModel::from),
            ipo_date: share.ipo_date.as_ref().map(TinkoffTimestampModel::from),
            first_1min_candle_date: share
                .first_1min_candle_date
                .as_ref()
                .map(TinkoffTimestampModel::from),
            first_1day_candle_date: share
                .first_1day_candle_date
                .as_ref()
                .map(TinkoffTimestampModel::from),
        }
    }
}
