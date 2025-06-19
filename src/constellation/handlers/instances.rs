use crate::error::AppError;
use actix_web::{HttpResponse, Responder, web};
use anyhow::Error;
use serde::{Deserialize, Serialize};
use sqlx::PgPool;
use star_constellation::api::server;
use crate::worker::AppContext;

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

pub async fn process_instances_randomness(path: web::Path<String>,
    req: web::Json<MeasureRequest>,
) -> Result<impl Responder, AppError> {
    let msg = req.data.clone();
    let speed = path.into_inner();
    let threshold = 2;
    let epoch = 1;

    // เตรียม measurement จากข้อความ
    let _agg_res = server::aggregate(&[Vec::from(msg)], threshold, epoch, 2);
 
    Ok(HttpResponse::Ok().body(format!(
         "speed : {}",
         speed
    )))
}
pub async fn process_instances_info(
    path: web::Path<String>,
    ctx: web::Data<AppContext>,
) -> Result<impl Responder, AppError> {
    let speed = path.into_inner();
    let pool = &ctx.pool;
    let public_key = get_public_key(pool, &speed).await;
    let public_key = match  public_key{
        Ok(p) => p.key,
        Err(_) => "Not found public key".to_string(),
    };
    let epoch= 3;
    let resp = PublicKeyResponse{ public_key, epoch };
    Ok(HttpResponse::Ok().json(resp))
}
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct PublicKey {
    id: i32,
    key: String,
    speed: String
}
async fn get_public_key(pool: &PgPool, speed: &str) -> Result<PublicKey, anyhow::Error> {
    let public_key = sqlx::query_as!(
        PublicKey,
        r#"
        SELECT id, key, speed
        FROM public_keys
        WHERE
            speed = $1
        "#,
        speed
    )
        .fetch_one(pool)
        .await?;
    Ok(public_key)
}
#[cfg(test)]
mod tests {
    use base64::Engine;
    use star_constellation::randomness::testing::{LocalFetcher as RandomnessFetcher};
    use base64::prelude::BASE64_STANDARD;

    #[test]
    fn basic_test() {
        let epoch = 0;
        let threshold = 1;
        let measurement =
            vec!["hello".as_bytes().to_vec(), "world".as_bytes().to_vec()];
        let random_fetcher = RandomnessFetcher::new();
        let pk_bincode = random_fetcher.get_server().get_public_key().serialize_to_bincode().expect("Should serialize to bincode");
        let sss = BASE64_STANDARD.encode(&pk_bincode);
    }
}