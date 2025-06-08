use chrono::DateTime;
use serde::{Deserialize, Serialize, Deserializer};
use serde_json::Value;
use crate::payload::Ping;

pub fn serial_to_bool(val: &str) -> bool {
    match val.trim().to_ascii_lowercase().as_str() {
        "true" | "1" | "yes" => true,
        "false" | "0" | "no" => false,
        _ => false, // ถ้าไม่ตรงอะไรเลย กำหนดเป็น false
    }
}
pub fn serial_to_int(val: &str) -> i64 {
    val.trim().parse().unwrap_or(0)
}
fn bool_from_string<'de, D>(deserializer: D) -> Result<bool, D::Error>
where
    D: Deserializer<'de>,
{
    let s = String::deserialize(deserializer)?;
    Ok(serial_to_bool(&s))
}

fn str_to_i64<'de, D>(deserializer: D) -> Result<i64, D::Error>
where
    D: Deserializer<'de>,
{
    let s = String::deserialize(deserializer)?;
    Ok(serial_to_int(&s))
}
#[derive(Debug, Serialize, Deserialize, Default)]
pub struct App {
    pub appid: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cohort: Option<String>,
    pub status: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cohortname: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ping: Option<Ping>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub updatecheck: Option<UpdateCheck>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub manifest: Option<Manifest>,
}

#[derive(Debug, Serialize, Deserialize, Default)]
pub struct Status {
    pub status: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct Manifest {
    pub version: String,
    pub packages: Packages,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct Packages {
    pub package: Vec<Package>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Package {
    pub hash_sha256: String,
    #[serde(deserialize_with = "str_to_i64")]
    pub size: i64,
    pub name: String,
    pub fp: String,
    #[serde(deserialize_with = "bool_from_string")]
    pub required: bool,
    pub hash: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct UpdateCheck {
    pub status: String,
    #[serde(rename = "urls")]
    pub urls: Urls,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct Urls {
    pub url: Vec<CodeBase>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CodeBase {
    pub codebase: String,
}

#[derive(Clone,Debug)]
pub struct Extension {
    pub id: String,
    pub cohort: String,
    pub cohortname: String,
    pub name: String,
    pub version: String,
    pub hash_sha256: String,
    pub status: String,
    pub fp: String,
    pub blacklisted: String,
    pub required: String,
    pub hash: String,
    pub size: String,
    pub created_at: DateTime<chrono::Utc>,
    pub update_at: DateTime<chrono::Utc>,
    
}
impl Extension {

    pub fn from_value(value: &Value) -> Option<Self> {
       
        let manifest_value = value.get("updatecheck")?.get("manifest")?;
        
        let version = manifest_value.get("version")
            .and_then(|v| v.as_str())
            .unwrap_or("0.0.0")
            .to_string();
        let package = manifest_value.get("packages")?.get("package")?.get(0)?;
        let name = package.get("name")?.as_str()?.to_string();
        let hash_sha256= package.get("hash_sha256")?.as_str()?.to_string();
        let fp = package.get("fp")?.as_str()?.to_string();
        let required= package.get("required")?.as_bool()?.to_string();
        let hash= package.get("hash")?.as_str()?.to_string();
        let size= package.get("size")?.as_number()?.to_string();
        Some(Self {
            id: value.get("appid")?.as_str()?.to_string(),
            cohort: value.get("cohort")?.as_str()?.to_string(),
            status: value.get("status")?.as_str()?.to_string(),
            cohortname: value.get("cohortname")?.as_str()?.to_string(),
            version,
            name,
            hash_sha256,
            fp,
            blacklisted: "false".to_string(),
            required,
            hash,
            size,
            created_at: Default::default(),
            update_at: Default::default(),
        })
       
    }
}
