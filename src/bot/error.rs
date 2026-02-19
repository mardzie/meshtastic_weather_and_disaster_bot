#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("Bot Config Error: {0}")]
    Config(#[from] crate::config::error::Error),
    #[error("Open Weather Map API Key Environmen Variable Error: {}", 0)]
    OpenWeatherMapApiKeyPath(#[from] std::env::VarError),
    #[error("Tokio Serial Error: {0}")]
    TokioSerial(#[from] tokio_serial::Error),
    #[error("Meshtastic Error: {0}")]
    Meshtastic(#[from] meshtastic::errors::Error),
}
