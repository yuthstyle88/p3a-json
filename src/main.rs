use actix_web::{web, App, HttpServer, middleware::Logger};
use sqlx::postgres::PgPoolOptions;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    dotenvy::dotenv().ok();
    env_logger::init();
    let database_url = std::env::var("DATABASE_URL").expect("DATABASE_URL not set");
    let brave_service_key = std::env::var("BRAVE_SERVICE_KEY").expect("BRAVE_SERVICE_KEY not set");

    let pool = PgPoolOptions::new()
        .max_connections(5)
        .connect(&database_url)
        .await
        .expect("Failed to connect to DB");

    HttpServer::new(move || {
        App::new()
            .wrap(telemetry_events::AuthMiddleware::new(brave_service_key.clone()))
            .wrap(Logger::default())
            .app_data(web::Data::new(pool.clone()))
            .route("/", web::post().to(telemetry_events::insert_event))
    })
        .bind(("0.0.0.0", 8080))?
        .run()
        .await
}