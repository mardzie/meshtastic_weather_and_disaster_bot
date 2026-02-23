use std::collections::HashMap;

use crate::{
    config::Config,
    consts::CONFIG_PATH,
    essential_forecast::{Forecast, ForecastSegment},
};

pub mod error;

use error::Error;
use meshtastic_api::{MeshtasticApi, packet::Target, payload::Payload};
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
        println!("Available Serial Ports: {:?}", available_ports);

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
        tracing::info!("Started running.");

        while let Some(packet) = self.packet_receiver.recv().await {
            tracing::debug!("Got packet: {:?}", packet);

            if packet.via_mqtt {
                continue;
            }

            let args: Vec<&str> = packet.payload.trim().split(' ').collect();
            if args.first() != Some(&self.config.forecast_request_command.as_str()) {
                continue;
            };

            let text = match (args.get(1), args.get(2)) {
                (Some(lat), Some(lon)) => {
                    if let Ok((lat, lon)) = Self::parse_lat_lon(lat, lon) {
                        self.get_forecast_text(lat, lon)
                            .await
                            .map_or_else(|e_fc| e_fc, |fc| fc)
                    } else {
                        format!(
                            "{}\n\
                            Failed to parse coordinates.",
                            Self::get_manual(&self.config.forecast_request_command)
                        )
                    }
                }
                _ => {
                    tracing::warn!("Got Message: {}; but args arent correct.", packet.payload);
                    Self::get_manual(&self.config.forecast_request_command)
                }
            };

            let payload = match Payload::new(text.clone()) {
                Ok(payload) => payload,
                Err(_) => {
                    tracing::warn!("Payload too long.");
                    Payload::new_unchecked(format!(
                        "{} Payload too long",
                        text[..meshtastic_api::payload::MAX_PAYLOAD_SIZE - 17].to_string()
                    ))
                }
            };

            let target = if packet.to == Target::PrimaryChannel {
                Target::PrimaryChannel
            } else {
                Target::NodeId(packet.from)
            };

            if let Err(e) = self
                .meshtastic_api
                .send_message(payload.clone(), target.clone(), None)
                .await
            {
                tracing::warn!("Failed to send Message to {}: {}", target.into_id(), e);
            };
            tracing::info!("Sent Message {:?}", payload);
        }

        Ok(())
    }

    async fn get_forecast_text(&mut self, lat: f64, lon: f64) -> Result<String, String> {
        let forecast = match self.owm_api.get_5day_3hour_forecast(lat, lon, None).await {
            Ok(forecast) => Forecast::from(forecast),
            Err(e) => {
                tracing::warn!("Failed to get forecast: {}", e);
                return Err(match e {
                    open_weather_map_api::error::Error::Json(_) => format!(
                        "Weather Bot Error\nMalformed JSON API response. Contact {}",
                        self.config.contact
                    ),
                    open_weather_map_api::error::Error::Reqwest(_) => format!(
                        "Weather Bot Error\nRequest Error: Contact {}",
                        self.config.contact
                    ),
                    open_weather_map_api::error::Error::StatusCode(code) => format!(
                        "Weather Bot Error\nAPI returned status code: {}\nContact {}",
                        code, self.config.contact
                    ),
                    open_weather_map_api::error::Error::TooManyRequested(_, _) => format!(
                        "Weather Bot Error\nRequested too many forecast segments.\nContact {}",
                        self.config.contact
                    ),
                });
            }
        };

        let mut forecast_string = String::with_capacity(meshtastic_api::payload::MAX_PAYLOAD_SIZE);

        let mut temp_string = String::with_capacity(32);
        for (i, fcs) in forecast.iter().enumerate() {
            if i == 0 {
                temp_string.push_str(&self.config.forecast_header);
            };

            temp_string.push_str(&self.config.forecast_segment);

            if let Err(e) = Self::replace_with_forecast(fcs, &mut forecast_string) {
                tracing::error!("Failed to build Weather report: {}", e);
                return Err(Self::get_error_weather(fcs));
            };

            forecast_string.push('\n');
            forecast_string.extend(temp_string.chars());
            temp_string.clear();
        }

        Ok(forecast_string)
    }

    /// Replaces {TEXT} with the corresponding variable.
    ///
    /// List of general vars:
    /// - {DATE}
    /// - {TIME}
    /// - {DATETIME}
    ///
    /// List of ForecastSegment vars:
    /// - {FC_DATETIME}
    /// - {CLOUDS} // In % 0 - 100
    /// - {HUMIDITY}
    /// - {POP} // Probability of precipitation in % 0 - 100
    /// - {PRESSURE}
    /// - {PRESSURE_GROUND}
    /// - {RAIN} // Rain volume in mm.
    /// - {SNOW} // Snow volume in cm.
    /// - {TEMP}
    /// - {MIN_TEMP}
    /// - {MAX_TEMP}
    /// - {FEELS_LIKE_TEMP}
    /// - {VISIBILITY} // In m. May be replace with "None".
    /// - {WIND_SPEED}
    /// - {WIND_DEG} // 0 = North; 90 = East; 180 = South; 270 = East
    /// - {WIND_GUST}
    /// - {WEATHER}
    fn replace_with_forecast(
        fcs: &ForecastSegment,
        buf: &mut String,
    ) -> Result<(), strfmt::FmtError> {
        use chrono::Local;

        let now = Local::now();

        let mut vars: HashMap<String, String> = HashMap::with_capacity(20);

        // Add general replaces.
        {
            vars.insert("DATE".to_string(), now.format("%d.%m.%Y").to_string());
            vars.insert("TIME".to_string(), now.format("%H:%M").to_string());
            vars.insert(
                "DATETIME".to_string(),
                now.format("%d.%m.%Y %H:%M %Z").to_string(),
            );
        }

        // Add forecast replaces.
        {
            vars.insert("FC_DATETIME".to_string(), fcs.date_time_txt.clone());
            vars.insert(
                "CLOUDS".to_string(),
                ((fcs.clouds * 100.0) as u8).to_string(),
            );
            vars.insert(
                "HUMIDITY".to_string(),
                ((fcs.humidity * 100.0) as u8).to_string(),
            );
            vars.insert("POP".to_string(), ((fcs.pop * 100.0) as u8).to_string());
            vars.insert(
                "PRESSURE".to_string(),
                format!("{:.0}", fcs.pressure.pressure.round()),
            );
            vars.insert(
                "PRESSURE_GROUND".to_string(),
                format!("{:.0}", fcs.pressure.ground_level.round()),
            );
            vars.insert(
                "RAIN".to_string(),
                fcs.rain
                    .map_or(format!("{:.0}", 0.0), |r| format!("{:.0}", r.round())),
            );
            vars.insert(
                "SNOW".to_string(),
                fcs.snow
                    .map_or(format!("{:.0}", 0.0), |s| format!("{:.0}", s.round())),
            );
            vars.insert("TEMP".to_string(), format!("{:.0}", fcs.temp.temp.round()));
            vars.insert(
                "MIN_TEMP".to_string(),
                format!("{:.0}", fcs.temp.min.round()),
            );
            vars.insert(
                "MAX_TEMP".to_string(),
                format!("{:.0}", fcs.temp.max.round()),
            );
            vars.insert(
                "FEELS_LIKE_TEMP".to_string(),
                format!("{:.0}", fcs.temp.feels_like.round()),
            );
            vars.insert(
                "VISIBILITY".to_string(),
                fcs.visibility.map_or("None".to_string(), |v| v.to_string()),
            );
            vars.insert(
                "WIND_SPEED".to_string(),
                format!("{:.0}", fcs.wind.speed.round()),
            );
            vars.insert("WIND_DEG".to_string(), fcs.wind.deg.to_string());
            vars.insert(
                "WIND_GUST".to_string(),
                format!("{:.0}", fcs.wind.gust.round()),
            );
            let weather = fcs
                .weather
                .iter()
                .fold(String::new(), |mut acc, v| {
                    acc.push_str(v);
                    acc.push(' ');
                    acc
                })
                .trim()
                .to_string();
            vars.insert("WEATHER".to_string(), weather);
        }

        *buf = match strfmt::strfmt(&buf, &vars) {
            Ok(buf) => buf,
            Err(e) => {
                tracing::error!("Weather Formatting Error: {}", e);
                return Err(e);
            }
        };

        Ok(())
    }

    fn get_error_weather(fcs: &ForecastSegment) -> String {
        format!(
            "Weather Bot Error\n\
            Failed to build weather report!\n\
            {}\n\
            Weather: {}\n\
            Temp: {:.0} C; Feels: {:.0} C;\n\
            Clouds: {} %\n\
            Prob Rain: {:.0} %; Rain: {:.0} mm; Snow: {:.0} cm\n\
            Press: {:.0}\n\
            Wind: {} m/s; Deg: {}",
            fcs.date_time_txt,
            fcs.weather.iter().fold(String::new(), |mut acc, v| {
                acc.push_str(v);
                acc
            }),
            fcs.temp.temp.round(),
            fcs.temp.feels_like.round(),
            fcs.clouds * 100.0,
            fcs.pop * 100.0,
            fcs.rain.map_or(0.0, |r| r * 100.0),
            fcs.snow.map_or(0.0, |s| s * 100.0),
            fcs.pressure.pressure.round(),
            fcs.wind.speed,
            fcs.wind.deg
        )
    }

    fn get_manual(command: &str) -> String {
        format!("{} [LAT] [LON]", command)
    }

    fn parse_lat_lon(lat: &str, lon: &str) -> Result<(f64, f64), ()> {
        match (lat.parse(), lon.parse()) {
            (Ok(lat), Ok(lon)) => Ok((lat, lon)),
            (Err(e), Ok(_)) => {
                tracing::warn!("Failed to parse lat: {} into f64: {}", lat, e);
                return Err(());
            }
            (Ok(_), Err(e)) => {
                tracing::warn!("Failed to parse lon: {} into f64: {}", lon, e);
                return Err(());
            }
            (Err(lat_e), Err(lon_e)) => {
                tracing::warn!(
                    "Failed to parse coords: {:?} into (f64, f64): {:?}",
                    (lat, lon),
                    (lat_e, lon_e)
                );
                return Err(());
            }
        }
    }
}
