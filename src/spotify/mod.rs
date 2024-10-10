use rspotify::{
    model::{FullArtist, FullTrack, PlayableItem, PlaylistId, SimplifiedArtist},
    prelude::{BaseClient, OAuthClient},
    scopes, AuthCodeSpotify, Credentials, OAuth,
};

use crate::track_album_artist::{MetadataType, SpotifyMetadata, TrackAlbumArtist};

pub async fn get_spotify_tracks(
    client_id: String,
    secret_token: String,
    playlist_id: String,
) -> Result<Vec<TrackAlbumArtist>, anyhow::Error> {
    let mut tracks: Vec<TrackAlbumArtist> = Vec::new();
    let creds = Credentials::new(&client_id, &secret_token);
    let oauth = OAuth {
        redirect_uri: "http://localhost:8888/callback".to_string(),
        scopes: scopes!("playlist-read-private"),
        ..Default::default()
    };

    let mut spotify = AuthCodeSpotify::new(creds, oauth);
    spotify.config.token_cached = true;

    let url = spotify.get_authorize_url(false).unwrap();
    spotify.prompt_for_token(&url)?;

    let stream = spotify.playlist_items(PlaylistId::from_id(playlist_id).unwrap(), None, None);

    let playable: Vec<PlayableItem> = stream.map(|item| item.unwrap().track.unwrap()).collect();
    let mut i = 1;
    // map to TrackAlbumArtist
    playable.iter().for_each(|p| {
        match p {
            PlayableItem::Track(track) => {
                let track_album_artist = TrackAlbumArtist {
                    track: track.name.to_lowercase().clone(),
                    album: track.album.name.to_lowercase().clone(),
                    artist: track
                        .artists
                        .iter()
                        .map(|a| a.name.trim().to_lowercase().clone())
                        .collect(),
                    metadata: MetadataType::Spotify(SpotifyMetadata {
                        uri: track.href.as_ref().unwrap().clone(),
                    }),
                };
                tracks.push(track_album_artist.clone());
            }
            PlayableItem::Episode(episode) => {
                let track_album_artist = TrackAlbumArtist {
                    track: episode.name.to_lowercase().clone(),
                    album: episode.show.name.to_lowercase().clone(),
                    artist: vec![episode.show.publisher.to_lowercase().clone()],
                    metadata: MetadataType::Spotify(SpotifyMetadata {
                        uri: episode.href.clone(),
                    }),
                };
                tracks.push(track_album_artist.clone());
            }
        }
        print!(
            "\rProcessing Spotify {} of {}               ",
            i,
            playable.len()
        );
        i += 1;
    });
    println!("");
    Ok(tracks)
}
