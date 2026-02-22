#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("Meshtastic Error: {0}")]
    Meshtastic(#[from] meshtastic::errors::Error),
    #[error("Join Error: Failed to join a task: {0}")]
    JoinError(#[from] tokio::task::JoinError),
}

#[derive(Debug, thiserror::Error)]
pub enum SendError {
    #[error(
        "Message too Big: Message Size: {} > {} Max Message Size",
        0,
        crate::MAX_PAYLOAD_SIZE
    )]
    TooBig(usize),
}
