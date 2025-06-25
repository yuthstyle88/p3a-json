// tests/api_tests.rs

use actix::Actor;
use actix_web::{test, web, App};
use lapin::{
    options::{BasicPublishOptions, QueueDeclareOptions},
    types::FieldTable,
    BasicProperties, Connection, ConnectionProperties,
};
use sqlx::{postgres::PgPoolOptions, Pool, Postgres};
use std::sync::Arc;
use aws_config::{load_defaults, BehaviorVersion};
use aws_sdk_dynamodb::Client as DynamoDbClient;
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

// Helper function to create RabbitMQ connection for tests
async fn setup_test_rabbitmq() -> Arc<lapin::Channel> {
    dotenvy::dotenv().ok();
    let rabbitmq_url = std::env::var("TEST_RABBITMQ_URL")
        .unwrap_or_else(|_| std::env::var("RABBITMQ_URL").expect("RABBITMQ_URL not set"));

    let conn = Connection::connect(&rabbitmq_url, ConnectionProperties::default())
        .await
        .expect("Failed to connect to RabbitMQ");

    let channel = conn
        .create_channel()
        .await
        .expect("Failed to create channel");

    // Create a test-specific queue
    let queue_name = "test_queue";
    channel
        .queue_declare(
            queue_name,
            QueueDeclareOptions {
                durable: true,
                auto_delete: false,
                ..QueueDeclareOptions::default()
            },
            FieldTable::default(),
        )
        .await
        .expect("Failed to declare test queue");

    Arc::new(channel)
}

// Setup function for the app context with test dependencies
 
 