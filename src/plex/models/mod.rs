use serde::Deserialize;

pub mod extras;
pub mod metadata;
pub mod providers;
#[derive(Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub(crate) struct MediaContainerWrapper<T> {
    #[serde(rename = "MediaContainer")]
    pub(crate) media_container: T,
}
