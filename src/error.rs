use actix_web::{web, App, HttpResponse, HttpServer, Responder, ResponseError};
use thiserror::Error;


#[derive(Error, Debug)]
pub enum AppError {
    #[error("BadRequest error: {0}")]
    BadRequest(String),
    
    #[error("Invalid ID provided")]
    InvalidId,

    #[error("Database error: {0}")]
    DatabaseError(String),

    #[error("InternalError error: {0}")]
    InternalError(#[from] std::io::Error),

    #[error("Other error")]
    Other,
}

impl ResponseError for AppError {
    fn error_response(&self) -> HttpResponse {
        match self {
            AppError::BadRequest(_) => {
                HttpResponse::InternalServerError().body(self.to_string())
            },AppError::InvalidId => {
                HttpResponse::InternalServerError().body(self.to_string())
            },
            AppError::DatabaseError(_) => {
                HttpResponse::InternalServerError().body(self.to_string())
            },
            AppError::InternalError(_) => {
                HttpResponse::InternalServerError().body(self.to_string())
            },
            AppError::Other => { // เปลี่ยนตรงนี้!
                HttpResponse::InternalServerError().body(self.to_string())
            }
        }
    }
}
