use std::path::Path;

use serde::{Deserialize, Serialize};

pub mod error;

use error::Error;

#[derive(Debug, Serialize, Deserialize)]
pub struct Config {
    pub openweathermap_api_key_path_var: String,
    pub forecast: Forecast,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Forecast {
    /// How many units of forecast time should be fetched.
    pub forecast_count: u8,
    /// The expiry in seconds.
    pub ttl_s: u32,
}

impl Config {
    /// Tries to load the config from `path`. If not existant write the default config to the `path` and return it.
    ///
    /// When there is a `Ok` in `Result<Self, Self>` then the config was loaded, on `Err` the default config is returned.
    pub async fn load(path: impl AsRef<Path>) -> Result<Result<Self, Self>, Error> {
        let config = if tokio::fs::try_exists(&path).await? {
            Ok(Self::read(path).await?)
        } else {
            let config = Self::default();
            config.write(path).await?;

            Err(config)
        };

        tracing::debug!("Loaded config");

        Ok(config)
    }

    /// Read the config from file.
    pub async fn read(path: impl AsRef<Path>) -> Result<Self, Error> {
        let config_string = tokio::fs::read_to_string(path).await?;
        let config = toml::from_str(&config_string)?;

        tracing::debug!("Read config");

        Ok(config)
    }

    /// Write the config to file.
    pub async fn write(&self, path: impl AsRef<Path>) -> Result<(), Error> {
        let config_string = toml::to_string_pretty(self)?;
        tokio::fs::write(&path, config_string).await?;

        tracing::debug!("Wrote config to {}", path.as_ref().to_string_lossy());

        Ok(())
    }
}

impl Default for Config {
    fn default() -> Self {
        Self {
            openweathermap_api_key_path_var: "OPEN_WEATHER_MAP_API_KEY".to_string(),
            forecast: Forecast {
                forecast_count: 6,
                ttl_s: 10800,
            },
        }
    }
}
