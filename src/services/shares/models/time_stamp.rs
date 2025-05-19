use chrono::{DateTime, TimeZone, Utc};
use prost_types::Timestamp;
use serde::{Deserialize, Serialize};
use tracing::warn;

/// Human-readable Timestamp model
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TinkoffTimestampModel {
    pub seconds: i64,
    pub nanos: i32,
    pub timestamp_utc: String,
    pub datetime: Option<DateTime<Utc>>,
}

impl From<&Timestamp> for TinkoffTimestampModel {
    fn from(ts: &Timestamp) -> Self {
        let seconds = ts.seconds;
        let nanos = ts.nanos;

        // Convert to DateTime<Utc> with proper handling of edge cases
        let datetime = DateTime::from_timestamp(seconds, nanos as u32).or_else(|| {
            // For dates that can't be processed with the standard method
            match Utc.timestamp_opt(seconds, nanos as u32) {
                chrono::offset::LocalResult::Single(dt) => Some(dt),
                _ => {
                    warn!("Invalid timestamp: seconds={}, nanos={}", seconds, nanos);
                    None
                }
            }
        });

        // Format as ISO 8601/RFC 3339 string
        let timestamp_utc = datetime
            .map(|dt| dt.to_rfc3339())
            .unwrap_or_else(|| "Invalid date".to_string());

        Self {
            seconds,
            nanos,
            timestamp_utc,
            datetime,
        }
    }
}

// Convenient conversion method from prost_types::Timestamp to DateTime<Utc>
pub fn timestamp_to_datetime(ts: &Option<Timestamp>) -> Option<DateTime<Utc>> {
    match ts {
        Some(ts) => {
            let seconds = ts.seconds;
            let nanos = ts.nanos as u32;

            DateTime::from_timestamp(seconds, nanos).or_else(|| {
                match Utc.timestamp_opt(seconds, nanos) {
                    chrono::offset::LocalResult::Single(dt) => Some(dt),
                    _ => {
                        warn!("Invalid timestamp: seconds={}, nanos={}", seconds, nanos);
                        None
                    }
                }
            })
        }
        None => None,
    }
}
