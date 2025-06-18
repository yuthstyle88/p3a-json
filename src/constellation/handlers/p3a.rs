use crate::error::AppError;
use actix_web::{HttpResponse, Responder, web};
use serde::Deserialize;
use star_constellation::api::server;

#[derive(Deserialize, Clone)]
pub struct MeasureRequest {
    pub data: Vec<u8>,
    pub epoch: u8,
}

pub async fn p3a_creative(req: web::Json<MeasureRequest>) -> Result<impl Responder, AppError> {
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