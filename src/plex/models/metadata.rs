use serde::Deserialize;

#[derive(Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct MetadataMediaContainer {
    pub total_size: Option<i32>,
    pub size: i32,
    #[serde(rename = "Metadata")]
    pub metadata: Option<Vec<Metadata>>,
}

#[derive(Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct Metadata {
    pub key: String,
    pub rating_key: String,
    pub title: String,
    #[serde(rename = "type")]
    pub rtype: String,
    pub original_title: Option<String>,
}
