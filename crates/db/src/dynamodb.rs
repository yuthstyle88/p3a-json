use aws_config::BehaviorVersion;
use aws_sdk_dynamodb::Client as DynamoDBClient;
use serde::{Deserialize, Serialize};
use std::error::Error;

pub mod dynamodb {
    use super::*;
    use aws_sdk_dynamodb::types::AttributeValue;

    #[derive(Serialize, Deserialize)]
    pub struct Extension {
        pub extension_id: String,
        pub version: String,
        pub crx_url: String,
    }

    pub async fn init_client(endpoint_url: &str) -> Result<DynamoDBClient, Box<dyn Error>> {
        let config = aws_config::defaults(BehaviorVersion::latest())
            .endpoint_url(endpoint_url)
            .test_credentials()
            .load()
            .await;
        let dynamodb_config = aws_sdk_dynamodb::config::Builder::from(&config).build();
        Ok(DynamoDBClient::from_conf(dynamodb_config))
    }

    pub async fn create_extensions_table(
        client: &DynamoDBClient,
    ) -> Result<(), aws_sdk_dynamodb::Error> {
        let table_exists = client
            .list_tables()
            .send()
            .await?
            .table_names
            .unwrap_or_default()
            .contains(&"Extensions".to_string());

        if !table_exists {
            client
                .create_table()
                .table_name("Extensions")
                .attribute_definitions(
                    aws_sdk_dynamodb::types::AttributeDefinition::builder()
                        .attribute_name("extension_id")
                        .attribute_type(aws_sdk_dynamodb::types::ScalarAttributeType::S)
                        .build()?,
                )
                .key_schema(
                    aws_sdk_dynamodb::types::KeySchemaElement::builder()
                        .attribute_name("extension_id")
                        .key_type(aws_sdk_dynamodb::types::KeyType::Hash)
                        .build()?,
                )
                .billing_mode(aws_sdk_dynamodb::types::BillingMode::PayPerRequest)
                .send()
                .await?;
            println!("Created Extensions table");
        }
        Ok(())
    }

    pub async fn add_extension(
        client: &DynamoDBClient,
        extension: Extension,
    ) -> Result<(), aws_sdk_dynamodb::Error> {
        client
            .put_item()
            .table_name("Extensions")
            .item(
                "extension_id",
                AttributeValue::S(extension.extension_id.clone()),
            )
            .item("version", AttributeValue::S(extension.version.clone()))
            .item("crx_url", AttributeValue::S(extension.crx_url.clone()))
            .send()
            .await?;
        println!(
            "Added extension: {} (version: {})",
            extension.extension_id, extension.version
        );
        Ok(())
    }

    pub async fn get_extensions(
        client: &DynamoDBClient,
    ) -> Result<Vec<Extension>, aws_sdk_dynamodb::Error> {
        let resp = client.scan().table_name("Extensions").send().await?;
        let mut extensions = Vec::new();
        if let Some(items) = resp.items {
            for item in items {
                let extension = Extension {
                    extension_id: item
                        .get("extension_id")
                        .and_then(|v| v.as_s().ok().cloned())
                        .unwrap_or_default(),
                    version: item
                        .get("version")
                        .and_then(|v| v.as_s().ok().cloned())
                        .unwrap_or_default(),
                    crx_url: item
                        .get("crx_url")
                        .and_then(|v| v.as_s().ok().cloned())
                        .unwrap_or_default(),
                };
                if extension.crx_url.ends_with(".crx3") {
                    extensions.push(extension);
                }
            }
        }
        Ok(extensions)
    }
}
