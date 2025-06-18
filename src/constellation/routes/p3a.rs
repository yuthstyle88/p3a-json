use actix_web::{web, Scope};
use crate::constellation::handlers::p3a::p3a_creative;

pub fn p3a_scope() -> Scope {
    web::scope("/p3a")
        .route("/creative", web::post().to(p3a_creative))
    
}