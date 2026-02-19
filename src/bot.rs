use crate::{config::Config, consts::CONFIG_PATH};

pub mod error;

use error::Error;
use open_weather_map_api::OwmApi;

#[derive(Debug)]
pub struct Bot {
    config: Config,
    owm_api: OwmApi,
}

impl Bot {
    pub async fn new() -> Result<Self, Error> {
        let config = match Config::load(CONFIG_PATH).await? {
            Ok(config) => config,
            Err(config) => config,
        };

        let owm_api_key = match std::env::var(&config.owm_api_key_env_var) {
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

        let owm_api = OwmApi::new(
            owm_api_key,
            chrono::TimeDelta::seconds(config.forecast.cache_ttl_s as i64),
            config.forecast.soft_cache_limit,
        );

        Ok(Self { config, owm_api })
    }

    pub async fn run(&mut self) -> Result<(), Error> {
        todo!("Implement run!");

        Ok(())
    }
}
