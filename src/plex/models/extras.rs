use serde::Deserialize;

use super::metadata::Metadata;

#[derive(Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct ExtrasMediaContainer {
    pub total_size: Option<i32>,
    pub size: i32,
    #[serde(rename = "Hub")]
    pub hub: Vec<Hub>,
}

#[derive(Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct Hub {
    pub size: i32,
    pub title: String,
    #[serde(rename = "type")]
    pub rtype: String,
    pub context: Option<String>,
    #[serde(rename = "Metadata")]
    pub metadata: Option<Vec<Metadata>>,
}
