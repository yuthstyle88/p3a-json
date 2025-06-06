use std::fs::File;
use std::io::Read;
use actix_web::{web, HttpResponse, Responder};
use aws_sdk_dynamodb::types::{AttributeDefinition, AttributeValue, KeySchemaElement, KeyType, ProvisionedThroughput, ScalarAttributeType};
use aws_sdk_dynamodb::error::{ProvideErrorMetadata, SdkError};
use serde_json::Value;
use crate::error::AppError;
use crate::worker::AppContext;
use crate::payload::MyRequest;
use crate::update2::model::ChromeApp;

pub async fn update2_json(
    ctx: web::Data<AppContext>,
    item: web::Json<MyRequest>,
) -> Result<impl Responder, AppError> {
    let client = &ctx.dynamodb_client;
    let _payload = item.into_inner();

    // ตัวอย่าง: อัปเดตฟิลด์หนึ่งใน DynamoDB
    let table_name = "Extensions";
    let id = "ID".to_string();

    let _ = is_not_exits_create_table(&client.clone()).await;
    let result = client
        .get_item()
        .table_name(table_name)
        .key("ID", AttributeValue::S(id.clone()))
        .send()
        .await
        .map_err(|e| AppError::DatabaseError(e.to_string()))?;

    Ok(HttpResponse::Ok().json("app data"))
}
pub async fn importer_data_from_json(
    ctx: web::Data<AppContext>,
) -> Result<impl Responder, AppError> {
    let client = &ctx.dynamodb_client;
    let _ = insert_chrome_app(&client).await;
    Ok(HttpResponse::Ok().json("success"))
}

pub async fn is_not_exits_create_table(client: &aws_sdk_dynamodb::Client) -> Result<(), AppError> {
    let table_name = "Extensions";
    let table_exists = match client
        .describe_table()
        .table_name(table_name)
        .send()
        .await
    {
        Ok(_) => true,
        Err(err) => match &err {
            SdkError::ServiceError { .. } if err.code() == Some("ResourceNotFoundException") => false,
            _ => return Err(AppError::DatabaseError("DatabaseError".to_string())),
        }
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

pub async fn insert_chrome_app(client: &aws_sdk_dynamodb::Client) -> Result<(), AppError>{

    let mut file = File::open("./extensions.json")?;
    let mut data = String::new();
    file.read_to_string(&mut data)?;

    // แปลง String เป็น Value สำหรับ parse
    let records: Vec<Value> = serde_json::from_str(&data).map_err(|e| AppError::BadRequest(e.to_string()))?;

    for record in &records {
        let mut item = std::collections::HashMap::new();
        if let Some(response) = record.get("response").and_then(|resp| resp.get("app"))
        {
            let app: ChromeApp = ChromeApp::from_value(&response[0]).unwrap();
            let table_name = "Extensions";
            item.insert("ID".to_string(), AttributeValue::S(app.appid.clone()));
            item.insert("COHORT".to_string(), AttributeValue::S(app.cohort.clone()));
            item.insert("STATUS".to_string(), AttributeValue::S(app.status.clone()));
            item.insert("COHORTNAME".to_string(), AttributeValue::S(app.cohortname.clone()));
            item.insert("PING_STATUS".to_string(), AttributeValue::S(app.ping_status.clone()));
            item.insert("UPDATECHECK_STATUS".to_string(), AttributeValue::S(app.updatecheck_status.clone()));
            item.insert("MANIFEST_VERSION".to_string(), AttributeValue::S(app.manifest_version.clone()));

            // แปลง Vec<CodebaseUrl> เป็น AttributeValue::L
            let urls_value = AttributeValue::L(
                app.urls.iter()
                    .map(|url| {
                        let mut m = std::collections::HashMap::new();
                        m.insert("CODEBASE".to_string(), AttributeValue::S(url.codebase.clone()));
                        AttributeValue::M(m)
                    })
                    .collect()
            );
            item.insert("URLS".to_string(), urls_value);

            // แปลง Vec<PackageInfo> เป็น AttributeValue::L
            let packages_value = AttributeValue::L(
                app.packages.iter()
                    .map(|pkg| {
                        let mut m = std::collections::HashMap::new();
                        m.insert("HASH_SHA256".to_string(), AttributeValue::S(pkg.hash_sha256.clone()));
                        m.insert("SIZE".to_string(), AttributeValue::N(pkg.size.to_string()));
                        m.insert("NAME".to_string(), AttributeValue::S(pkg.name.clone()));
                        m.insert("FP".to_string(), AttributeValue::S(pkg.fp.clone()));
                        m.insert("REQUIRED".to_string(), AttributeValue::Bool(pkg.required));
                        m.insert("HASH".to_string(), AttributeValue::S(pkg.hash.clone()));
                        AttributeValue::M(m)
                    })
                    .collect()
            );
            item.insert("PACKAGES".to_string(), packages_value);

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
