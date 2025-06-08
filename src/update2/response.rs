use serde::{Serialize};
use crate::update2::model::App;

#[derive(Serialize, Debug)]
pub struct ResponseRoot {
    pub response: Response,
}

#[derive(Serialize, Debug)]
pub struct Response {
    pub server: String,
    pub protocol: String,
    pub daystart: DayStart,
    pub app: Vec<App>,
}

#[derive(Serialize, Debug)]
pub struct DayStart {
    pub elapsed_seconds: u64,
    pub elapsed_days: u64,
}
