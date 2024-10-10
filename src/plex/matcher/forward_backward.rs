use crate::{
    plex::{client::Plex, playlist},
    track_album_artist::TrackAlbumArtist,
};

use super::Matcher;

pub struct MatchForwardBack;
#[async_trait::async_trait]
impl Matcher for MatchForwardBack {
    async fn match_fn(
        &self,
        playlist_id: &mut String,
        plex: &Plex,
        plex_tracks: &Vec<TrackAlbumArtist>,
        spotify_track: &TrackAlbumArtist,
        playlist_name: &String,
    ) -> Result<TrackAlbumArtist, anyhow::Error> {
        for plex_track in plex_tracks.iter() {
            if (plex_track.artist.starts_with(&spotify_track.artist)
                && plex_track.track.starts_with(&spotify_track.track))
                || (spotify_track.artist.starts_with(&plex_track.artist)
                    && spotify_track.track.starts_with(&plex_track.track))
            {
                playlist(playlist_id, &plex, plex_track, playlist_name).await?;
                return Ok(plex_track.clone());
            }
        }
        Err(anyhow::anyhow!("No match found"))
    }
}
