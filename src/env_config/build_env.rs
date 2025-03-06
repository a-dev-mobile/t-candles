use super::models::app_env::{AppEnv, Env};
use std::env;
use std::str::FromStr;

impl AppEnv {
    pub fn new() -> AppEnv {
        let env = get_env_var("ENV");
        let server_port = get_env_var("SERVER_PORT");
        let server_address = get_env_var("SERVER_ADDRESS");
        let clickhouse_url = get_env_var("CLICKHOUSE_URL");
        let tinkoff_token = get_env_var("TINKOFF_TOKEN");

        AppEnv {
            env: Env::from_str(&env).expect("Unknown environment"),
            server_port: server_port.parse().expect("PORT must be a number"),
            server_address,
            clickhouse_url,
            tinkoff_token,
        }
    }
}

fn get_env_var(name: &str) -> String {
    env::var(name).unwrap_or_else(|_| panic!("ENV -> {} is not set", name))
}
