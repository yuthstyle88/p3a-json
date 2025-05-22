use crate::telemetry_event;
use sqlx::PgPool;
use std::sync::Once;
use dotenvy::dotenv;

static INIT: Once = Once::new();

pub async fn setup_test_db() -> PgPool {
    // Load .env.test if exists, fallback to .env
    INIT.call_once(|| {
        dotenvy::from_filename(".env.test").ok();
        dotenv().ok();
    });

    let database_url = std::env::var("DATABASE_URL")
        .expect("DATABASE_URL must be set in .env.test or .env");

    let pool = PgPool::connect(&database_url)
        .await
        .expect("Failed to connect to database");

    // Clear test database
    sqlx::query!("TRUNCATE TABLE telemetry_events")
        .execute(&pool)
        .await
        .expect("Failed to clear test database");

    pool
}

pub fn get_test_event() -> TelemetryEvent {
    TelemetryEvent {
        id: None,
        cadence: "daily".to_string(),
        channel: "release".to_string(),
        country_code: "TH".to_string(),
        metric_name: "test_metric".to_string(),
        metric_value: 42,
        platform: "desktop".to_string(),
        version: "1.0.0".to_string(),
        woi: 1,
        wos: 2,
        yoi: 2024,
        yos: 12,
        received_at: Some(chrono::Utc::now()),
    }
}