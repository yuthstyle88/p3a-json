use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct MyPayload {
    pub cadence: String,
    pub channel: String,
    pub country_code: String,
    pub metric_name: String,
    pub metric_value: i32,
    pub platform: String,
    pub version: String,
    pub woi: i16,
    pub wos: i16,
    pub yoi: i16,
    pub yos: i16,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct MyExtensionPayload {
    pub app_id: String,
    pub timestamp: chrono::NaiveDateTime,
    pub version: String,
    pub cohort: String,
    pub cohort_name: String,
    pub status: String,
    pub package_name: String,
    pub package_size: i64,
    pub hash_sha256: String,
    pub fp: String,
    pub hash_base64: String,
    pub download_urls: Vec<String>,
}
