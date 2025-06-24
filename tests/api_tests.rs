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
    worker::{AppContext, RabbitMqWorker},
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
async fn setup_test_context() -> AppContext {
    let pool = setup_test_db().await;
    let brave_service_key = std::env::var("BRAVE_SERVICE_KEY")
        .unwrap_or_else(|_| "test_brave_service_key".to_string());
    let rabbit_channel = setup_test_rabbitmq().await;
    let config = load_defaults(BehaviorVersion::latest()).await;
    let dynamodb_client = DynamoDbClient::new(&config);
    AppContext {
        pool,
        brave_service_key,
        rabbit_channel,

    }
}

async fn create_test_app(
    ctx: web::Data<AppContext>,
) -> impl actix_web::dev::Service<
    actix_http::Request,
    Response = actix_web::dev::ServiceResponse,
    Error = actix_web::Error,
> {
    test::init_service(
        App::new()
            .app_data(ctx)
            .route("/", web::post().to(queue_job)),
    )
    .await
}



#[actix_rt::test]
async fn test_worker_processes_messages() {
    // Initialize logger for better debugging
    let _ = env_logger::builder()
        .filter_level(log::LevelFilter::Debug)
        .try_init();

    // Setup test context
    let ctx = setup_test_context().await;

    // Start the worker as an actor
    let worker = RabbitMqWorker {
        channel: ctx.rabbit_channel.clone(),
        pool: ctx.pool.clone(),
        buffer: Default::default(),
    }
        .start();

    // Create a sample payload
    let test_payload = MyPayload {
        cadence: "test".to_string(),
        channel: "test".to_string(),
        country_code: "th".to_string(),
        metric_name: "app".to_string(),
        metric_value: 100,
        platform: "ios".to_string(),
        version: "1.78".to_string(),
        woi: 22,
        wos: Some(22),
        yoi: 2025,
        yos: 2025,
    };

    // Create test app
    let app = create_test_app(web::Data::new(ctx.clone())).await;

    // Send a test request to queue a job
    let req = test::TestRequest::post()
        .uri("/")
        .set_json(&test_payload)
        .to_request();

    let resp = test::call_service(&app, req).await;

    // Verify the response
    assert!(resp.status().is_success());

    // Give the worker some time to process the message
    tokio::time::sleep(tokio::time::Duration::from_millis(500)).await;

    // Here you would typically verify that the worker processed the message
    // This could involve checking a database for changes, or other side effects

    // For example, if your worker writes to the database:
    // let result = sqlx::query!("SELECT * FROM processed_events WHERE event_id = $1", test_payload.event_id)
    //    .fetch_optional(&ctx.pool)
    //    .await
    //    .expect("Database query failed");
    // 
    // assert!(result.is_some(), "Worker did not process the message");
}

#[actix_rt::test]
async fn test_worker_handles_invalid_messages() {
    // Initialize logger
    let _ = env_logger::builder()
        .filter_level(log::LevelFilter::Debug)
        .try_init();

    // Setup test context
    let ctx = setup_test_context().await;

    // Start the worker
    let worker = RabbitMqWorker {
        channel: ctx.rabbit_channel.clone(),
        pool: ctx.pool.clone(),
        buffer: Default::default(),
    }
        .start();

    // Publish an invalid message directly to the queue
    let invalid_payload = b"{\"invalid\": \"json\"}";

    ctx.rabbit_channel
        .basic_publish(
            "",
            "test_queue",
            BasicPublishOptions::default(),
            invalid_payload,
            BasicProperties::default(),
        )
        .await
        .expect("Failed to publish invalid message");

    // Give the worker time to process
    tokio::time::sleep(tokio::time::Duration::from_millis(500)).await;

    // The test passes if the worker doesn't crash
    // Ideally, you would check logs or a dead letter queue
}