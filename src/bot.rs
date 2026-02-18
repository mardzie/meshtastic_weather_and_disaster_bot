use crate::{config::Config, consts::DEFAULT_CONFIG_PATH};

pub mod error;

use error::Error;

#[derive(Debug)]
pub struct Bot {
    config: Config,
    owm_api_key: String,
}

impl Bot {
    pub async fn new() -> Result<Self, Error> {
        let config = match Config::load(DEFAULT_CONFIG_PATH).await? {
            Ok(config) => config,
            Err(config) => config,
        };

        let owm_api_key = match std::env::var(&config.openweathermap_api_key_path_var) {
            Ok(key) => key,
            Err(std::env::VarError::NotPresent) => {
                tracing::error!(
                    "Failed to fetch the Open Weather API Key from enviroment variables!"
                );

                return Err(Error::OpenWeatherMapApiKeyPath(
                    std::env::VarError::NotPresent,
                ));
            }
            Err(e) => return Err(Error::OpenWeatherMapApiKeyPath(e)),
        };

        Ok(Self {
            config,
            owm_api_key,
        })
    }

    pub async fn run(&mut self) -> Result<(), Error> {
        Ok(())
    }
}
