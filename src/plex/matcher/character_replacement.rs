use std::collections::HashMap;

use crate::{plex::client::Plex, track_album_artist::TrackAlbumArtist};

use super::{forward_backward::MatchForwardBack, levenshtein::LevenshteinDistance, Matcher};

pub struct MatchWithCharReplacements;
#[async_trait::async_trait]
impl Matcher for MatchWithCharReplacements {
    async fn match_fn(
        &self,
        playlist_id: &mut String,
        plex: &Plex,
        plex_tracks: &Vec<TrackAlbumArtist>,
        spotify_track: &TrackAlbumArtist,
        playlist_name: &String,
    ) -> Result<TrackAlbumArtist, anyhow::Error> {
        let replacements = HashMap::from([
            ("’", "'"),
            ("&", "and"),
            ("-", " "),
            ("(", ""),
            (")", ""),
            (".", ""),
        ]);

        let new_plex_tracks: Vec<TrackAlbumArtist> = plex_tracks
            .iter()
            .map(|p| {
                let mut track = p.clone();
                for (from, to) in replacements.iter() {
                    for artist in track.artist.iter_mut() {
                        *artist = artist.replace(from, to);
                    }
                    track.track = track.track.replace(from, to);
                }
                track
            })
            .collect();

        let mut spotify_track = spotify_track.clone();
        for (from, to) in replacements.iter() {
            for artist in spotify_track.artist.iter_mut() {
                *artist = artist.replace(from, to);
            }
            spotify_track.track = spotify_track.track.replace(from, to);
        }

        let result = MatchForwardBack {}
            .match_fn(
                playlist_id,
                plex,
                &new_plex_tracks,
                &spotify_track,
                playlist_name,
            )
            .await;
        if result.is_ok() {
            return result;
        }
        let result = LevenshteinDistance {}
            .match_fn(
                playlist_id,
                plex,
                &new_plex_tracks,
                &spotify_track,
                playlist_name,
            )
            .await;
        if result.is_ok() {
            return result;
        }

        Err(anyhow::anyhow!("No match found"))
    }
}
