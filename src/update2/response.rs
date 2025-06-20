use crate::payload::Ping;
use crate::update2::model::App;
use crate::update2::{Extension, Manifest, UpdateCheck, Package, Packages};

use serde::Serialize;
use crate::update2::utils::{gen_codebase_urls, get_daystart};

#[derive(Serialize, Debug, Default)]
pub struct ResponseRoot {
    pub response: Response,
}

#[derive(Serialize, Debug, Default)]
pub struct Response {
    pub server: String,
    pub protocol: String,
    pub daystart: DayStart,
    pub app: Vec<App>,
}

#[derive(Serialize, Debug, Default)]
pub struct DayStart {
    pub elapsed_seconds: u64,
    pub elapsed_days: u64,
}

fn get_update_status(status: &str) -> String {
    if status == "noupdate" {
        "noupdate".to_string()
    } else {
        "ok".to_string()
    }
}


pub fn gen_manifest(ext: &Extension) -> Manifest {
    let package = Package {
        hash_sha256: ext.hash_sha256.clone(),
        size: ext.size.parse().unwrap(),
        name: ext.name.clone(),
        fp: ext.fp.clone(),
        required: ext.required.parse().unwrap(),
        hash: ext.hash.clone(),
    };

    Manifest {
        version: ext.version.clone(),
        packages: Packages {
            package: vec![package]
        },
    }
}
impl ResponseRoot {
    pub fn to_json(data: &Vec<Extension>, protocol: &str) -> ResponseRoot {
        let apps = data
            .iter()
            .map(|ext| {
                if ext.status == "noupdate" {
                    App {
                        appid: ext.id.clone(),
                        cohort: None,
                        status: get_update_status(&ext.status),
                        cohortname: None,
                        ping: None,
                        updatecheck: None,
                        manifest: None,
                    }
                } else {
                    let urls = gen_codebase_urls(&ext.id, &ext.version);
                    let updatecheck = UpdateCheck {
                        status: get_update_status(&ext.status.to_string()),
                        urls,
                    };
                    let ping = Ping {
                        status: Some("ok".to_string()),
                        ping_freshness: None,
                        rd: None,
                        r: None,
                    };
                    App {
                        appid: ext.id.clone(),
                        cohort: Some(ext.cohort.clone()),
                        status: get_update_status(&ext.status),
                        cohortname: Some(ext.cohortname.clone()),
                        ping: Some(ping),
                        updatecheck: Some(updatecheck),
                        manifest: Option::from(gen_manifest(&ext)),
                    }
                }
            })
            .collect();
        let (days, seconds) = get_daystart();
        let daystart = DayStart {
            elapsed_seconds: seconds as u64,
            elapsed_days: days,
        };
        ResponseRoot {
            response: Response {
                server: "prod".to_string(),
                protocol: protocol.to_string(),
                daystart,
                app: apps,
            },
        }
    }
}
