use crate::{plex::client::Plex, track_album_artist::TrackAlbumArtist};

use super::{
    character_replacement::MatchWithCharReplacements, forward_backward::MatchForwardBack,
    levenshtein::LevenshteinDistance, Matcher,
};

pub struct RemoveSections {}
#[async_trait::async_trait]
impl Matcher for RemoveSections {
    async fn match_fn(
        &self,
        playlist_id: &mut String,
        plex: &Plex,
        plex_tracks: &Vec<TrackAlbumArtist>,
        spotify_track: &TrackAlbumArtist,
        playlist_name: &String,
    ) -> Result<TrackAlbumArtist, anyhow::Error> {
        let re = vec![
            regex::Regex::new(r"\(feat.+?\)").unwrap(),
            regex::Regex::new(r"\(.+?remix\)").unwrap(),
            regex::Regex::new(r"\(.+?version\)").unwrap(),
            regex::Regex::new(r"\(.+?radio edit\)").unwrap(),
            regex::Regex::new(r"\(.+?remastered\)").unwrap(),
            regex::Regex::new(r"\(.+?mix\)").unwrap(),
            regex::Regex::new(r"\(.+?\)").unwrap(),
        ];

        let mut spotify_track = spotify_track.clone();
        for r in re.iter() {
            for artist in spotify_track.artist.iter_mut() {
                *artist = r.replace_all(&artist, "").to_string();
            }

            spotify_track.track = r.replace_all(&spotify_track.track, "").to_string();

            let new_plex_tracks: Vec<TrackAlbumArtist> = plex_tracks
                .iter()
                .map(|p| {
                    let mut track = p.clone();
                    for r in re.iter() {
                        for artist in track.artist.iter_mut() {
                            *artist = r.replace_all(&artist, "").to_string();
                        }
                    }
                    track.track = r.replace_all(&track.track, "").to_string();
                    track
                })
                .collect();

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
            let result = MatchWithCharReplacements {}
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
        }
        Err(anyhow::anyhow!("No match found"))
    }
}
