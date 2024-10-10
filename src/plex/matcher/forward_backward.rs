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
            // loop through each artist in the plex track.
            for plex_artist in plex_track.artist.iter() {
                // loop through each artist in the spotify track.
                for spotify_artist in spotify_track.artist.iter() {
                    // if the plex artist starts with the spotify artist or vice versa
                    // and the plex track starts with the spotify track or vice versa
                    // then add the plex track to the playlist and return the plex track.
                    if (plex_artist.starts_with(spotify_artist)
                        && plex_track.track.starts_with(&spotify_track.track))
                        || (spotify_artist.starts_with(plex_artist)
                            && spotify_track.track.starts_with(&plex_track.track))
                    {
                        playlist(playlist_id, &plex, plex_track, playlist_name).await?;
                        return Ok(plex_track.clone());
                    }
                }
            }
        }
        Err(anyhow::anyhow!("No match found"))
    }
}
