use actix_web::{web, Scope};
use crate::constellation::handlers::collector::{process_collector_creative, process_collector_express, process_collector_slow, process_collector_typical};

pub fn collector_scope() -> Scope {
    web::scope("/collector")
        .route("/creative", web::post().to(process_collector_creative))
        .route("/slow", web::post().to(process_collector_slow))
        .route("/typical", web::post().to(process_collector_typical))
        .route("/express", web::post().to(process_collector_express))
}