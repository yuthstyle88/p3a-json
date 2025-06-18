use actix_web::{web::self};
use actix_web::dev::HttpServiceFactory;
use crate::AuthMiddleware;
use crate::constellation::handlers::collector::process_collector_express;
use crate::constellation::handlers::instances::{process_instances_info, process_instances_randomness};
use crate::queue_job::queue_job;
use crate::update2::importer_data_from_json;

pub fn constellation_scope() -> impl HttpServiceFactory {
    web::scope("/api/v1")
        .wrap(AuthMiddleware::new())
        .route("/p3a", web::post().to(queue_job))
        .route("/import", web::get().to(importer_data_from_json))
        .route("/{speed}", web::post().to(process_collector_express))
        .service(
            web::scope("/instances")
                .route("/{speed}/info", web::post().to(process_instances_info)),
        )
        .service(
            web::scope("/collector")
                .route("/{speed}/randomness", web::post().to(process_instances_randomness)),
        )
}