use crate::env_config::models::app_setting::AppSettings;
use crate::generate::tinkoff_public_invest_api_contract_v1::market_data_stream_service_client::MarketDataStreamServiceClient;
use crate::generate::tinkoff_public_invest_api_contract_v1::{
    instruments_service_client::InstrumentsServiceClient,
    market_data_service_client::MarketDataServiceClient,
    operations_service_client::OperationsServiceClient, users_service_client::UsersServiceClient,
 
};
use rustls::crypto::aws_lc_rs;

use std::io::Result;
use std::{sync::Arc, time::Duration};
use tonic::{
    metadata::MetadataValue,
    transport::{Channel, ClientTlsConfig},
    Request, 
};

#[derive(Clone)]
pub struct TinkoffClient {
    pub instruments: InstrumentsServiceClient<Channel>,
    pub market_data: MarketDataServiceClient<Channel>,
    pub market_data_stream: MarketDataStreamServiceClient<Channel>,
    pub operations: OperationsServiceClient<Channel>,
    pub users: UsersServiceClient<Channel>,
    pub token: String,
}

impl TinkoffClient {
    /// Создает новый экземпляр клиента с заданными настройками
    pub async fn new(settings: Arc<AppSettings>) -> Result<Self> {
        // Инициализация криптографического провайдера
        let provider = aws_lc_rs::default_provider();
        provider.install_default().unwrap();

        // Настройка TLS
        let tls_config = ClientTlsConfig::new()
            .domain_name(&settings.app_config.tinkoff_api.domain)
            .with_enabled_roots();

        // Создание канала с настроенной конфигурацией
        let channel = Channel::from_shared(settings.app_config.tinkoff_api.base_url.clone().into_bytes())
            .expect("Invalid URI format")
            .tls_config(tls_config)
            .expect("TLS configuration failed")
            .tcp_keepalive(Some(Duration::from_secs(settings.app_config.tinkoff_api.keepalive)))
            .timeout(Duration::from_secs(settings.app_config.tinkoff_api.timeout))
            .connect()
            .await
            .expect("Failed to connect to gRPC server");

        Ok(Self {
            instruments: InstrumentsServiceClient::new(channel.clone()),
            market_data: MarketDataServiceClient::new(channel.clone()),
            market_data_stream: MarketDataStreamServiceClient::new(channel.clone()),
            operations: OperationsServiceClient::new(channel.clone()),
            users: UsersServiceClient::new(channel.clone()),
            token: settings.app_env.tinkoff_token.clone(),
        })
    }

    /// Создает новый gRPC запрос с добавлением токена авторизации
    pub fn create_request<T>(&self, request: T) -> Result<Request<T>> {
        let mut request = Request::new(request);
        let auth_header_value = MetadataValue::try_from(&format!("Bearer {}", self.token))
            .expect("Invalid token format");
        request
            .metadata_mut()
            .insert("authorization", auth_header_value);
        Ok(request)
    }

}
