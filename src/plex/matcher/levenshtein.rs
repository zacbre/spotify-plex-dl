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
        let mut sorted: Vec<(usize, TrackAlbumArtist, String, String)> = plex_tracks
            .iter()
            .enumerate()
            .map(|(_, f)| {
                // get which one has the lowest distance.
                let mut distances: Vec<(usize, String, String)> = f
                    .artist
                    .iter()
                    .map(|artist| {
                        spotify_track.artist.iter().map(|spotify_artist| {
                            let spotify_artist = spotify_artist.clone();
                            let artist = artist.clone();
                            (
                                levenshtein(&spotify_artist, &artist),
                                spotify_artist.clone(),
                                artist.clone(),
                            )
                        })
                    })
                    .flatten()
                    .collect();
                distances.sort_by_key(|(d, _, _)| *d);
                let (artist_distance, spotify_artist, plex_artist) = distances.get(0).unwrap();
                let track_distance = levenshtein(&spotify_track.track, &f.track);
                return (
                    *artist_distance + track_distance,
                    f.clone(),
                    spotify_artist.clone(),
                    plex_artist.clone(),
                );
            })
            .collect();

        sorted.sort_by_key(|(distance, _, _, _)| *distance);

        // get highest score?
        if let Some((distance, plex_track, spotify_artist, plex_artist)) = sorted.get(0) {
            if *distance <= 4 {
                println!(
                    "Closest match ({}): {} - {} => {} - {}",
                    distance,
                    spotify_artist,
                    //spotify_track.album,
                    spotify_track.track,
                    plex_artist,
                    //plex_track.album,
                    plex_track.track
                );
                playlist(playlist_id, &plex, plex_track, playlist_name).await?;
                return Ok(plex_track.clone());
            }
        };

        Err(anyhow::anyhow!("No match found"))
    }
}
