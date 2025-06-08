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
use std::sync::Arc;
use aws_sdk_dynamodb::Client;
use chrono::{TimeZone, Timelike, Utc};
use tokio::time::{interval, Duration};
use tokio::sync::RwLock;
use crate::update2::{CodeBase, Urls};
const CODEBASE_JSON: [&str; 6] = [
    "http://edgedl.me.gvt1.com/edgedl/release2/chrome_component/",
    "https://edgedl.me.gvt1.com/edgedl/release2/chrome_component/",
    "http://dl.google.com/release2/chrome_component/",
    "https://dl.google.com/release2/chrome_component/",
    "http://www.google.com/dl/release2/chrome_component/",
    "https://www.google.com/dl/release2/chrome_component/"
];
const  TABLE_NAME: &str = "Extensions";
pub async fn importer_data_from_json(
    ctx: web::Data<AppContext>,
) -> Result<impl Responder, AppError> {
    let client = &ctx.dynamodb_client;
    let _ = is_not_exits_create_table(&client.clone()).await;
    let _ = insert_extensions(&client).await;
    Ok(HttpResponse::Ok().json("success"))
}
pub async fn is_not_exits_create_table(client: &Client) -> Result<(), AppError> {
    let table_name = TABLE_NAME;
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
    let table_name = TABLE_NAME;
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
    ids: Vec<String>
) -> Result<Vec<Extension>, AppError> {
    let table_name = TABLE_NAME;
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

    let items = resp
        .responses()
        .and_then(|m| m.get(table_name))
        .cloned()
        .unwrap_or_default();
    
    let extensions = items.into_iter().map(|v| v.into()).collect::<Vec<Extension>>();
    Ok(extensions)
}

pub async fn scan_all_extensions(
    client: &Client,
) -> Result<Vec<Extension>, AppError> {
    let table_name = TABLE_NAME;
    let mut extensions = Vec::new();
    let mut last_evaluated_key: Option<std::collections::HashMap<String, AttributeValue>> = None;

    loop {
        let mut scan_builder = client
            .scan()
            .table_name(table_name.to_owned());
        if let Some(ref lek) = last_evaluated_key {
            scan_builder = scan_builder.set_exclusive_start_key(Some(lek.clone()));
        }

        let resp = scan_builder
            .send()
            .await
            .map_err(|e| AppError::DatabaseError(e.to_string()))?;

        if let Some(items) = resp.items {
            for item in items {
                extensions.push(item.into());
            }
        }
        // เช็คว่ามีหน้าถัดไปหรือไม่ (DynamoDB จะคืน LastEvaluatedKey ถ้ายังสแกนไม่ครบ)
        last_evaluated_key = resp.last_evaluated_key;
        if last_evaluated_key.is_none() {
            break;
        }
    }
    Ok(extensions)
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
pub fn init_from_dynamodb(
    items: Vec<Extension>,
) -> Arc<RwLock<HashMap<String, Extension>>> {
    let mut db_map = HashMap::new();
    for item in items {
        let ext: Extension = item.clone().into();
        db_map.insert(ext.id.clone(), ext);
    }
    Arc::new(RwLock::new(db_map))
}

pub fn spawn_periodic_refresh(
    app_context: Arc<AppContext>,
) {
    tokio::spawn(async move {
        let mut interval = interval(Duration::from_secs(1200)); // 10 นาที = 600 วินาที
        loop {
            interval.tick().await;

            match scan_all_extensions(&app_context.dynamodb_client).await {
                Ok(fresh_extensions) => {
                    let mut new_map = HashMap::new();
                    for ext in fresh_extensions {
                        new_map.insert(ext.id.clone(), ext); // เปลี่ยนเป็น key ที่ต้องการ
                    }
                    let mut map_writer = app_context.map.write().await;
                    *map_writer = new_map;
                    log::info!("Map refreshed successfully!");
                }
                Err(e) => {
                    log::error!("Failed to refresh map: {e}");
                }
            }
        }
    });
}

pub fn gen_codebase_urls(path_id: &str, version: &str) -> Urls {
    Urls { url: CODEBASE_JSON.iter().map(|c| CodeBase { codebase: format!("{}{}_{}", c, path_id, version) }).collect() }
}
pub fn get_daystart() -> (u64, u32) {
    let start_date = Utc.ymd(2007, 1, 1).and_hms(0, 0, 0);
    let now = Utc::now();

    let elapsed_days = (now.date_naive() - start_date.date_naive()).num_days();
    let elapsed_seconds = now.num_seconds_from_midnight();

    (elapsed_days as u64, elapsed_seconds)
}