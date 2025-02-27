use serde::{Deserialize, Serialize};

#[derive(Serialize, Debug, Deserialize)]
pub struct Tag {
    name: String,
    value: String,
}

#[derive(Serialize, Debug)]
pub struct DataItem {
    pub signature_type: String,
    pub owner: String,
    pub tags: Vec<Tag>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub bundled_in: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub block_height: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub timestamp: Option<i64>,
    pub tx_pos: usize,
    pub _id: String,
}
