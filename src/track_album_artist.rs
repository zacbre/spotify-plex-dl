#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct TrackAlbumArtist {
    pub track: String,
    pub album: String,
    pub artist: String,
    pub metadata: MetadataType,
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct PlexMetadata {
    pub machine_identifier: String,
    pub provider_identifier: String,
    pub rating_key: String,
    pub key: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SpotifyMetadata {
    pub uri: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub enum MetadataType {
    #[default]
    None,
    Plex(PlexMetadata),
    Spotify(SpotifyMetadata),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Artist {
    pub artist: String,
    pub metadata: MetadataType,
}
