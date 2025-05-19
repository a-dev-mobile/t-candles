use std::sync::Arc;

use crate::db::clickhouse::connection::ClickhouseConnection;

pub struct ClickhouseInstrumentRepository {
    connection: Arc<ClickhouseConnection>,
}