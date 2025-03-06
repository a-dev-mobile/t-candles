use serde::{Deserialize, Serialize};
use std::str::FromStr;
use std::fmt;

#[derive(Debug, Deserialize, Clone)]
pub struct AppEnv {
    pub env: Env,
    pub postgres_url: String,
    pub mongo_url: String,
    pub tinkoff_token: String,
    pub server_port: u16,
    pub server_address: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, Copy)]
pub enum Env {
    Local,
    Development,
    Production,
}

impl FromStr for Env {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "local" => Ok(Env::Local),
            "dev" | "development" => Ok(Env::Development),
            "prod" | "production" => Ok(Env::Production),
            _ => Err(format!("Unknown environment: {}", s)),
        }
    }
}



impl fmt::Display for Env {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let s = match self {
            Env::Local => "local",
            Env::Development => "dev",
            Env::Production => "prod",
        };
        write!(f, "{}", s)
    }
}
impl Env {
    pub fn is_dev(env: &Env) -> bool {
        matches!(env, Env::Local | Env::Development)
    }
}
