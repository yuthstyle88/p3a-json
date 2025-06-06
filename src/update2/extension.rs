use actix_web::{web, HttpResponse, Responder};
use aws_sdk_dynamodb::types::AttributeValue;
use crate::error::AppError;
use crate::worker::AppContext;
use crate::payload::{MyExtensionPayload};

pub async fn update2_json(
    ctx: web::Data<AppContext>,
    item: web::Json<MyExtensionPayload>,
) -> Result<impl Responder, AppError> {
    let client = &ctx.dynamodb_client;
    let _payload = item.into_inner();

    let table_name = "Extensions";
    let id = "ID".to_string();

    
    let result = client
        .get_item()
        .table_name(table_name)
        .key("ID", AttributeValue::S(id.clone()))
        .send()
        .await
        .map_err(|e| AppError::DatabaseError(e.to_string()))?;

    Ok(HttpResponse::Ok().json("app data"))
}
