use std::collections::HashMap;
use std::io::Read;
use std::sync::Arc;
use aws_sdk_dynamodb::operation::get_item::GetItemOutput;
use aws_sdk_dynamodb::types::AttributeValue;
use chrono::DateTime;
use tokio::sync::RwLock;
use crate::update2::extract_data_from_have_map;
use crate::update2::model::{App, Extension};

impl Extension {
    pub async fn filter_for_updates(
        extensions: &Vec<Extension>,
        all_extensions_map: &Arc<RwLock<HashMap<String, Extension>>>,
    ) -> Vec<Extension> {
        // Initialize a Vec to collect filtered extensions
        let mut filtered_extensions = Vec::new();
        let read_guard = all_extensions_map.read().await;
        for ext_being_checked in extensions {
            let being_checked: Extension = ext_being_checked.clone().into();    // <== implement from_value()
            if let Some(found_extension) = read_guard.get(&being_checked.id) {
                let status = compare_versions(&being_checked.version, &found_extension.version);
                let found = found_extension.blacklisted.parse::<bool>().unwrap();
                if !found && status <= 0 {
                    let mut ext = found_extension.clone();
                    if status == 0 {
                        ext.status = "noupdate".to_owned();
                    }
                    ext.fp = being_checked.fp.clone();
                    filtered_extensions.push(ext);
                }
            }
        }
        // Return the collected filtered extensions; update return type to Vec<Extension>
        filtered_extensions
    }
}

impl From<HashMap<String, AttributeValue>> for Extension {
    fn from(item: HashMap<String, AttributeValue>) -> Self {
        Self{
            id: extract_data_from_have_map("ID", &item),
            cohort: extract_data_from_have_map("COHORT", &item),
            cohortname: extract_data_from_have_map("COHORTNAME", &item),
            name: extract_data_from_have_map("NAME", &item),
            version: extract_data_from_have_map("VERSION", &item),
            hash_sha256: extract_data_from_have_map("HASH_SHA256", &item),
            status: extract_data_from_have_map("STATUS", &item),
            fp: extract_data_from_have_map("FP", &item),
            blacklisted: extract_data_from_have_map("BLACKLISTED", &item),
            required: extract_data_from_have_map("REQUIRED", &item),
            hash: extract_data_from_have_map("HASH", &item),
            size: extract_data_from_have_map("SIZE", &item),
            created_at: DateTime::parse_from_rfc3339(&extract_data_from_have_map("CREATE_AT", &item)).unwrap_or_default().with_timezone(&chrono::Utc),
            update_at: DateTime::parse_from_rfc3339(&extract_data_from_have_map("UPDATE_AT", &item)).unwrap_or_default().with_timezone(&chrono::Utc),
        }
    }
}
pub fn compare_versions(version1: &str, version2: &str) -> i32 {
    let version1_parts: Vec<u32> = version1
        .split('.')
        .map(|s| s.parse::<u32>().unwrap_or(0))
        .collect();
    let version2_parts: Vec<u32> = version2
        .split('.')
        .map(|s| s.parse::<u32>().unwrap_or(0))
        .collect();

    let len = version1_parts.len().min(version2_parts.len());
    for i in 0..len {
        if version1_parts[i] < version2_parts[i] {
            return -1;
        }
        if version1_parts[i] > version2_parts[i] {
            return 1;
        }
    }
    0
}