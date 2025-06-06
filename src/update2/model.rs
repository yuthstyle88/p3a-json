use serde::{Deserialize, Serialize};
use serde_json::Value;

#[derive(Debug, Serialize, Deserialize)]
pub struct ChromeApp {
    pub appid: String,
    pub cohort: String,
    pub status: String,
    pub cohortname: String,
    pub ping_status: String,
    pub updatecheck_status: String,
    pub urls: Vec<CodebaseUrl>,
    pub manifest_version: String,
    pub packages: Vec<PackageInfo>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CodebaseUrl {
    pub codebase: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct PackageInfo {
    pub hash_sha256: String,
    pub size: i64,
    pub name: String,
    pub fp: String,
    pub required: bool,
    pub hash: String,
}
impl ChromeApp {
    pub fn from_value(value: &Value) -> Option<Self> {
        Some(Self {
            appid: value.get("appid")?.as_str()?.to_string(),
            cohort: value.get("cohort")?.as_str()?.to_string(),
            status: value.get("status")?.as_str()?.to_string(),
            cohortname: value.get("cohortname")?.as_str()?.to_string(),
            ping_status: value.get("ping")?.get("status")?.as_str()?.to_string(),
            updatecheck_status: value.get("updatecheck")?.get("status")?.as_str()?.to_string(),
            urls: value.get("updatecheck")?
                .get("urls")?
                .get("url")?
                .as_array()?
                .iter()
                .filter_map(|url| {
                    url.get("codebase").and_then(|s| s.as_str()).map(|c| CodebaseUrl { codebase: c.to_string() })
                })
                .collect(),
            manifest_version: value.get("updatecheck")?
                .get("manifest")?
                .get("version")?
                .as_str()?.to_string(),
            packages: value.get("updatecheck")?
                .get("manifest")?
                .get("packages")?
                .get("package")?
                .as_array()?
                .iter()
                .filter_map(|pkg| {
                    Some(PackageInfo {
                        hash_sha256: pkg.get("hash_sha256")?.as_str()?.to_string(),
                        size: pkg.get("size")?.as_i64()?,
                        name: pkg.get("name")?.as_str()?.to_string(),
                        fp: pkg.get("fp")?.as_str()?.to_string(),
                        required: pkg.get("required")?.as_bool()?,
                        hash: pkg.get("hash")?.as_str()?.to_string(),
                    })
                })
                .collect(),
        })
    }
}
