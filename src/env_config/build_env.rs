use super::models::app_env::{AppEnv, Env};
use std::env;
use std::str::FromStr;

impl AppEnv {
    pub fn new() -> AppEnv {
        let env = get_env_var("ENV");
        let server_port = get_env_var("SERVER_PORT");
        let server_address = get_env_var("SERVER_ADDRESS");
        let postgres_url = get_env_var("POSTGRES_URL");
        let mongo_url = get_env_var("MONGO_URL");
        let tinkoff_token = get_env_var("TINKOFF_TOKEN");

        AppEnv {
            env: Env::from_str(&env).expect("Unknown environment"),
            server_port: server_port.parse().expect("PORT must be a number"),
            server_address: server_address,
            postgres_url: postgres_url,
            mongo_url: mongo_url,
            tinkoff_token,
        }
    }
}
fn get_env_var(name: &str) -> String {
    env::var(name).unwrap_or_else(|_| panic!("ENV -> {} is not set", name))
}
