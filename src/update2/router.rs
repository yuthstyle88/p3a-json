use crate::error::AppError;
use crate::update2::{batch_get_items_by_ids, extract_appids, App, Extension, Response, ResponseRoot};
use crate::worker::AppContext;
use actix_web::{HttpResponse, Responder, web};
use serde_json::Value;
use crate::omaha::detect_protocol_version;
use crate::payload::MyRequest;

pub async fn update2_json(
    ctx: web::Data<AppContext>,
    item: web::Json<Value>,
) -> Result<impl Responder, AppError> {
    let client = &ctx.dynamodb_client;
    let maps = ctx.map.clone();
    let payload = item.into_inner();
    let table_name = "Extensions";
    let keys = extract_appids(&payload);

    let items = batch_get_items_by_ids(&client, table_name, keys).await?;
    
    let request: MyRequest = serde_json::from_value(payload)
        .map_err(|e| AppError::SerdeError(e.to_string()))?;
    let protocol = detect_protocol_version(&request);
    
    let items = Extension::filter_for_updates(&items, &maps).await;
    let resp =  ResponseRoot::to_json(&items, &protocol);
    Ok(HttpResponse::Ok().json(resp))
}