use crate::error::AppError;
use crate::omaha::detect_protocol_version;
use crate::payload::MyRequest;
use crate::update2::{Extension,  ResponseRoot,  extract_appid_and_version};
use crate::worker::AppContext;
use actix_web::{HttpResponse, Responder, web};
use serde_json::Value;

pub async fn update2_json(
    ctx: web::Data<AppContext>,
    item: web::Json<Value>,
) -> Result<impl Responder, AppError> {
    let maps = ctx.map.clone();
    let payload = item.into_inner();
    let exts = extract_appid_and_version(&payload);

    let request: MyRequest =
        serde_json::from_value(payload).map_err(|e| AppError::SerdeError(e.to_string()))?;
    let protocol = detect_protocol_version(&request);

    let items = Extension::filter_for_updates(&exts, &maps).await;

    let resp = ResponseRoot::to_json(&items, &protocol);
    let prefix = ")]}'\n";
    Ok(HttpResponse::Ok()
        .append_header(("Content-Type", "application/json"))
        .body(format!("{}{}", prefix, serde_json::to_string(&resp).unwrap())))
}
