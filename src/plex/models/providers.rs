use serde::Deserialize;

#[derive(Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct ProviderMediaContainer {
    pub friendly_name: String,
    pub machine_identifier: String,
    #[serde(rename = "MediaProvider")]
    pub media_provider: Vec<MediaProvider>,
}

#[derive(Deserialize, Debug, Clone)]
pub struct MediaProvider {
    pub title: String,
    pub identifier: String,
    #[serde(rename = "Feature")]
    pub features: Vec<Feature>,
}

#[derive(Deserialize, Debug, Clone)]
pub struct Feature {
    pub key: Option<String>,
    #[serde(rename = "type")]
    pub rtype: String,
    #[serde(rename = "Directory")]
    pub directories: Option<Vec<Directory>>,
}

#[derive(Deserialize, Debug, Clone)]
pub struct Directory {
    pub agent: Option<String>,
    pub title: Option<String>,
    pub id: Option<String>,
}
