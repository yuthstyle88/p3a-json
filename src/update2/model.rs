use chrono::DateTime;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use crate::payload::Ping;

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
    pub size: String,
    pub name: String,
    pub fp: String,
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

#[derive(Clone)]
pub struct Extension {
    pub id: String,
    pub cohort: String,
    pub cohortname: String,
    pub name: String,
    pub version: String,
    pub hash_sha256: String,
    pub status: String,
    pub fp: String,
    pub blacklisted: bool,
    pub required: bool,
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
        let required= package.get("required")?.as_bool()?;
        let hash= package.get("hash")?.as_str()?.to_string();
        let size= package.get("size")?.as_number()?.to_string();
        println!("{:?}", package);
        Some(Self {
            id: value.get("appid")?.as_str()?.to_string(),
            cohort: value.get("cohort")?.as_str()?.to_string(),
            status: value.get("status")?.as_str()?.to_string(),
            cohortname: value.get("cohortname")?.as_str()?.to_string(),
            version,
            name,
            hash_sha256,
            fp,
            blacklisted: false,
            required,
            hash,
            size,
            created_at: Default::default(),
            update_at: Default::default(),
        })
       
    }
}
