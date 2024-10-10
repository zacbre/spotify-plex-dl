use reqwest::Response;

use super::models::{
    extras::ExtrasMediaContainer, metadata::MetadataMediaContainer,
    providers::ProviderMediaContainer, MediaContainerWrapper,
};

pub struct Plex {
    base_url: String,
    token: String,
}

impl Plex {
    pub fn new(base_url: String, token: String) -> Self {
        Self { base_url, token }
    }

    pub async fn get(&self, url: &str) -> Result<Response, anyhow::Error> {
        let client = reqwest::Client::new();
        let response = client
            .get(&format!("{}/{}", self.base_url, url))
            .header("X-Plex-Token", &self.token)
            .header("Accept", "application/json")
            .send()
            .await?;

        Ok(response)
    }

    pub async fn post(&self, url: &str) -> Result<Response, anyhow::Error> {
        let client = reqwest::Client::new();
        let response = client
            .post(&format!("{}/{}", self.base_url, url))
            .header("X-Plex-Token", &self.token)
            .header("Accept", "application/json")
            .send()
            .await?;

        Ok(response)
    }

    pub async fn put(&self, url: &str) -> Result<Response, anyhow::Error> {
        let client = reqwest::Client::new();
        let response = client
            .put(&format!("{}/{}", self.base_url, url))
            .header("X-Plex-Token", &self.token)
            .header("Accept", "application/json")
            .send()
            .await?;

        Ok(response)
    }

    pub async fn get_providers(
        &self,
    ) -> Result<MediaContainerWrapper<ProviderMediaContainer>, anyhow::Error> {
        let providers: MediaContainerWrapper<ProviderMediaContainer> =
            self.get("media/providers").await?.json().await?;
        Ok(providers)
    }

    pub async fn get_artists(
        &self,
        section: String,
        offset: i32,
    ) -> Result<MediaContainerWrapper<MetadataMediaContainer>, anyhow::Error> {
        let _type = 8;
        let artists: MediaContainerWrapper<MetadataMediaContainer> = self.get(format!("library/sections/{}/all?type={}&includeCollections=1&includeExternalMedia=1&includeAdvanced=1&includeMeta=1&X-Plex-Container-Start={}&X-Plex-Container-Size=50", section, _type, offset).as_str()).await?.json().await?;
        Ok(artists)
    }

    pub async fn get_extra_items(
        &self,
        section: &String,
        artist: &String,
        type_id: i32,
    ) -> Result<MediaContainerWrapper<MetadataMediaContainer>, anyhow::Error> {
        //let response = self.get(format!("library/sections/{}/all?format!=EP,Single,Compilation,Live,Soundtrack&artist.id={}&includeMetadata=1&type={}&X-Plex-Container-Start=0&X-Plex-Container-Size=1000", section, artist, type_id).as_str()).await?;
        //println!("{:?}", response.text().await?);
        let response = self.get(format!("library/sections/{}/all?format!=EP,Single,Compilation,Live,Soundtrack&artist.id={}&includeMetadata=1&type={}&X-Plex-Container-Start=0&X-Plex-Container-Size=1000", section, artist, type_id).as_str()).await?;
        let extra_items: MediaContainerWrapper<MetadataMediaContainer> = response.json().await?;
        Ok(extra_items)
    }

    // pub async fn get_extras(
    //     &self,
    //     key: &str,
    // ) -> Result<MediaContainerWrapper<ExtrasMediaContainer>, anyhow::Error> {
    //     let extras: MediaContainerWrapper<ExtrasMediaContainer> = self
    //         .get(format!("library/metadata/{}/related?includeAugmentations=1&includeExternalMetadata=1&includeMeta=1", key).as_str())
    //         .await?
    //         .json()
    //         .await?;
    //     Ok(extras)
    // }

    pub async fn get_metadata(
        &self,
        key: &str,
    ) -> Result<MediaContainerWrapper<MetadataMediaContainer>, anyhow::Error> {
        let extras: MediaContainerWrapper<MetadataMediaContainer> = self
            .get(format!("library/metadata/{}?includeConcerts=1&includeExtras=1&includeOnDeck=1&includePopularLeaves=1&includePreferences=1&includeReviews=1&includeChapters=1&includeStations=1&includeExternalMedia=1&asyncAugmentMetadata=1&asyncCheckFiles=1&asyncRefreshAnalysis=1&asyncRefreshLocalMediaAgent=1", key).as_str())
            .await?
            .json()
            .await?;
        Ok(extras)
    }

    pub async fn get_metadata_children(
        &self,
        key: &str,
    ) -> Result<MediaContainerWrapper<MetadataMediaContainer>, anyhow::Error> {
        let metadata: MediaContainerWrapper<MetadataMediaContainer> = self
            .get(format!("library/metadata/{}/children", key).as_str())
            .await?
            .json()
            .await?;
        Ok(metadata)
    }

    pub async fn create_playlist(
        &self,
        name: &str,
        machine_identifier: &str,
        provider_identifier: &str,
        key: &str,
    ) -> Result<MediaContainerWrapper<MetadataMediaContainer>, anyhow::Error> {
        let response: MediaContainerWrapper<MetadataMediaContainer> = self
            .post(
                format!(
                    "playlists?type=audio&smart=0&uri=server://{}/{}{}&title={}",
                    machine_identifier, provider_identifier, key, name
                )
                .as_str(),
            )
            .await?
            .json()
            .await?;

        Ok(response)
    }

    pub async fn update_playlist(
        &self,
        playlist_key: &str,
        machine_identifier: &str,
        provider_identifier: &str,
        key: &str,
    ) -> Result<MediaContainerWrapper<MetadataMediaContainer>, anyhow::Error> {
        let response: MediaContainerWrapper<MetadataMediaContainer> = self
            .put(
                format!(
                    "playlists/{}/items?uri=server://{}/{}{}",
                    playlist_key, machine_identifier, provider_identifier, key
                )
                .as_str(),
            )
            .await?
            .json()
            .await?;

        Ok(response)
    }
}
