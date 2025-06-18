use serde::Deserialize;

#[derive(Deserialize, Clone)]
pub struct MeasureRequest {
    pub data: Vec<u8>,
    pub epoch: u8,
}