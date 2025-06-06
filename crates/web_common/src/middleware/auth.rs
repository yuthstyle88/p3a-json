use crate::worker::AppContext;
use actix_web::body::BoxBody;
use actix_web::{
    Error, HttpResponse,
    dev::{ServiceRequest, ServiceResponse, Transform, forward_ready},
    web,
};
use futures_util::future::{LocalBoxFuture, Ready, ok};
use std::rc::Rc;

pub struct AuthMiddleware;

pub struct AuthMiddlewareMiddleware<S> {
    service: Rc<S>,
}

impl AuthMiddleware {
    pub fn new() -> Self {
        Self
    }
}

impl<S> Transform<S, ServiceRequest> for AuthMiddleware
where
    S: actix_service::Service<ServiceRequest, Response = ServiceResponse<BoxBody>, Error = Error>
        + 'static,
{
    type Response = ServiceResponse<BoxBody>;
    type Error = Error;
    type Transform = AuthMiddlewareMiddleware<S>;
    type InitError = ();
    type Future = Ready<Result<Self::Transform, Self::InitError>>;

    fn new_transform(&self, service: S) -> Self::Future {
        ok(AuthMiddlewareMiddleware {
            service: Rc::new(service),
        })
    }
}

impl<S> actix_service::Service<ServiceRequest> for AuthMiddlewareMiddleware<S>
where
    S: actix_service::Service<ServiceRequest, Response = ServiceResponse<BoxBody>, Error = Error>
        + 'static,
{
    type Response = ServiceResponse<BoxBody>;
    type Error = Error;
    type Future = LocalBoxFuture<'static, Result<Self::Response, Self::Error>>;

    forward_ready!(service);

    fn call(&self, req: ServiceRequest) -> Self::Future {
        let service = Rc::clone(&self.service);

        Box::pin(async move {
            // ดึง AppContext จาก request extensions
            if let Some(app_ctx) = req.app_data::<web::Data<AppContext>>() {
                let expected_key = &app_ctx.brave_service_key;
                if let Some(header) = req.headers().get("BraveServiceKey") {
                    if let Ok(header_str) = header.to_str() {
                        if header_str == expected_key {
                            return service.call(req).await;
                        }
                    }
                }
            }
            let (http_req, _) = req.into_parts();
            let response = HttpResponse::Unauthorized().finish().map_into_boxed_body();
            Ok(ServiceResponse::new(http_req, response))
        })
    }
}
