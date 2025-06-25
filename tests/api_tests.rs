// tests/api_tests.rs

use actix::Actor;
use actix_web::{test, web, App};

use sqlx::{postgres::PgPoolOptions, Pool, Postgres};
use std::sync::Arc;

use telemetry_events::{
    payload::MyPayload,
    worker::{ActorWorker},
};
use telemetry_events::queue_job::queue_job;

// Helper function to create test database pool
async fn setup_test_db() -> Pool<Postgres> {
    // Use a test-specific database URL or environment variable
    dotenvy::dotenv().ok();
    let database_url = std::env::var("TEST_DATABASE_URL")
        .unwrap_or_else(|_| std::env::var("DATABASE_URL").expect("DATABASE_URL not set"));

    PgPoolOptions::new()
        .max_connections(2)
        .connect(&database_url)
        .await
        .expect("Failed to connect to test database")
}
 
// Setup function for the app context with test dependencies
 
 