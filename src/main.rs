
use actix_web::{web, App, HttpServer};
use actix_web::middleware::Logger;
use sqlx::postgres::PgPoolOptions;
use lapin::{Connection, ConnectionProperties, options::QueueDeclareOptions, types::FieldTable};
use std::sync::Arc;
use actix::Actor;
use telemetry_events::queue_job::queue_job;
use telemetry_events::worker::{AppContext, RabbitMqWorker};



#[actix_web::main]
async fn main() -> std::io::Result<()> {
    dotenvy::dotenv().ok();
    env_logger::init();

    // Load environment variables
    let database_url = std::env::var("DATABASE_URL").expect("DATABASE_URL not set");
    let brave_service_key = std::env::var("BRAVE_SERVICE_KEY").expect("BRAVE_SERVICE_KEY not set");
    let rabbitmq_url = std::env::var("RABBITMQ_URL").expect("RABBITMQ_URL not set");

    // Setup PostgreSQL connection pool
    let pool = PgPoolOptions::new()
        .max_connections(5)
        .connect(&database_url)
        .await
        .expect("Failed to connect to DB");

    // Connect to RabbitMQ and prepare channel + queue
    let conn = Connection::connect(&rabbitmq_url, ConnectionProperties::default())
        .await
        .expect("Failed to connect to RabbitMQ");

    let channel = conn.create_channel()
        .await
        .expect("Failed to create channel");

    channel
        .queue_declare(
            "my_queue",
            QueueDeclareOptions::default(),
            FieldTable::default(),
        )
        .await
        .expect("Failed to declare queue");

    let arc_channel = Arc::new(channel);
   
    
    // Start the RabbitMqWorker as an actor
    let _worker_addr = RabbitMqWorker {
        channel: arc_channel.clone(),
        pool: pool.clone(),
    }
        .start();

    // Prepare AppContext and Actix Web server
    let app_context = AppContext {
        pool,
        brave_service_key,
        rabbit_channel: arc_channel,
    };

    HttpServer::new(move || {
        App::new()
            .wrap(Logger::default())
            .app_data(web::Data::new(app_context.clone()))
            // GET "/" ไม่ต้อง auth
            .route("/", web::get().to(|| async {
                actix_web::HttpResponse::Ok()
                    .content_type("text/plain; charset=utf-8")
                    .body("Submission of privacy-preserving product analytics. See https://support.brave.com/hc/en-us/articles/9140465918093-What-is-P3A-in-Brave for details.")
            }))
            // POST "/" ต้อง auth
            .service(
                web::resource("/")
                    .wrap(telemetry_events::AuthMiddleware::new())
                    .route(web::post().to(queue_job))
            )
        // เพิ่ม service, middleware อื่น ๆ ของคุณตรงนี้
    })
        .bind(("0.0.0.0", 8080))?
        .run()
        .await
}