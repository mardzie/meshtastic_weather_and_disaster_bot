use std::ops::Deref;

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, PartialOrd, Serialize, Deserialize)]
pub enum Forecast {
    Hour3(Vec<ForecastSegment>),
}

#[derive(Debug, Clone, PartialEq, PartialOrd, Serialize, Deserialize)]
pub struct ForecastSegment {
    /// UNIX timestamp.
    pub date_time: u64,
    /// Human readable timestamp.
    pub date_time_txt: String,
    pub temp: Temp,

    /// Pressure
    ///
    /// hPa.
    pub pressure: Pressure,

    /// Humidity
    ///
    /// %, 0.0 - 1.0
    pub humidity: f32,
    /// Weather conditions that are active.
    pub weather: Vec<Weather>,

    /// Cloudiness
    ///
    /// %, 0.0 - 1.0
    pub clouds: f32,
    pub wind: Wind,
    pub visibility: Option<u16>,

    /// Probability of precipitation.
    ///
    /// %, 0.0 - 1.0
    pub pop: f32,
    /// Day/Night
    pub day_time: DayTime,
    /// Rain volume in cm/X hours.
    pub rain: Option<f32>,
    /// Snow volume in cm/X hours.
    pub snow: Option<f32>,
}

#[derive(Debug, Clone, PartialEq, PartialOrd, Serialize, Deserialize)]
pub struct Temp {
    pub temp: f32,
    pub feels_like: f32,
    pub max: f32,
    pub min: f32,
}

#[derive(Debug, Clone, PartialEq, PartialOrd, Serialize, Deserialize)]
pub struct Pressure {
    pub pressure: f32,
    pub ground_level: f32,
}

#[derive(Debug, Clone, PartialEq, PartialOrd, Serialize, Deserialize)]
pub enum Weather {
    Thunderstorm(String),
    Drizzle(String),
    Rain(String),
    Snow(String),
    Atmosphere(Atmosphere),
    Clear(String),
    Clouds(String),
}

#[derive(Debug, Clone, PartialEq, PartialOrd, Serialize, Deserialize)]
pub enum Atmosphere {
    Mist(String),
    Smoke(String),
    Haze(String),
    SandDust(String),
    Fog(String),
    Sand(String),
    Dust(String),
    Ash(String),
    Squall(String),
    Tornado(String),
}

#[derive(Debug, Clone, PartialEq, PartialOrd, Serialize, Deserialize)]
pub struct Wind {
    /// Wind speed.
    pub speed: f32,

    /// Wind direction in degrees.
    ///
    /// 0 = North, 90 = East, 180 = South, 270 = West
    pub deg: u16,
    /// Gust speed
    pub gust: f32,
}

#[derive(Debug, Clone, PartialEq, PartialOrd, Hash, Serialize, Deserialize)]
pub enum DayTime {
    Day,
    Night,
}

impl From<open_weather_map_api::forecast::Forecast> for Forecast {
    fn from(lfc: open_weather_map_api::forecast::Forecast) -> Self {
        let mut forecast: Vec<ForecastSegment> = lfc
            .list
            .into_iter()
            .map(|fc| {
                let fc_main = fc.main;

                ForecastSegment {
                    date_time: fc.dt,
                    date_time_txt: fc.dt_txt,
                    temp: Temp {
                        temp: fc_main.temp,
                        feels_like: fc_main.feels_like,
                        max: fc_main.temp_max,
                        min: fc_main.temp_min,
                    },
                    pressure: Pressure {
                        pressure: fc_main.pressure,
                        ground_level: fc_main.grnd_level,
                    },
                    humidity: fc_main.humidity as f32 / 100.0,
                    weather: {
                        let len = fc.weather.len();
                        let mut weather =
                            fc.weather
                                .into_iter()
                                .fold(Vec::with_capacity(len), |mut acc, w| {
                                    acc.push(Weather::from_id(w.id, w.description));
                                    acc
                                });
                        weather.shrink_to_fit();
                        weather
                    },
                    clouds: fc.clouds.all as f32 / 100.0,
                    wind: Wind {
                        speed: fc.wind.speed,
                        deg: fc.wind.deg,
                        gust: fc.wind.gust,
                    },
                    visibility: fc.visibility,
                    pop: fc.pop,
                    day_time: DayTime::from(fc.sys.pod),
                    rain: fc.rain.map(|r| r.three_hours),
                    snow: fc.snow.map(|r| r.three_hours),
                }
            })
            .collect();
        forecast.shrink_to_fit();

        Self::Hour3(forecast)
    }
}

impl From<&Forecast> for meshtastic::protobufs::EnvironmentMetrics {
    /// Input a full day forecast to get the 24h rainfall forecast metric.
    fn from(forecast: &Forecast) -> Self {
        let forecast_segments: Vec<ForecastSegment> = match forecast {
            Forecast::Hour3(fc) => {
                if fc.len() >= 8 {
                    fc[..8].iter().cloned().collect()
                } else {
                    vec![fc.get(0).expect("Failed to access forecast for environment metrics update. No segments available.").clone()]
                }
            }
        };

        let (fcs, rainfall_1h) = match forecast {
            Forecast::Hour3(fc) => {
                let fcs = fc[0].clone();

                (fcs.clone(), fcs.rain.map_or(None, |rain| Some(rain / 3.0)))
            }
        };

        let rainfall_24h =
            forecast_segments
                .into_iter()
                .fold(None, |acc: Option<f32>, fc: ForecastSegment| {
                    if let Some(rain) = fc.rain {
                        acc.map_or_else(|| Some(rain), |rain_prev| Some(rain_prev + rain))
                    } else {
                        acc
                    }
                });

        meshtastic::protobufs::EnvironmentMetrics {
            temperature: Some(fcs.temp.temp),
            relative_humidity: Some(fcs.humidity),
            barometric_pressure: Some(fcs.pressure.ground_level),
            gas_resistance: None,
            voltage: None,
            current: None,
            iaq: None,
            distance: None,
            lux: None,
            white_lux: None,
            ir_lux: None,
            uv_lux: None,
            wind_direction: Some(fcs.wind.deg as u32),
            wind_speed: Some(fcs.wind.speed),
            weight: None,
            wind_gust: Some(fcs.wind.gust),
            wind_lull: None,
            radiation: None,
            rainfall_1h: rainfall_1h,
            rainfall_24h: rainfall_24h,
            soil_moisture: None,
            soil_temperature: None,
        }
    }
}

impl Deref for Weather {
    type Target = String;

    fn deref(&self) -> &Self::Target {
        match self {
            Self::Clear(s) => s,
            Self::Clouds(s) => s,
            Self::Drizzle(s) => s,
            Self::Rain(s) => s,
            Self::Snow(s) => s,
            Self::Thunderstorm(s) => s,
            Self::Atmosphere(a) => a,
        }
    }
}

impl Deref for Atmosphere {
    type Target = String;

    fn deref(&self) -> &Self::Target {
        match self {
            Self::Ash(s) => s,
            Self::Dust(s) => s,
            Self::Fog(s) => s,
            Self::Haze(s) => s,
            Self::Mist(s) => s,
            Self::Sand(s) => s,
            Self::SandDust(s) => s,
            Self::Smoke(s) => s,
            Self::Squall(s) => s,
            Self::Tornado(s) => s,
        }
    }
}

impl Weather {
    fn from_id(id: u16, s: String) -> Self {
        match id {
            200..300 => Self::Thunderstorm(s),
            300..400 => Self::Drizzle(s),
            500..600 => Self::Rain(s),
            600..700 => Self::Snow(s),
            700..800 => Self::Atmosphere(Atmosphere::from_id(id, s)),
            800 => Self::Clear(s),
            801..810 => Self::Clouds(s),
            invalid_id => {
                tracing::error!("Weather: Invalid Weather ID {}", invalid_id);
                panic!("Weather: Invalid Weather ID {}", invalid_id);
            }
        }
    }
}

impl Atmosphere {
    fn from_id(id: u16, s: String) -> Self {
        match id {
            701 => Self::Mist(s),
            711 => Self::Smoke(s),
            721 => Self::Haze(s),
            731 => Self::SandDust(s),
            741 => Self::Fog(s),
            751 => Self::Sand(s),
            761 => Self::Dust(s),
            762 => Self::Ash(s),
            771 => Self::Squall(s),
            781 => Self::Tornado(s),
            invalid_id => {
                tracing::error!("Atmosphere: Invalid Atmosphere ID {}", invalid_id);
                panic!("Atmosphere: Invalid Atmosphere ID {}", invalid_id);
            }
        }
    }
}

impl From<char> for DayTime {
    fn from(ch: char) -> Self {
        match ch.to_ascii_lowercase() {
            'd' => Self::Day,
            'n' => Self::Night,
            ch => panic!(
                "Failed to create `DayTime` from {}. `d` and `n` are valid.",
                ch
            ),
        }
    }
}
