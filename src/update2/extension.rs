use actix_web::{web, HttpResponse, Responder};
use aws_sdk_dynamodb::types::{AttributeDefinition, AttributeValue, KeySchemaElement, KeyType, ProvisionedThroughput, ScalarAttributeType};
use aws_sdk_dynamodb::error::{ProvideErrorMetadata, SdkError};
use crate::error::AppError;
use crate::worker::AppContext;
use crate::payload::MyRequest;

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
