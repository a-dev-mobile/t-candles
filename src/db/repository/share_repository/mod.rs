mod clickhouse_impl;
mod helper;

pub use clickhouse_impl::ClickhouseShareRepository;
use crate::generate::tinkoff_public_invest_api_contract_v1::Share;
use clickhouse::error::Error as ClickhouseError;
use async_trait::async_trait;
use crate::db::models::share::DbSharesLiquid;

#[async_trait]
pub trait ShareRepository {
    async fn insert_shares(&self, shares: &[Share]) -> Result<u64, ClickhouseError>;
    async fn get_liquid_shares(&self) -> Result<Vec<DbSharesLiquid>, ClickhouseError>;
}