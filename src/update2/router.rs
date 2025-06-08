use crate::error::AppError;
use crate::update2::{batch_get_items_by_ids, extract_appids, App, Extension, Response, ResponseRoot};
use crate::worker::AppContext;
use actix_web::{HttpResponse, Responder, web};
use aws_sdk_dynamodb::types::AttributeValue;
use serde_json::Value;

pub async fn update2_json(
    ctx: web::Data<AppContext>,
    item: web::Json<Value>,
) -> Result<impl Responder, AppError> {
    let client = &ctx.dynamodb_client;
    let maps = ctx.map.clone();
    let payload = item.into_inner();

    // ตัวอย่าง: อัปเดตฟิลด์หนึ่งใน DynamoDB
    let table_name = "Extensions";
    let keys = extract_appids(&payload);

    let items = batch_get_items_by_ids(&client, table_name, keys).await?;

    let res =  ResponseRoot::to_json(&items);
    // Convert each DynamoDB item to a serializable serde_json::Value map
  
    Ok(HttpResponse::Ok().json(res))
}

fn attribute_value_to_json(av: &AttributeValue) -> Value {
    match av {
        AttributeValue::S(s) => Value::String(s.clone()),
        AttributeValue::N(n) => {
            if let Ok(i) = n.parse::<i64>() {
                Value::Number(i.into())
            } else if let Ok(f) = n.parse::<f64>() {
                serde_json::Number::from_f64(f)
                    .map(Value::Number)
                    .unwrap_or(Value::Null)
            } else {
                Value::Null
            }
        }
        AttributeValue::Bool(b) => Value::Bool(*b),
        AttributeValue::L(list) => Value::Array(list.iter().map(attribute_value_to_json).collect()),
        AttributeValue::M(map) => {
            let obj: serde_json::Map<String, Value> = map
                .iter()
                .map(|(k, v)| (k.clone(), attribute_value_to_json(v)))
                .collect();
            Value::Object(obj)
        }
        _ => Value::Null,
    }
}
