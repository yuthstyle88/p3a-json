use std::collections::HashMap;
use aws_sdk_dynamodb::operation::batch_get_item::BatchGetItemOutput;
use aws_sdk_dynamodb::types::AttributeValue;
use serde::{Serialize};
use crate::update2::Extension;
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
    pub fn to_json(data: &Vec<Extension>) -> ResponseRoot {
        let apps = data.iter().map(|ext| App {
            appid: ext.id.clone(),
            cohort: ext.cohort.clone(),
            status: ext.status.clone(),
            cohortname: ext.cohortname.clone(),
            ping: Default::default(),
            updatecheck: Default::default(),
            manifest: Default::default(),
        }).collect();

        ResponseRoot {
            response: Response {
                server: "example_server".to_string(),
                protocol: "1.0".to_string(),
                daystart: DayStart::default(),
                app: apps,
            }
        }
    }
}