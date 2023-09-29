mod handler;

use handler::{
    WeatherResponse,
    WeatherDisplay,
    IndexTemplate,
    LatLong,
    WeatherQuery,
    GeoResponse,
    StrError
};
use sqlx::PgPool;
use axum::extract::{Query, State};
use reqwest::StatusCode;
use std::net::SocketAddr;
use axum::{routing::get, Router};

async fn index() -> IndexTemplate {
    IndexTemplate
}


#[axum_macros::debug_handler]
async fn stats() -> &'static str {
    "stats"
}

async fn get_lat_long(pool: &PgPool, name: &str) -> Result<LatLong, Box<dyn std::error::Error>> {
    let lat_long = sqlx::query_as::<_, LatLong>(
        "SELECT lat AS latitude, long AS longitude FROM cities WHERE name = $1",
        )
        .bind(name)
        .fetch_optional(pool)
        .await?;

    if let Some(lat_long) = lat_long {
        return Ok(lat_long);
    }

    let lat_long = fetch_lat_long(name).await?;
    sqlx::query("INSERT INTO cities (name, lat, long) VALUES ($1, $2, $3)")
        .bind(name)
        .bind(lat_long.latitude)
        .bind(lat_long.longitude)
        .execute(pool)
        .await?;
    Ok(lat_long)
}

async fn fetch_lat_long(city: &str) -> Result<LatLong, Box<dyn std::error::Error>> {
	let endpoint = format!(
    	"https://geocoding-api.open-meteo.com/v1/search?name={}&count=1&language=en&format=json",
    	city
	);

	let response = reqwest::get(&endpoint).await?.json::<GeoResponse>().await?;
	response
    	.results
    	.get(0)
    	.cloned()
    	.ok_or("No results found".into())

    //match response.results.get(0) {
    //    Some(lat_long) => Ok(lat_long.clone()),
    //    None => Err("No results found".into()),
    //}
}

async fn fetch_weather(lat_long: LatLong) -> Result<WeatherResponse, Box<dyn std::error::Error>> {
    let endpoint = format!(
    	"https://api.open-meteo.com/v1/forecast?latitude={}&longitude={}&hourly=temperature_2m",
        lat_long.latitude, lat_long.longitude
        );
    let response = reqwest::get(&endpoint).await?.json::<WeatherResponse>().await?;
    Ok(response)
}

async fn weather(
    Query(params): Query<WeatherQuery>,
    State(pool): State<PgPool>
) -> Result<WeatherDisplay, StatusCode>{
    //match fetch_lat_long(&params.city).await {
    //    Ok(lat_long) => Ok(
    //        format!("{}: {}, {}", params.city, lat_long.latitude, lat_long.longitude)
    //        ),
    //    Err(_) => Err(StatusCode::NOT_FOUND),

    //}
    let lat_long = get_lat_long(&pool, &params.city)
        .await
        .map_err(|_| StatusCode::NOT_FOUND)?;

    let weather = fetch_weather(lat_long)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    Ok(WeatherDisplay::new(params.city, weather))
}

async fn establish_db_pool() -> Result<PgPool, Box<dyn std::error::Error>> {
    let db_connection_str = std::env::var("DATABASE_URL").map_err(|_| StrError("DATABASE_URL must be set".to_string()))?;
	let pool = sqlx::PgPool::connect(&db_connection_str)
    	.await
        .map_err(|err| StrError(err.to_string()))?;
    Ok(pool)
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let pool = establish_db_pool().await?;
    let app = Router::new()
        .route("/", get(index))
        .route("/weather", get(weather))
        .route("/stats", get(stats))
        .with_state(pool);

    let addr = SocketAddr::from(([127, 0, 0, 1], 3000));
    axum::Server::bind(&addr)
        .serve(app.into_make_service())
        .await
        .unwrap();
    Ok(())
}
