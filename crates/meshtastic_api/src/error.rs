#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("Meshtastic Error: {0}")]
    Meshtastic(#[from] meshtastic::errors::Error),
    #[error("Tokio Serial Error: {0}")]
    TokioSerial(#[from] tokio_serial::Error),
    #[error("Join Error: Failed to join a task: {0}")]
    JoinError(#[from] tokio::task::JoinError),
}
