use std::collections::HashMap;
use crate::error::AppError;
use crate::update2::model::{App, Extension};
use crate::worker::AppContext;
use actix_web::{HttpResponse, Responder, web};
use aws_sdk_dynamodb::error::{ProvideErrorMetadata, SdkError};
use aws_sdk_dynamodb::operation::get_item::GetItemOutput;
use aws_sdk_dynamodb::types::{AttributeDefinition, AttributeValue, KeySchemaElement, KeyType, KeysAndAttributes, ProvisionedThroughput, ScalarAttributeType};
use serde_json::Value;
use std::fs::File;
use std::io::Read;
use aws_sdk_dynamodb::Client;
use chrono::Utc;

pub async fn importer_data_from_json(
    ctx: web::Data<AppContext>,
) -> Result<impl Responder, AppError> {
    let client = &ctx.dynamodb_client;
    let _ = is_not_exits_create_table(&client.clone()).await;
    let _ = insert_extensions(&client).await;
    Ok(HttpResponse::Ok().json("success"))
}
pub async fn is_not_exits_create_table(client: &aws_sdk_dynamodb::Client) -> Result<(), AppError> {
    let table_name = "Extensions";
    let table_exists = match client.describe_table().table_name(table_name).send().await {
        Ok(_) => true,
        Err(err) => match &err {
            SdkError::ServiceError { .. } if err.code() == Some("ResourceNotFoundException") => {
                false
            }
            _ => return Err(AppError::DatabaseError("DatabaseError".to_string())),
        },
    };

    if table_exists {
        return Ok(());
    }

    let attr_def = AttributeDefinition::builder()
        .attribute_name("ID")
        .attribute_type(ScalarAttributeType::S)
        .build()
        .map_err(|e| AppError::DatabaseError(e.to_string()))?;

    let key_schema = KeySchemaElement::builder()
        .attribute_name("ID")
        .key_type(KeyType::Hash)
        .build()
        .map_err(|e| AppError::DatabaseError(e.to_string()))?;

    let provisioned_throughput = ProvisionedThroughput::builder()
        .read_capacity_units(5)
        .write_capacity_units(5)
        .build()
        .map_err(|e| AppError::DatabaseError(e.to_string()))?;

    client
        .create_table()
        .table_name(table_name)
        .attribute_definitions(attr_def)
        .key_schema(key_schema)
        .provisioned_throughput(provisioned_throughput)
        .send()
        .await
        .map_err(|e| AppError::DatabaseError(e.to_string()))?;

    Ok(())
}

pub async fn insert_extensions(client: &aws_sdk_dynamodb::Client) -> Result<(), AppError> {
    let table_name = "Extensions";
    let mut file = File::open("./extensions.json")?;
    let mut data = String::new();
    file.read_to_string(&mut data)?;

    // แปลง String เป็น Value สำหรับ parse
    let records: Vec<Value> =
        serde_json::from_str(&data).map_err(|e| AppError::BadRequest(e.to_string()))?;

    for record in &records {
        if let Some(response) = record.get("response").and_then(|resp| resp.get("app")) {
            let mut item = std::collections::HashMap::new();
            let app: App = App::from_value(&response[0]).unwrap();
            let ext = Extension::from(app);
            item.insert("ID".to_string(), AttributeValue::S(ext.id.clone()));
            item.insert("COHORT".to_string(), AttributeValue::S(ext.cohort.clone()));
            item.insert(
                "COHORTNAME".to_string(),
                AttributeValue::S(ext.cohortname.clone()),
            );
            item.insert(
                "NAME".to_string(),
                AttributeValue::S(ext.package_name.clone()),
            );
            item.insert(
                "VERSION".to_string(),
                AttributeValue::S(ext.version.clone()),
            );
            item.insert(
                "HASH_SHA256".to_string(),
                AttributeValue::S(ext.hash_sha256.clone()),
            );
            item.insert("FP".to_string(), AttributeValue::S(ext.fp.clone()));
            item.insert("BLACKLISTED".to_string(), AttributeValue::Bool(ext.blacklisted.clone()));
            item.insert("REQUIRED".to_string(), AttributeValue::Bool(ext.required.clone()));
            item.insert("HASH".to_string(), AttributeValue::S(ext.hash.clone()));
            item.insert("SIZE".to_string(), AttributeValue::N(ext.size.to_string()));
            item.insert("CREATED_AT".to_string(), AttributeValue::S(Utc::now().naive_local().to_string()));
            item.insert("UPDATE_AT".to_string(), AttributeValue::S(Utc::now().naive_local().to_string()));
            client
                .put_item()
                .table_name(table_name)
                .set_item(Some(item))
                .send()
                .await
                .map_err(|e| AppError::DatabaseError(e.to_string()))?;
        }
    }
    Ok(())
}

pub async fn batch_get_items_by_ids(
    client: &Client,
    table_name: &str,
    ids: Vec<String>
) -> Result<Vec<HashMap<String, AttributeValue>>, AppError> {
    let keys: Vec<HashMap<String, AttributeValue>> = ids
        .into_iter()
        .map(|id| {
            let mut key = HashMap::new();
            key.insert("ID".to_string(), AttributeValue::S(id));
            key
        })
        .collect();

    let keys_and_attrs = KeysAndAttributes::builder()
        .set_keys(Some(keys))
        .build()
        .map_err(|e| AppError::DatabaseError(e.to_string()))?;

    let mut req_items = HashMap::new();
    req_items.insert(table_name.to_string(), keys_and_attrs);

    let resp = client
        .batch_get_item()
        .set_request_items(Some(req_items))
        .send()
        .await
        .map_err(|e| AppError::DatabaseError(e.to_string()))?;

    // คืน Vec<Item>
    let items = resp
        .responses()
        .and_then(|m| m.get(table_name))
        .cloned()
        .unwrap_or_else(Vec::new);

    Ok(items)
}

pub fn extract_data_from_get_item(field: &str, output: &GetItemOutput) -> String {
    output
        .item()
        .and_then(|m| m.get(field))
        .and_then(|v| v.as_s().ok())
        .map(|s| s.to_owned())
        .unwrap_or_default()
}
pub fn extract_data_from_have_map(field: &str, output: &HashMap<String, AttributeValue>) -> String {
    output
        .get(field)
        .and_then(|v| v.as_s().ok())
        .map(|s| s.to_owned())
        .unwrap_or_default()
       
}

pub fn extract_appids(json: &Value) -> Vec<String> {
    json.get("request")
        .and_then(|req| req.get("app"))
        .and_then(|apps| apps.as_array())
        .map(|apps| {
            apps.iter()
                .filter_map(|app| app.get("appid").and_then(|id| id.as_str()).map(|id| id.to_string()))
                .collect()
        })
        .unwrap_or_default()
}
