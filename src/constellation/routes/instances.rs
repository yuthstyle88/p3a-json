use actix_web::{web, Scope};
use crate::constellation::handlers::instances::{process_instances_info, process_instances_randomness};

pub fn instances_scope() -> Scope {
    web::scope("/instances")
        .route("/creative", web::post().to(process_instances_randomness))
        .route("/slow", web::post().to(process_instances_info))
        .route("/typical", web::post().to(process_instances_info))
        .route("/express", web::post().to(process_instances_info))
}