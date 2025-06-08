use std::collections::HashMap;
use std::io::Read;
use std::sync::Arc;
use aws_sdk_dynamodb::operation::get_item::GetItemOutput;
use chrono::DateTime;
use tokio::sync::RwLock;
use crate::update2::model::{App, Extension};
use crate::update2::utils::extract_data;

impl Extension {
    pub async fn filter_for_updates(
        extensions: Vec<&GetItemOutput>,
        all_extensions_map: &Arc<RwLock<HashMap<String, Extension>>>,
    ) -> Vec<Extension> {
        // Initialize a Vec to collect filtered extensions
        let mut filtered_extensions = Vec::new();
        let read_guard = all_extensions_map.read().await;
        for ext_being_checked in extensions {
            let being_checked: Extension = ext_being_checked.into();    // <== implement from_value()
            if let Some(found_extension) = read_guard.get(&being_checked.id) {
                let status = compare_versions(&being_checked.version, &found_extension.version);
                if !found_extension.blacklisted && status <= 0 {
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
impl From<&GetItemOutput> for Extension {
    fn from(item: &GetItemOutput) -> Self {
        Self{
            id: extract_data("ID", item),
            cohort: extract_data("COHORT", item),
            cohortname: extract_data("COHORTNAME", item),
            package_name: extract_data("PACKAGE_NAME", item),
            version: extract_data("VERSION", item),
            hash_sha256: extract_data("HASH_SHA256", item),
            status: extract_data("STATUS", item),
            fp: extract_data("FP", item),
            blacklisted: extract_data("BLACKLISTED", item).parse::<bool>().unwrap_or_default(),
            required: extract_data("REQUIRED", item).parse::<bool>().unwrap_or_default(),
            hash: extract_data("HASH", item),
            size: extract_data("SIZE", item).parse::<u64>().unwrap_or_default(),
            created_at: DateTime::parse_from_rfc3339(&extract_data("CREATE_AT", item)).unwrap_or_default().with_timezone(&chrono::Utc),
            update_at: DateTime::parse_from_rfc3339(&extract_data("UPDATE_AT", item)).unwrap_or_default().with_timezone(&chrono::Utc),
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