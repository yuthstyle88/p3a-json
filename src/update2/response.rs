use std::collections::HashMap;
use aws_sdk_dynamodb::operation::batch_get_item::BatchGetItemOutput;
use aws_sdk_dynamodb::types::AttributeValue;
use serde::{Serialize};
use crate::update2::model::App;

#[derive(Serialize, Debug, Default)]
pub struct ResponseRoot {
    pub response: Response,
}

#[derive(Serialize, Debug,Default)]
pub struct Response {
    pub server: String,
    pub protocol: String,
    pub daystart: DayStart,
    pub app: Vec<App>,
}

#[derive(Serialize, Debug ,Default)]
pub struct DayStart {
    pub elapsed_seconds: u64,
    pub elapsed_days: u64,
}

impl ResponseRoot {
    pub fn to_json(data: &Vec<HashMap<String, AttributeValue>>) -> Vec<App> {
         Vec::new()
    }
}