use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, PartialOrd, Serialize, Deserialize)]
pub struct Forecast {
    pub forecasts: Vec<ForecastSegment>,
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
    /// %, 0 - 100
    pub humidity: u8,
    /// Weather conditions that are active.
    pub weather: Vec<String>,
    /// Cloudiness
    ///
    /// %, 0 - 100
    pub clouds: u8,
    pub wind: Wind,
    pub visibility: Option<u16>,
    /// Probability of precipitation.
    ///
    /// %, 0 - 100
    pub pop: u8,
    /// Rain volume in mm/X hours.
    pub rain: Option<u16>,
    /// Snow volume in mm/X hours.
    pub snow: Option<u16>,
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

impl From<crate::forecast::Forecast> for Forecast {
    fn from(lfc: crate::forecast::Forecast) -> Self {
        let mut forecasts: Vec<ForecastSegment> = lfc
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
                    humidity: fc_main.humidity,
                    weather: {
                        let len = fc.weather.len();
                        let mut weather =
                            fc.weather
                                .into_iter()
                                .fold(Vec::with_capacity(len), |mut acc, w| {
                                    acc.push(w.description);

                                    acc
                                });
                        weather.shrink_to_fit();
                        weather
                    },
                    clouds: fc.clouds.all,
                    wind: Wind {
                        speed: fc.wind.speed,
                        deg: fc.wind.deg,
                        gust: fc.wind.gust,
                    },
                    visibility: fc.visibility,
                    pop: (fc.pop * 100.0) as u8,
                    rain: fc.rain.map(|r| r.three_hours),
                    snow: fc.snow.map(|r| r.three_hours),
                }
            })
            .collect();
        forecasts.shrink_to_fit();

        Self { forecasts }
    }
}
