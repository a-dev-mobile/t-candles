mod clickhouse_impl;
mod helper;

use crate::{
    db::clickhouse::models::share::DbSharesLiquid,
    generate::tinkoff_public_invest_api_contract_v1::Share,
};
use async_trait::async_trait;
use clickhouse::error::Error as ClickhouseError;
pub use clickhouse_impl::ClickhouseShareRepository;

#[async_trait]
pub trait ShareRepository {
    async fn insert_shares(&self, shares: &[Share]) -> Result<u64, ClickhouseError>;
    async fn get_liquid_shares(&self) -> Result<Vec<DbSharesLiquid>, ClickhouseError>;
}
