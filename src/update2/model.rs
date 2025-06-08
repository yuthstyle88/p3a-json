use chrono::DateTime;
use serde::{Deserialize, Serialize};
use serde_json::Value;

#[derive(Debug, Serialize, Deserialize, Default)]
pub struct App {
    pub appid: String,
    pub cohort: String,
    pub status: String,
    pub cohortname: String,
    pub ping: Status,
    pub updatecheck: UpdateCheck,
    pub manifest: Manifest,
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
    pub size: u64,
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
    pub package_name: String,
    pub version: String,
    pub hash_sha256: String,
    pub status: String,
    pub fp: String,
    pub blacklisted: bool,
    pub required: bool,
    pub hash: String,
    pub size: u64,
    pub created_at: DateTime<chrono::Utc>,
    pub update_at: DateTime<chrono::Utc>,
    
}
impl From<App> for Extension {
    fn from(value: App) -> Self {
        Self {
            id: value.appid,
            cohort: value.cohort,
            status: value.status,
            cohortname: value.cohortname,
            package_name: value.manifest.packages.package[0].name.clone(),
            version: value.manifest.version,
            hash_sha256: value.manifest.packages.package[0].hash_sha256.clone(),
            fp: value.manifest.packages.package[0].fp.clone(),
            blacklisted: false,
            required: value.manifest.packages.package[0].required,
            hash: value.manifest.packages.package[0].hash.clone(),
            size: value.manifest.packages.package[0].size,
            created_at: Default::default(),
            update_at: Default::default(),
        }
    }
}
impl App {
    pub fn from_value(value: &Value) -> Option<Self> {
        let ping_status = value.get("ping")?.get("status")?.as_str()?.to_string();
        let updatecheck_value = value.get("updatecheck")?;
        let manifest_value = updatecheck_value.get("manifest")?;
        let packages: Vec<Package> = manifest_value
            .get("packages")?
            .get("package")?
            .as_array()?
            .iter()
            .filter_map(|pkg| {
                Some(Package {
                    hash_sha256: pkg.get("hash_sha256")?.as_str()?.to_string(),
                    size: pkg.get("size")?.as_u64()?,
                    name: pkg.get("name")?.as_str()?.to_string(),
                    fp: pkg.get("fp")?.as_str()?.to_string(),
                    required: pkg.get("required")?.as_bool()?,
                    hash: pkg.get("hash")?.as_str()?.to_string(),
                })
            })
            .collect();

        let url_vec: Vec<CodeBase> = updatecheck_value
            .get("urls")?
            .get("url")?
            .as_array()?
            .iter()
            .filter_map(|url| {
                url.get("codebase")
                    .and_then(|s| s.as_str())
                    .map(|c| CodeBase {
                        codebase: c.to_string(),
                    })
            })
            .collect();

        let urls = Urls { url: url_vec };

        let manifest = Manifest {
            version: manifest_value.get("version")
                .and_then(|v| v.as_str())
                .unwrap_or("0.0.0")
                .to_string(),
            packages: Packages { package: packages },
        };

        let updatecheck = UpdateCheck {
            status: updatecheck_value.get("status")?.as_str()?.to_string(),
            urls,
        };

        Some(Self {
            appid: value.get("appid")?.as_str()?.to_string(),
            cohort: value.get("cohort")?.as_str()?.to_string(),
            status: value.get("status")?.as_str()?.to_string(),
            cohortname: value.get("cohortname")?.as_str()?.to_string(),
            ping: Status {
                status: ping_status,
            },
            updatecheck,
            manifest,
        })
    }
}
