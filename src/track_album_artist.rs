use serde::Serialize;

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct TrackAlbumArtist {
    pub track: String,
    pub album: String,
    pub artist: Vec<String>,
    pub metadata: MetadataType,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct PlexMetadata {
    pub machine_identifier: String,
    pub provider_identifier: String,
    pub rating_key: String,
    pub key: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct SpotifyMetadata {
    pub uri: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub enum MetadataType {
    Plex(PlexMetadata),
    Spotify(SpotifyMetadata),
}
