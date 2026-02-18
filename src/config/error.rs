#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("IO Error: {0}")]
    Io(#[from] std::io::Error),
    #[error("Toml Serialize Error: {0}")]
    TomlSer(#[from] toml::ser::Error),
    #[error("Toml Deserialize Error: {0}")]
    TomlDe(#[from] toml::de::Error),
}
