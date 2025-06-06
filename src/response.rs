use serde::{Serialize, Deserialize};

#[derive(Serialize, Deserialize, Debug)]
pub struct ResponseRoot {
    pub response: Response,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Response {
    pub server: String,
    pub protocol: String,
    pub daystart: DayStart,
    pub app: Vec<AppResponse>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct DayStart {
    pub elapsed_seconds: u64,
    pub elapsed_days: u64,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct AppResponse {
    pub appid: String,
    pub cohort: String,
    pub status: String,
    pub cohortname: String,
    pub ping: AppPing,
    pub updatecheck: UpdateCheck,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct AppPing {
    pub status: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct UpdateCheck {
    pub status: String,
}