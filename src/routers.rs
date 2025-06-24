use actix_web::{web::self};
use actix_web::dev::HttpServiceFactory;
use crate::AuthMiddleware;
use crate::queue_job::queue_job;


pub fn service_scope() -> impl HttpServiceFactory {
    web::scope("/api/v1")
        // .wrap(AuthMiddleware::new())
        .route("/{channel}", web::post().to(queue_job))
        
}