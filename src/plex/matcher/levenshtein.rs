use crate::{
    plex::{client::Plex, playlist},
    track_album_artist::TrackAlbumArtist,
};

use super::Matcher;
use levenshtein::levenshtein;

pub struct LevenshteinDistance;
#[async_trait::async_trait]
impl Matcher for LevenshteinDistance {
    async fn match_fn(
        &self,
        playlist_id: &mut String,
        plex: &Plex,
        plex_tracks: &Vec<TrackAlbumArtist>,
        spotify_track: &TrackAlbumArtist,
        playlist_name: &String,
    ) -> Result<TrackAlbumArtist, anyhow::Error> {
        let mut sorted: Vec<(usize, TrackAlbumArtist)> = plex_tracks
            .iter()
            .enumerate()
            .map(|(_, f)| {
                let artist_distance = levenshtein(&spotify_track.artist, &f.artist);
                let track_distance = levenshtein(&spotify_track.track, &f.track);
                return (artist_distance + track_distance, f.clone());
            })
            .collect();

        sorted.sort_by_key(|(distance, _)| *distance);

        // get highest score?
        if let Some((distance, plex_track)) = sorted.get(0) {
            if *distance < 11 {
                println!(
                    "Closest match ({}): {:?} {:?} {:?} => {:?} {:?} {:?}",
                    distance,
                    spotify_track.artist,
                    spotify_track.album,
                    spotify_track.track,
                    plex_track.artist,
                    plex_track.album,
                    plex_track.track
                );
                playlist(playlist_id, &plex, plex_track, playlist_name).await?;
                return Ok(plex_track.clone());
            }
        };

        Err(anyhow::anyhow!("No match found"))
    }
}
