use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TinkoffTradingStatusModel {
    pub raw: i32,
    pub value: String,
}

impl From<i32> for TinkoffTradingStatusModel {
    fn from(raw: i32) -> Self {
        let value = match raw {
            0 => "UNSPECIFIED",
            1 => "NOT_AVAILABLE_FOR_TRADING",
            2 => "OPENING_PERIOD",
            3 => "CLOSING_PERIOD",
            4 => "BREAK_IN_TRADING",
            5 => "NORMAL_TRADING",
            6 => "CLOSING_AUCTION",
            7 => "DARK_POOL_AUCTION",
            8 => "DISCRETE_AUCTION",
            9 => "OPENING_AUCTION_PERIOD",
            10 => "TRADING_AT_CLOSING_AUCTION_PRICE",
            11 => "SESSION_ASSIGNED",
            12 => "SESSION_CLOSE",
            13 => "SESSION_OPEN",
            14 => "DEALER_NORMAL_TRADING",
            15 => "DEALER_BREAK_IN_TRADING",
            16 => "DEALER_NOT_AVAILABLE_FOR_TRADING",
            _ => "UNKNOWN",
        };

        Self {
            raw,
            value: value.to_string(),
        }
    }
}
