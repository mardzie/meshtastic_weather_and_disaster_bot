use crate::{config::Config, consts::CONFIG_PATH};

pub mod error;

use error::Error;
use meshtastic_api::MeshtasticApi;
use open_weather_map_api::OwmApi;

#[derive(Debug)]
pub struct Bot {
    config: Config,
    owm_api: OwmApi,
    meshtastic_api: MeshtasticApi,
    packet_receiver: tokio::sync::mpsc::Receiver<meshtastic_api::packet::Packet>,

    listener_task: Option<tokio::task::JoinHandle<()>>,
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

        let available_ports = meshtastic::utils::stream::available_serial_ports()?;
        tracing::info!("Available Serial Ports: {:?}", available_ports);

        if config.meshtastic.serial_path == Config::default().meshtastic.serial_path {
            tracing::error!("Please set the Meshtastic serial path!");
        };

        let (packet_sender, packet_receiver) =
            tokio::sync::mpsc::channel(config.meshtastic.packet_buffer);
        let meshtastic_api = meshtastic_api::MeshtasticApi::new(
            config.meshtastic.serial_path.clone(),
            packet_sender,
        )
        .await?;

        Ok(Self {
            config,
            owm_api,
            meshtastic_api,
            packet_receiver,

            listener_task: None,
        })
    }

    pub async fn run(&mut self) -> Result<(), Error> {
        Ok(())
    }
}
