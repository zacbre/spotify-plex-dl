pub mod character_replacement;
pub mod forward_backward;
pub mod levenshtein;

use crate::track_album_artist::TrackAlbumArtist;

use super::client::Plex;

#[async_trait::async_trait]
pub trait Matcher: Send + Sync {
    async fn match_fn(
        &self,
        playlist_id: &mut String,
        plex: &Plex,
        plex_tracks: &Vec<TrackAlbumArtist>,
        spotify_track: &TrackAlbumArtist,
        playlist_name: &String,
    ) -> Result<TrackAlbumArtist, anyhow::Error>;
}
