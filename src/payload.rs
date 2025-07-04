use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MyPayload {
    pub cadence: String,
    pub channel: String,
    pub country_code: String,
    pub metric_name: String,
    pub metric_value: i32,
    pub platform: String,
    pub version: String,
    pub woi: i16,
    pub wos: Option<i16>,
    pub yoi: i16,
    pub yos: i16,
}
