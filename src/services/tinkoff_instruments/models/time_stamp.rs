use chrono::{DateTime, TimeZone, Utc};
use prost_types::Timestamp;
use serde::{Deserialize, Serialize};


/// Human-readable Timestamp model
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TinkoffTimestampModel {
    pub seconds: i64,
    pub nanos: i32,
    pub timestamp_utc: String, 
}

impl From<&Timestamp> for TinkoffTimestampModel {
    fn from(ts: &Timestamp) -> Self {
        let seconds = ts.seconds;
        let nanos = ts.nanos;
        
        // Use DateTime::from_timestamp and format as proper ISO 8601 with UTC timezone
        let timestamp_utc = DateTime::from_timestamp(seconds, nanos as u32)
            .map(|dt| dt.to_rfc3339())  // RFC 3339 is a profile of ISO 8601 that includes the timezone
            .unwrap_or_else(|| {
                // For dates that can't be processed with the standard method
                if seconds < 0 {
                    // For dates before 1970
                    match Utc.timestamp_opt(seconds, nanos as u32) {
                        chrono::offset::LocalResult::Single(dt) => dt.to_rfc3339(),
                        _ => "Invalid date".to_string()
                    }
                } else {
                    "Invalid date".to_string()
                }
            });
            
        Self {
            seconds,
            nanos,
            timestamp_utc,
        }
    }
}