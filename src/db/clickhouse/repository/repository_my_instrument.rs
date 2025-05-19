use std::sync::Arc;

use clickhouse::error::Error as ClickhouseError;
use tracing::info;

use crate::db::clickhouse::{
    connection::ClickhouseConnection, models::db_model_my_instrument::DbModelMyInstrument,
};

pub struct RepositoryMyInstrument {
    connection: Arc<ClickhouseConnection>,
}

impl RepositoryMyInstrument {
    pub fn new(connection: Arc<ClickhouseConnection>) -> Self {
        Self { connection }
    }

    pub async fn get_my_instrument(&self) -> Result<Vec<DbModelMyInstrument>, ClickhouseError> {
        let client = self.connection.get_client();
        let database = self.connection.get_database();
        let query = format!(
            "SELECT 
                uid, 
                 
                first_1min_candle_date, 
                last_1min_candle_date, 
            FROM {}.instrument_candle_info 
        ",
            database
        );
        let temp_rows = client
            .query(&query)
            .fetch_all::<DbModelMyInstrument>()
            .await?;

        return Ok(temp_rows);
    }
    pub async fn update_last_candle_date(
        &self,
        uid: &str,
        last_date: i64,
    ) -> Result<(), clickhouse::error::Error> {
        let client = self.connection.get_client();
        let database = self.connection.get_database();

        let query = format!(
            "ALTER TABLE {}.instrument_candle_info UPDATE 
            last_1min_candle_date = {}, 
            update_time = now() 
            WHERE uid = '{}'",
            database, last_date, uid
        );

        info!(
            "Updating last candle date for instrument {}: {}",
            uid, last_date
        );
        client.query(&query).execute().await?;

        Ok(())
    }
}
