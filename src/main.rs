use actix_web::{web, App, HttpServer};
use actix_web::middleware::Logger;
use std::sync::Arc;
use actix::Actor;
use clap::Parser;
use telemetry_events::models::{DBConnectionType, DBPool};
use telemetry_events::worker::ActorWorker;
use telemetry_events::routers::service_scope;
#[derive(Parser, Debug, Clone)]
#[clap(version, about)]

struct CliArgs {
    #[clap(short, long, help = "Enable server mode")]
    server: bool,
    
    #[clap(
        short = 'c',
        long,
        default_value = "p3a_db",
        help = "Main data channel to use. See README for details on data channel configuration."
    )]
    main_channel_name: String,
}
#[actix::main]
async fn main() -> std::io::Result<()> {
    dotenvy::dotenv().ok();
    env_logger::init();
    let cli_args = CliArgs::parse();
    let channel_name = DBConnectionType::Normal { channel_name: &*cli_args.main_channel_name.clone() };
    let db_pool = Arc::new(DBPool::new(channel_name).await);
 
    let worker_addr =  ActorWorker {
        pool: db_pool.clone(),
        buffer: Default::default(),
    }.start();

    HttpServer::new(move || {
        App::new()
            .wrap(Logger::default())
            .app_data(web::Data::new(worker_addr.clone()))
            .route("/", web::get().to(|| async {
                actix_web::HttpResponse::Ok()
                    .content_type("text/plain; charset=utf-8")
                    .body("Submission of privacy-preserving product analytics. See https://support.brave.com/hc/en-us/articles/9140465918093-What-is-P3A-in-Brave for details.")
            }))
            .service(service_scope()
            )

        // เพิ่ม service, middleware อื่น ๆ ของคุณตรงนี้
    })
        .bind(("0.0.0.0", 8011))?
        .run()
        .await
}