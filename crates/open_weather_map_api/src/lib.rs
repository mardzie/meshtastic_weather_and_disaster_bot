use chrono::Duration;
use tracing::instrument;

use crate::{
    cache::Cache,
    error::Error,
    forecast::{
        Forecast,
        essential::{self},
    },
};

mod cache;

pub mod error;
pub mod forecast;

type Latitude = f64;
type Longitude = f64;

#[derive(Debug)]
pub struct OwmApi {
    api_key: String,
    cache: Cache,
}

impl OwmApi {
    pub fn new(api_key: String, cache_expiry: Duration) -> Self {
        Self {
            api_key,
            cache: Cache::new(cache_expiry),
        }
    }

    #[instrument]
    pub async fn get_5day_3hour_forecast(
        &mut self,
        lat: Latitude,
        lon: Longitude,
        count: Option<u8>,
    ) -> Result<essential::Forecast, Error> {
        const MAX_REQUESTABLE: u8 = (24 / 3) * 5; // (24 hours / 3 hours) * 5 days: This calculates the max count.

        let forecast = if let Some(forecast_hit) = self.cache.lookup(lat, lon) {
            forecast_hit
        } else {
            let mut url = format!(
                "https://api.openweathermap.org/data/2.5/forecast?lat={}&lon={}&appid={}&units=metric&lang=de",
                lat, lon, self.api_key
            );

            if let Some(count) = count {
                if count > MAX_REQUESTABLE {
                    return Err(Error::TooManyRequested(count, MAX_REQUESTABLE));
                };

                url.push_str(&format!("&cnt={}", count));
            };

            let response = reqwest::get(url).await?;
            Self::handle_status_code(&response)?;
            let response_text = response.text().await?;

            tracing::debug!(
                "5 Day 3 Hour Response at Lat: {}, Lon: {}: {}",
                lat,
                lon,
                response_text
            );

            let forecast: essential::Forecast =
                serde_json::from_str::<Forecast>(&response_text)?.into();
            self.cache.cache(lat, lon, forecast.clone());

            forecast
        };

        Ok(forecast)
    }

    pub async fn get_5day_3hour_forecast_by_name(
        &mut self,
        city_name: String,
        country_code: Option<String>,
    ) -> Result<essential::Forecast, Error> {
        todo!()
    }

    pub async fn get_lat_lon_by_name(
        &mut self,
        city_name: String,
        country_code: Option<String>,
    ) -> Result<(Latitude, Longitude), Error> {
        const LIMIT: usize = 1;

        let url = if let Some(country_code) = country_code {
            format!(
                "http://api.openweathermap.org/geo/1.0/direct?q={},{}&limit={}&appid={}",
                city_name, country_code, LIMIT, self.api_key
            )
        } else {
            format!(
                "http://api.openweathermap.org/geo/1.0/direct?q={}&limit={}&appid={}",
                city_name, LIMIT, self.api_key
            )
        };

        let response = reqwest::get(url).await?;

        Self::handle_status_code(&response)?;

        todo!();
    }

    fn handle_status_code(response: &reqwest::Response) -> Result<(), Error> {
        if 200 > response.status().as_u16() || response.status().as_u16() >= 300 {
            return Err(Error::StatusCode(response.status()));
        };

        Ok(())
    }
}
