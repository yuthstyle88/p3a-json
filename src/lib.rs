use actix_web::{web, HttpResponse, Result};
use actix_web::dev::{ServiceRequest, ServiceResponse, Transform, forward_ready};
use actix_web::Error;
use futures_util::future::{ok, LocalBoxFuture, Ready};
use sqlx::PgPool;

use std::rc::Rc;
use actix_web::body::{BoxBody};


// Export public items
pub use crate::models::TelemetryEvent;
pub use crate::middleware::AuthMiddleware;
pub use crate::handlers::insert_event;

// Modules
pub mod models {
    use serde::{Deserialize, Serialize};
    use chrono;

    #[derive(Debug, Clone, Deserialize, Serialize)]
    pub struct TelemetryEvent {
        pub id: Option<i64>,
        pub cadence: String,
        pub channel: String,
        pub country_code: String,
        pub metric_name: String,
        pub metric_value: i32,
        pub platform: String,
        pub version: String,
        pub woi: i16,
        pub wos: i16,
        pub yoi: i16,
        pub yos: i16,
        pub received_at: Option<chrono::DateTime<chrono::Utc>>,
    }
}

pub mod middleware {
    use super::*;

    pub struct AuthMiddleware {
        pub expected_key: String,
    }

    pub struct AuthMiddlewareMiddleware<S> {
        service: Rc<S>,
        expected_key: String,
    }

    impl AuthMiddleware {
        pub fn new(expected_key: impl Into<String>) -> Self {
            Self {
                expected_key: expected_key.into(),
            }
        }
    }

    impl<S> Transform<S, ServiceRequest> for AuthMiddleware
    where
        S: actix_service::Service<ServiceRequest, Response = ServiceResponse<BoxBody>, Error = Error> + 'static,
    {
        type Response = ServiceResponse<BoxBody>;
        type Error = Error;
        type Transform = AuthMiddlewareMiddleware<S>;
        type InitError = ();
        type Future = Ready<Result<Self::Transform, Self::InitError>>;

        fn new_transform(&self, service: S) -> Self::Future {
            ok(AuthMiddlewareMiddleware {
                service: Rc::new(service),
                expected_key: self.expected_key.clone(),
            })
        }
    }

    impl<S> actix_service::Service<ServiceRequest> for AuthMiddlewareMiddleware<S>
    where
        S: actix_service::Service<ServiceRequest, Response = ServiceResponse<BoxBody>, Error = Error> + 'static,
    {
        type Response = ServiceResponse<BoxBody>;
        type Error = Error;
        type Future = LocalBoxFuture<'static, Result<Self::Response, Self::Error>>;

        forward_ready!(service);

        fn call(&self, req: ServiceRequest) -> Self::Future {
            let service = Rc::clone(&self.service);
            let expected_key = self.expected_key.clone();

            Box::pin(async move {
                if let Some(header) = req.headers().get("BraveServiceKey") {
                    if let Ok(header_str) = header.to_str() {
                        if header_str == expected_key {
                            return service.call(req).await;
                        }
                    }
                }
                let (http_req, _) = req.into_parts();
                let response = HttpResponse::Unauthorized()
                    .finish()
                    .map_into_boxed_body();
                Ok(ServiceResponse::new(http_req, response))
            })
        }
    }
}

pub mod handlers {
    use super::*;
    use crate::models::TelemetryEvent;

    pub async fn insert_event(pool: web::Data<PgPool>, item: web::Json<TelemetryEvent>) -> Result<HttpResponse> {
        sqlx::query!(
            "INSERT INTO telemetry_events (
                cadence, channel, country_code, metric_name, metric_value,
                platform, version, woi, wos, yoi, yos, received_at
            ) VALUES (
                $1, $2, $3, $4, $5,
                $6, $7, $8, $9, $10, $11, $12
            )",
            item.cadence,
            item.channel,
            item.country_code,
            item.metric_name,
            item.metric_value,
            item.platform,
            item.version,
            item.woi,
            item.wos,
            item.yoi,
            item.yos,
            item.received_at.unwrap_or_else(chrono::Utc::now)
        )
            .execute(pool.get_ref())
            .await
            .map_err(actix_web::error::ErrorInternalServerError)?;

        Ok(HttpResponse::Ok().json("ok"))
    }
}