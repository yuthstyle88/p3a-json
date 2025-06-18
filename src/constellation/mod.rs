pub mod handlers;
pub mod routes;
pub mod types;


use actix_web::Scope;
use actix_web::web;

pub fn constellation_scope() -> Scope {
    web::scope("/constellation")
        .service(routes::collector::collector_scope())
        .service(routes::instances::instances_scope())
        .service(routes::p3a::p3a_scope())
}