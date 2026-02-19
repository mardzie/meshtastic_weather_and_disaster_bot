use serde::{Deserialize, Serialize};

use crate::{Latitude, Longitude};

pub mod essential;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Forecast {
    /// Internal parameter.
    cod: String,
    /// Internal parameter.
    message: i32,
    /// Number of timestamps returned.
    pub cnt: u8,
    /// The forecasts.
    pub list: Vec<ForecastSegment>,
    /// Deprecated.
    pub city: City,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ForecastSegment {
    /// Time of data forecasted, unix, UTC
    pub dt: u64,
    /// The main values of the forecast.
    pub main: MainValues,
    /// A list of Weather conditions.
    pub weather: Vec<Weather>,
    /// Cloudiness in %.
    pub clouds: Clouds,
    /// Wind.
    pub wind: Wind,
    /// Average visibility in meters. Maximum is 10 km.
    pub visibility: Option<u16>,
    /// Probability of precipitation. %, 0 - 1
    pub pop: f32,
    /// Rain volume in mm/last X hours.
    pub rain: Option<Rain>,
    /// Snow volume in mm/last X hours.
    pub snow: Option<Snow>,
    /// Day night. 'd' - Day, 'n' - Night
    pub sys: Sys,
    /// Time of data forecasted, ISO, UTC
    pub dt_txt: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MainValues {
    /// Temperature
    pub temp: f32,
    /// Feels like temperature. Takes the human perception into account.
    pub feels_like: f32,
    /// Minimum temperature at the moment of calculation. Should only be used optionally.
    pub temp_min: f32,
    /// Maximum temperature at the moment of calculation. Should only be used optionally.
    pub temp_max: f32,
    /// Atmospheric pressure at the sea level by default, hPa
    pub pressure: f32,
    /// Atmospheric pressure at the sea level, hPa
    pub sea_level: f32,
    /// Atmospheric pressure at ground level, hPa
    pub grnd_level: f32,
    /// Humidity, %
    pub humidity: u8,
    /// Internal parameter.
    temp_kf: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Weather {
    /// Weather condition id.
    pub id: u16,
    /// Group of weather parameters. (Rain, Snow, Clouds etc.)
    pub main: String,
    /// Weather conditions withing the group.
    pub description: String,
    /// Icon id. Can be fetched from "https://openweathermap.org/payload/api/media/file/{icon}.png"
    pub icon: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Clouds {
    /// Cloudiness, %
    pub all: u8,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Wind {
    /// Wind speed.
    pub speed: f32,
    /// Wind direction, degrees (meteorological)
    pub deg: u16,
    /// Wind gust.
    pub gust: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Rain {
    /// Rain volume for the last 3 hours in mm
    #[serde(alias = "3h")]
    pub three_hours: u16,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Snow {
    /// Snow volume for the last 3 hours in mm.
    #[serde(alias = "3h")]
    pub three_hours: u16,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Sys {
    /// 'd' - Day, 'n' - Night
    pub pod: char,
}

/// Deprecated by OWM.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct City {
    pub id: u32,
    pub name: String,
    pub coord: Coordinates,
    pub country: String,
    pub population: u32,
    pub timezone: i16,
    pub sunrise: u64,
    pub sunset: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Coordinates {
    pub lat: Latitude,
    pub lon: Longitude,
}
