use actix_web::{web, Scope};
use crate::constellation::handlers::instances::{process_instances_info, process_instances_randomness};

pub fn instances_scope() -> Scope {
    web::scope("/instances")
        .route("/{speed}/info", web::get().to(process_instances_info))
        .route("/{speed}/randomness", web::post().to(process_instances_randomness))
}