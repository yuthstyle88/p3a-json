use crate::error::AppError;
use actix_web::{HttpResponse, Responder, web};
use serde::Deserialize;
use star_constellation::api::server;

#[derive(Deserialize, Clone)]
pub struct MeasureRequest {
    pub data: Vec<u8>,
    pub epoch: u8,
}

pub async fn process_collector_creative(
    req: web::Json<MeasureRequest>,
) -> Result<impl Responder, AppError> {
    let msg = req.data.clone();
    let threshold = 2;
    let epoch = 1;

    // เตรียม measurement จากข้อความ
    let agg_res = server::aggregate(&[msg], threshold, epoch, 2);

    Ok(HttpResponse::Ok().body(format!(
        "Random data points len: {}",
        agg_res.outputs().len()
    )))
}
pub async fn process_collector_slow(
    req: web::Json<MeasureRequest>,
) -> Result<impl Responder, AppError> {
    let msg = req.data.clone();
    let threshold = 2;
    let epoch = 1;

    // เตรียม measurement จากข้อความ
    let agg_res = server::aggregate(&[msg], threshold, epoch, 2);

    Ok(HttpResponse::Ok().body(format!(
        "Random data points len: {}",
        agg_res.outputs().len()
    )))
}
pub async fn process_collector_typical(
    req: web::Json<MeasureRequest>,
) -> Result<impl Responder, AppError> {
    let msg = req.data.clone();
    let threshold = 2;
    let epoch = 1;

    // เตรียม measurement จากข้อความ
    let agg_res = server::aggregate(&[msg], threshold, epoch, 2);

    Ok(HttpResponse::Ok().body(format!(
        "Random data points len: {}",
        agg_res.outputs().len()
    )))
}
pub async fn process_collector_express(
    req: web::Json<MeasureRequest>,
) -> Result<impl Responder, AppError> {
    let msg = req.data.clone();
    let threshold = 2;
    let epoch = 1;

    // เตรียม measurement จากข้อความ
    let agg_res = server::aggregate(&[msg], threshold, epoch, 2);

    Ok(HttpResponse::Ok().body(format!(
        "Random data points len: {}",
        agg_res.outputs().len()
    )))
}
