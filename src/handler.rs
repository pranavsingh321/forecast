use std::f64;

use askama::Template;
use serde::Deserialize;

#[derive(serde::Deserialize)]
pub struct GeoResponse {
    pub results: Vec<LatLong>,
}

#[derive(sqlx::FromRow, Deserialize, Debug, Clone)]
pub struct LatLong {
    pub latitude: f64,
    pub longitude: f64,
}

#[derive(Deserialize)]
pub struct WeatherQuery {
    pub city: String,
}

#[derive(Deserialize, Debug)]
pub struct WeatherResponse {
    pub latitude: f64,
    pub longitude: f64,
    pub timezone: String,
    pub hourly: Hourly,
}

#[derive(Deserialize, Debug)]
pub struct Hourly {
    pub time: Vec<String>,
    pub temperature_2m: Vec<f64>
}

#[derive(Template, Deserialize, Debug)]
#[template(path="weather.html")]
pub struct WeatherDisplay {
    pub city: String,
    pub forecasts: Vec<Forecast>,
}

#[derive(Template)]
#[template(path="index.html")]
pub struct IndexTemplate;

#[derive(Deserialize, Debug)]
pub struct Forecast {
    pub date: String,
    pub temperature: String,
}

impl WeatherDisplay {
    pub fn new (city: String, response: WeatherResponse) -> Self{
        let display = WeatherDisplay {
            city,
            forecasts: response
                .hourly
                .time
                .iter()
                .zip(response.hourly.temperature_2m.iter())
                .map(|(date, temperature)| Forecast {
                    date: date.to_string(),
                    temperature: temperature.to_string(),
                })
            .collect(),
        };
        display
    }
}

#[derive(Debug)]
pub struct StrError(pub String);

impl std::fmt::Display for StrError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl std::error::Error for StrError {}
