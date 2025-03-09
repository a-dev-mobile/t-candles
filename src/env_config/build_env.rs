use super::models::app_env::{AppEnv, Env};
use std::env;
use std::str::FromStr;

impl AppEnv {
    pub fn new() -> AppEnv {
        let env = get_env_var("ENV");
        let server_port = get_env_var("SERVER_PORT");
        let server_address = get_env_var("SERVER_ADDRESS");
        let clickhouse_url = get_env_var("CLICKHOUSE_HOST");
        let clickhouse_user = get_env_var("CLICKHOUSE_USER");
        let clickhouse_password = get_env_var("CLICKHOUSE_PASSWORD");
        let clickhouse_database = get_env_var("CLICKHOUSE_DATABASE");
        let tinkoff_token = get_env_var("TINKOFF_TOKEN");

        AppEnv {
            env: Env::from_str(&env).expect("Unknown environment"),
            server_port: server_port.parse().expect("PORT must be a number"),
            server_address,
            clickhouse_url,
            clickhouse_user,
            clickhouse_password,
            clickhouse_database,
            tinkoff_token,
            postgres_host: get_env_var("POSTGRES_HOST"),
            postgres_user: get_env_var("POSTGRES_USER"),
            postgres_password: get_env_var("POSTGRES_PASSWORD"),
            postgres_database: get_env_var("POSTGRES_DATABASE"),
        }
    }
}

impl Default for AppEnv {
    fn default() -> Self {
        Self::new()
    }
}

fn get_env_var(name: &str) -> String {
    env::var(name).unwrap_or_else(|_| panic!("ENV -> {} is not set", name))
}
