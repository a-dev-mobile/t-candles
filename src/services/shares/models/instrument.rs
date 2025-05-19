use serde::{Deserialize, Serialize};

use super::{
    bond::TinkoffBondModel, etf::TinkoffEtfModel, future::TinkoffFutureModel, share::DbTinkoffShare,
};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum TinkoffInstrumentEnum {
    Share(DbTinkoffShare),
    Bond(TinkoffBondModel),
    Etf(TinkoffEtfModel),
    Future(TinkoffFutureModel),
}
