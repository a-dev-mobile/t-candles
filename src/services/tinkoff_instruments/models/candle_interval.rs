// models/candle_interval.rs

#[derive(Debug, Clone, Copy)]
pub enum MyCandleInterval {
    /// Интервал не определён.
    Unspecified = 0,
    /// от 1 минуты до 1 дня.
    OneMin = 1,
    /// от 5 минут до 1 дня.
    FiveMin = 2,
    /// от 15 минут до 1 дня.
    FifteenMin = 3,
    /// от 1 часа до 1 недели.
    Hour = 4,
    /// от 1 дня до 1 года.
    Day = 5,
    /// от 2 минут до 1 дня.
    TwoMin = 6,
    /// от 3 минут до 1 дня.
    ThreeMin = 7,
    /// от 10 минут до 1 дня.
    TenMin = 8,
    /// от 30 минут до 2 дней.
    ThirtyMin = 9,
    /// от 2 часов до 1 месяца.
    TwoHour = 10,
    /// от 4 часов до 1 месяца.
    FourHour = 11,
    /// от 1 недели до 2 лет.
    Week = 12,
    /// от 1 месяца до 10 лет.
    Month = 13,
}

impl MyCandleInterval {
    pub fn from_i32(value: i32) -> Option<Self> {
        match value {
            0 => Some(Self::Unspecified),
            1 => Some(Self::OneMin),
            2 => Some(Self::FiveMin),
            3 => Some(Self::FifteenMin),
            4 => Some(Self::Hour),
            5 => Some(Self::Day),
            6 => Some(Self::TwoMin),
            7 => Some(Self::ThreeMin),
            8 => Some(Self::TenMin),
            9 => Some(Self::ThirtyMin),
            10 => Some(Self::TwoHour),
            11 => Some(Self::FourHour),
            12 => Some(Self::Week),
            13 => Some(Self::Month),
            _ => None,
        }
    }

    pub fn as_str_name(&self) -> &'static str {
        match self {
            Self::Unspecified => "CANDLE_INTERVAL_UNSPECIFIED",
            Self::OneMin => "CANDLE_INTERVAL_1_MIN",
            Self::FiveMin => "CANDLE_INTERVAL_5_MIN",
            Self::FifteenMin => "CANDLE_INTERVAL_15_MIN",
            Self::Hour => "CANDLE_INTERVAL_HOUR",
            Self::Day => "CANDLE_INTERVAL_DAY",
            Self::TwoMin => "CANDLE_INTERVAL_2_MIN",
            Self::ThreeMin => "CANDLE_INTERVAL_3_MIN",
            Self::TenMin => "CANDLE_INTERVAL_10_MIN",
            Self::ThirtyMin => "CANDLE_INTERVAL_30_MIN",
            Self::TwoHour => "CANDLE_INTERVAL_2_HOUR",
            Self::FourHour => "CANDLE_INTERVAL_4_HOUR",
            Self::Week => "CANDLE_INTERVAL_WEEK",
            Self::Month => "CANDLE_INTERVAL_MONTH",
        }
    }
}