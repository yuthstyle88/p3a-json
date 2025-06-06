
use actix_web::{web, App, HttpServer};
use actix_web::middleware::Logger;
use sqlx::postgres::PgPoolOptions;
use lapin::{Connection, ConnectionProperties, options::QueueDeclareOptions, types::FieldTable};
use std::sync::Arc;
use actix::Actor;
use telemetry_events::queue_job::queue_job;
use telemetry_events::worker::{AppContext, RabbitMqWorker};
use aws_config::BehaviorVersion;
use aws_types::region::Region;
use star_constellation::api::client;
use star_constellation::randomness::testing::LocalFetcher;
use telemetry_events::constellation::process_measurement;
use telemetry_events::update2::update2_json;


#[actix_web::main]
async fn main() -> std::io::Result<()> {
    dotenvy::dotenv().ok();
    env_logger::init();

    let random_fetcher = LocalFetcher::new();

    let measurements_1 = vec!["hello".as_bytes().to_vec(), "world".as_bytes().to_vec()];
    let epoch = 0u8;

    let rrs = client::prepare_measurement(&measurements_1, epoch).unwrap();
    let req = client::construct_randomness_request(&rrs);
    let req_slice_vec: Vec<&[u8]> = req.iter().map(|v| v.as_slice()).collect();
    let resp = random_fetcher.eval(&req_slice_vec, epoch).unwrap();

    println!("Random response: <cannot print, LocalFetcherResponse does not implement Debug>");


    // Load environment variables
    let database_url = std::env::var("DATABASE_URL").expect("DATABASE_URL not set");
    let brave_service_key = std::env::var("BRAVE_SERVICE_KEY").expect("BRAVE_SERVICE_KEY not set");
    let rabbitmq_url = std::env::var("RABBITMQ_URL").expect("RABBITMQ_URL not set");

    let endpoint = std::env::var("AWS_ENDPOINT").ok();
    let region = std::env::var("AWS_REGION").unwrap_or_else(|_| "us-east-1".to_string());
    let region = Region::new(region);
    let sdk_config = aws_config::defaults(BehaviorVersion::latest())
        .region(region)
        .load()
        .await;
    
    let db_config_builder = aws_sdk_dynamodb::config::Builder::from(&sdk_config);
    let db_config = if let Some(endpoint_url) = endpoint {
        db_config_builder.endpoint_url(endpoint_url).build()
    } else {
        db_config_builder.build()
    };

    let dynamodb_client = aws_sdk_dynamodb::Client::from_conf(db_config);
    
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
        dynamodb_client
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
                web::scope("/")
                    .wrap(telemetry_events::AuthMiddleware::new())
                    .service(
                        web::resource("")
                            .route(web::post().to(queue_job))
                    )
                    .service(
                        web::resource("update2/json")
                            .route(web::post().to(update2_json))
                    ).service(
                        web::resource("process")
                            .route(web::post().to(process_measurement))
                    )
                    
            )
        // เพิ่ม service, middleware อื่น ๆ ของคุณตรงนี้
    })
        .bind(("0.0.0.0", 8080))?
        .run()
        .await
}