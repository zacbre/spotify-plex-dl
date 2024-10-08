mod plex;
mod track_album_artist;

use std::collections::HashMap;

use levenshtein::levenshtein;
use plex::{
    client::Plex,
    models::{metadata::Metadata, providers::ProviderMediaContainer, MediaContainerWrapper},
};
use rspotify::{
    model::{PlayableItem, PlaylistId},
    prelude::{BaseClient, OAuthClient},
    scopes, AuthCodeSpotify, Credentials, OAuth,
};
use track_album_artist::{MetadataType, PlexMetadata, SpotifyMetadata, TrackAlbumArtist};

fn get_music_provider(providers: &MediaContainerWrapper<ProviderMediaContainer>) -> Option<String> {
    for provider in providers.media_container.media_provider.iter() {
        for feature in &provider.features {
            if feature.directories.is_none() {
                continue;
            }

            for directory in feature.directories.as_ref().unwrap() {
                if directory.title == Some("Music".to_string()) {
                    //println!("found music provider: {:?}", provider);
                    return directory.id.clone();
                }
            }
        }
    }
    None
}

#[tokio::main]
async fn main() -> Result<(), anyhow::Error> {
    // You can use any logger for debugging.
    env_logger::init();

    let plex_url = std::env::var("PLEX_URL").expect("PLEX_URL not set");
    let plex_token = std::env::var("PLEX_TOKEN").expect("PLEX_TOKEN not set");
    let spotify_client_id = std::env::var("SPOTIFY_CLIENT_ID").expect("SPOTIFY_CLIENT_ID not set");
    let spotify_secret_token =
        std::env::var("SPOTIFY_SECRET_TOKEN").expect("SPOTIFY_SECRET_TOKEN not set");

    let plex = plex::client::Plex::new(plex_url, plex_token);

    let spotify_tracks = get_spotify_tracks(spotify_client_id, spotify_secret_token).await?;
    let plex_tracks = get_plex_tracks(&plex).await?;

    let mut playlist_id = String::default();

    // try to find a single match?
    'outer: for (key, spotify_track) in spotify_tracks.iter() {
        println!(
            "Looking for {} - {}",
            spotify_track.artist, spotify_track.track
        );
        if let Some(plex_track) = plex_tracks.get(key) {
            //println!("found a match: {:?}", plex_track);
            playlist(&mut playlist_id, &plex, &plex_track).await?;
        } else {
            println!("No match found for {:?}", spotify_track);
            // try to find a reverse match with starts_with?
            for (_, plex_track) in plex_tracks.iter() {
                if (plex_track.artist.starts_with(&spotify_track.artist)
                    && plex_track.track.starts_with(&spotify_track.track))
                    || (spotify_track.artist.starts_with(&plex_track.artist)
                        && spotify_track.track.starts_with(&plex_track.track))
                {
                    println!("Possible match: {:?}", plex_track);
                    playlist(&mut playlist_id, &plex, plex_track).await?;
                    continue 'outer;
                }
            }
            // can't find a match? levenstein distance on artist and track
            let mut sorted: Vec<(usize, TrackAlbumArtist)> = plex_tracks
                .iter()
                .enumerate()
                .map(|(_, (_, f))| {
                    let artist_distance = levenshtein(&spotify_track.artist, &f.artist);
                    let track_distance = levenshtein(&spotify_track.track, &f.track);
                    return (artist_distance + track_distance, f.clone());
                })
                .collect();

            sorted.sort_by_key(|(distance, _)| *distance);

            // get highest score?
            if let Some((_, plex_track)) = sorted.get(0) {
                println!("Closest match: {:?}", plex_track);
                playlist(&mut playlist_id, &plex, plex_track).await?;
                continue 'outer;
            }

            println!("No match found for {:?}", spotify_track);
        }
    }

    Ok(())
}

async fn playlist(
    playlist_id: &mut String,
    plex: &Plex,
    plex_track: &TrackAlbumArtist,
) -> Result<(), anyhow::Error> {
    if playlist_id.len() <= 0 {
        if let MetadataType::Plex(meta) = &plex_track.metadata {
            let playlist = plex
                .create_playlist(
                    "Playlist",
                    meta.machine_identifier.as_str(),
                    meta.provider_identifier.as_str(),
                    meta.key.as_str(),
                )
                .await?;
            *playlist_id = playlist
                .media_container
                .metadata
                .unwrap()
                .get(0)
                .unwrap()
                .rating_key
                .clone();
        }
    } else {
        if let MetadataType::Plex(meta) = &plex_track.metadata {
            let _ = plex
                .update_playlist(
                    playlist_id.as_str(),
                    meta.machine_identifier.as_str(),
                    meta.provider_identifier.as_str(),
                    meta.key.as_str(),
                )
                .await?;
        }
    }
    Ok(())
}

async fn get_spotify_tracks(
    client_id: String,
    secret_token: String,
) -> Result<HashMap<String, TrackAlbumArtist>, anyhow::Error> {
    let mut tracks: HashMap<String, TrackAlbumArtist> = HashMap::new();
    let creds = Credentials::new(&client_id, &secret_token);
    let oauth = OAuth {
        redirect_uri: "http://localhost:8888/callback".to_string(),
        scopes: scopes!("playlist-read-private"),
        ..Default::default()
    };

    let mut spotify = AuthCodeSpotify::new(creds, oauth);
    spotify.config.token_cached = true;

    // Obtaining the access token
    let url = spotify.get_authorize_url(false).unwrap();
    spotify.prompt_for_token(&url)?;

    let playlist_id = std::env::var("PLAYLIST_ID").expect("PLAYLIST_ID not set");

    let stream = spotify.playlist_items(PlaylistId::from_id(playlist_id).unwrap(), None, None);

    let playable: Vec<PlayableItem> = stream.map(|item| item.unwrap().track.unwrap()).collect();
    // map to TrackAlbumArtist
    playable.iter().for_each(|p| match p {
        PlayableItem::Track(track) => {
            let track_album_artist = TrackAlbumArtist {
                track: track.name.clone(),
                album: track.album.name.clone(),
                artist: track.artists[0].name.clone(),
                metadata: MetadataType::Spotify(SpotifyMetadata {
                    uri: track.href.as_ref().unwrap().clone(),
                }),
            };
            tracks.insert(
                format!(
                    "{}{}",
                    track_album_artist.artist.replace("’", "'"),
                    track_album_artist.track.replace("’", "'")
                )
                .to_lowercase(),
                track_album_artist,
            );
        }
        PlayableItem::Episode(episode) => {
            let track_album_artist = TrackAlbumArtist {
                track: episode.name.clone(),
                album: episode.show.name.clone(),
                artist: episode.show.publisher.clone(),
                metadata: MetadataType::Spotify(SpotifyMetadata {
                    uri: episode.href.clone(),
                }),
            };
            tracks.insert(
                format!(
                    "{}{}",
                    track_album_artist.artist.replace("’", "'"),
                    track_album_artist.track.replace("’", "'")
                )
                .to_lowercase(),
                track_album_artist,
            );
        }
    });
    Ok(tracks)
}

async fn get_plex_tracks(plex: &Plex) -> Result<HashMap<String, TrackAlbumArtist>, anyhow::Error> {
    let mut tracks = HashMap::new();
    // access to plex.

    let providers = plex.get_providers().await?;

    // find provider with name "Music"
    let provider = get_music_provider(&providers).expect("no music provider found");

    let mut artists_final: Vec<Metadata> = vec![];
    let mut offset = 0;
    let mut total = 1;
    while offset < total {
        let artists = plex.get_artists(provider.clone(), offset).await?;
        total = artists.media_container.total_size.unwrap();
        offset += artists.media_container.size;
        artists_final.extend(artists.media_container.metadata.unwrap());
    }

    // get total.
    //println!("Artists Total: {:?}", artists_final.len());

    for artist in artists_final.iter() {
        println!("artist: {:?}", artist.title);
        // get any additional metadata from artist.
        let mut metadata: Vec<Metadata> = Vec::new();
        let extras = plex.get_extras(&artist.rating_key).await?;
        for extra in extras.media_container.hub.iter() {
            if extra.metadata.is_none()
                || extra.rtype != "album"
                || (extra.context.is_some() && extra.context.clone().unwrap().contains("external"))
            {
                continue;
            }
            for meta in extra.metadata.as_ref().unwrap().iter() {
                // get metadata.
                let album_meta = plex.get_metadata(&meta.rating_key).await?;
                if album_meta.media_container.metadata.is_none() {
                    continue;
                }
                for meta in album_meta.media_container.metadata.as_ref().unwrap().iter() {
                    metadata.push(meta.clone());
                }
            }
        }
        let albums = plex.get_metadata_children(&artist.rating_key).await?;
        if albums.media_container.metadata.is_some() {
            for album in albums.media_container.metadata.as_ref().unwrap().iter() {
                metadata.push(album.clone());
            }
        }

        for meta in metadata {
            println!("\talbum: {:?} ({})", meta.title, meta.rating_key);
            let tracks_meta = plex.get_metadata_children(&meta.rating_key).await?;
            if tracks_meta.media_container.metadata.is_none() {
                continue;
            }

            for track in tracks_meta
                .media_container
                .metadata
                .as_ref()
                .unwrap()
                .iter()
            {
                println!("\t\ttrack: {:?}", track.title);
                let artist_title = if track.original_title.is_some() {
                    track.original_title.clone().unwrap()
                } else {
                    artist.title.clone()
                };
                // track album artist.
                let track_album_artist = TrackAlbumArtist {
                    track: track.title.clone(),
                    album: meta.title.clone(),
                    artist: artist_title,
                    metadata: MetadataType::Plex(PlexMetadata {
                        machine_identifier: providers.media_container.machine_identifier.clone(),
                        provider_identifier: providers
                            .media_container
                            .media_provider
                            .get(0)
                            .expect("no media provider found")
                            .identifier
                            .clone(),
                        rating_key: track.rating_key.clone(),
                        key: track.key.clone(),
                    }),
                };
                tracks.insert(
                    format!(
                        "{}{}",
                        track_album_artist.artist.replace("’", "'"),
                        track_album_artist.track.replace("’", "'")
                    )
                    .to_lowercase(),
                    track_album_artist,
                );
            }
        }
    }

    Ok(tracks)
}
