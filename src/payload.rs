use lapin::protocol::basic::Publish;
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
pub struct MyRequest {
   pub request: Request,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Request {
    #[serde(rename = "@os")]
    pub at_os: String,
    #[serde(rename = "@updater")]
    pub updater: String,
    pub acceptformat: String,
    pub app: Vec<App>,
    pub arch: String,
    pub dedup: String,
    pub hw: Hw,
    pub ismachine: bool,
    pub nacl_arch: String,
    pub os: OsInfo,
    pub prodchannel: String,
    pub prodversion: String,
    pub protocol: String,
    pub requestid: String,
    pub sessionid: String,
    pub updaterchannel: String,
    pub updaterversion: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct App {
    pub appid: String,
    pub cohort: String,
    pub cohortname: String,
    pub enabled: bool,
    pub installdate: u64,
    pub packages: Packages,
    pub ping: Ping,
    pub updatecheck: Option<UpdateCheck>,
    pub version: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Packages {
    #[serde(rename = "package")]
    pub package_list: Vec<Package>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Package {
    pub fp: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Ping {
    pub ping_freshness: String,
    pub rd: u64,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct UpdateCheck {}

#[derive(Serialize, Deserialize, Debug)]
pub struct Hw {
    pub avx: bool,
    pub physmemory: u64,
    pub sse: bool,
    pub sse2: bool,
    pub sse3: bool,
    pub sse41: bool,
    pub sse42: bool,
    pub ssse3: bool,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct OsInfo {
    pub arch: String,
    pub platform: String,
    pub version: String,
}
