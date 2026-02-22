#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("Bot Config Error: {0}")]
    Config(#[from] crate::config::error::Error),
    #[error("Open Weather Map API Key Environmen Variable Error: {}", 0)]
    OpenWeatherMapApiKeyPath(#[from] std::env::VarError),
    #[error("Meshtastic API Error: {0}")]
    MeshtasticApi(#[from] meshtastic_api::error::Error),
    #[error("Tokio Serial Error: {0}")]
    TokioSerial(#[from] tokio_serial::Error),
}
