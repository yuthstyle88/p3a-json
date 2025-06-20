use crate::constellation::utils::{
    calculate_epoch_express, calculate_epoch_slow, calculate_epoch_typical,
    next_epoch_time_express, next_epoch_time_slow, next_epoch_time_typical,
};
use crate::error::AppError;
use crate::model::public_key::PublicKey;
use crate::worker::AppContext;
use actix_web::{HttpResponse, Responder, web};
use base64::Engine;
use base64::prelude::BASE64_STANDARD;
use chrono::Utc;
use serde::{Deserialize, Serialize};
use star_constellation::api::server;
use star_constellation::randomness::testing::LocalFetcher as RandomnessFetcher;

#[derive(Deserialize, Clone)]
pub struct MeasureRequest {
    pub data: String,
    pub epoch: u8,
}
#[derive(Serialize, Clone)]
pub struct PublicKeyResponse {
    pub public_key: String,
    pub epoch: u8,
}

pub async fn process_instances_randomness(
    path: web::Path<String>,
    req: web::Json<MeasureRequest>,
) -> Result<impl Responder, AppError> {
    let msg = req.data.clone();
    let speed = path.into_inner();
    let threshold = 2;
    let epoch = 1;

    // เตรียม measurement จากข้อความ
    let _agg_res = server::aggregate(&[Vec::from(msg)], threshold, epoch, 2);

    Ok(HttpResponse::Ok().body(format!("speed : {}", speed)))
}

#[derive(Serialize, Clone)]
pub struct InstanceInfo {
    public_key: String,
    current_epoch: i64,
    next_epoch_time: String,
    max_points: u32,
}

pub async fn process_instances_info(
    path: web::Path<String>,
    ctx: web::Data<AppContext>,
) -> Result<impl Responder, actix_web::Error> {
    let speed = path.into_inner().to_lowercase();
    let now = Utc::now();
    let pool = &ctx.pool;
    let public_key = PublicKey::get_public_key(pool, &speed).await;
    let public_key = match public_key {
        Ok(p) => p.key,
        Err(_) => "Not found public key".to_string(),
    };

    let (current_epoch, next_epoch_time, max_points) = match speed.as_str() {
        "express" => (
            calculate_epoch_express(now),
            next_epoch_time_express(now).to_rfc3339(),
            1024,
        ),
        "typical" => (
            calculate_epoch_typical(now),
            next_epoch_time_typical(now).to_rfc3339(),
            1024,
        ),
        "slow" => (
            calculate_epoch_slow(now),
            next_epoch_time_slow(now).to_rfc3339(),
            1024,
        ),
        _ => {
            return Ok(HttpResponse::BadRequest().body("Unknown speed type"));
        }
    };

    let response = InstanceInfo {
        public_key,
        current_epoch,
        next_epoch_time,
        max_points,
    };

    Ok(HttpResponse::Ok().json(response))
}

#[cfg(test)]
mod tests {
    use super::*;
    use actix_web::{App, http::StatusCode, test, web};
    use serde_json::Value;

    #[actix_web::test]
    async fn test_process_instances_info_valid_speeds() {
        let app = test::init_service(
            App::new().route("/{speed}/info", web::get().to(process_instances_info)),
        )
        .await;

        for speed in ["express", "typical", "slow"] {
            let req = test::TestRequest::get()
                .uri(&format!("/{}/info", speed))
                .to_request();

            let resp = test::call_service(&app, req).await;
            assert_eq!(resp.status(), StatusCode::OK);

            let body: Value = test::read_body_json(resp).await;
            assert!(body.get("current_epoch").is_some());
            assert!(body.get("next_epoch_time").is_some());
            assert_eq!(body.get("max_points").unwrap(), 1024);
        }
    }

    #[test]
    fn basic_test() {
        let epoch = 0;
        let threshold = 1;
        let measurement = vec!["hello".as_bytes().to_vec(), "world".as_bytes().to_vec()];
        let random_fetcher = RandomnessFetcher::new();
        let pk_bincode = random_fetcher
            .get_server()
            .get_public_key()
            .serialize_to_bincode()
            .expect("Should serialize to bincode");
        let sss = BASE64_STANDARD.encode(&pk_bincode);
        let sss = BASE64_STANDARD.encode(&pk_bincode);
    }
}
