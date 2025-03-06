use std::fmt;

#[derive(Debug)]
pub enum ClickhouseError {
    ConnectionError(String),
    QueryError(String),
    ConversionError(String),
    NotFound,
}

impl fmt::Display for ClickhouseError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ClickhouseError::ConnectionError(msg) => write!(f, "Connection error: {}", msg),
            ClickhouseError::QueryError(msg) => write!(f, "Query error: {}", msg),
            ClickhouseError::ConversionError(msg) => write!(f, "Conversion error: {}", msg),
            ClickhouseError::NotFound => write!(f, "Record not found"),
        }
    }
}

impl std::error::Error for ClickhouseError {}

impl From<clickhouse::error::Error> for ClickhouseError {
    fn from(err: clickhouse::error::Error) -> Self {
        ClickhouseError::QueryError(err.to_string())
    }
}