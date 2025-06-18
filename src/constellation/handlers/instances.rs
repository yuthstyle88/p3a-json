use crate::error::AppError;
use actix_web::{HttpResponse, Responder, web};
use serde::Deserialize;
use star_constellation::api::server;

#[derive(Deserialize, Clone)]
pub struct MeasureRequest {
    pub data: Vec<u8>,
    pub epoch: u8,
}

pub async fn process_instances_randomness(path: web::Path<String>,
    req: web::Json<MeasureRequest>,
) -> Result<impl Responder, AppError> {
    let msg = req.data.clone();
    let speed = path.into_inner();
    let threshold = 2;
    let epoch = 1;

    // เตรียม measurement จากข้อความ
    let _agg_res = server::aggregate(&[msg], threshold, epoch, 2);

    Ok(HttpResponse::Ok().body(format!(
         "speed : {}",
         speed
    )))
}
pub async fn process_instances_info(path: web::Path<String>) -> Result<impl Responder, AppError> {
    let speed = path.into_inner();
    let public_key = "public_key";

    Ok(HttpResponse::Ok().body(format!(
        "public_key : {}, speed : {}",
        public_key, speed
    )))
}
