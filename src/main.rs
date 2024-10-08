mod plex;
mod track_album_artist;

use std::collections::{BTreeMap, HashMap};

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

    let mut dupe_list: BTreeMap<String, (TrackAlbumArtist, TrackAlbumArtist)> = BTreeMap::new();

    // try to find a single match?
    'outer: for spotify_track in spotify_tracks.iter() {
        // println!(
        //     "Looking for {} - {}",
        //     spotify_track.artist, spotify_track.track
        // );
        //
        let list_of_fns: Vec<Box<dyn Matcher>> = vec![
            Box::new(MatchForwardBack {}),
            Box::new(LevenshteinDistance {}),
            Box::new(MatchWithCharReplacements {}),
        ];

        for fun in list_of_fns {
            let result = fun
                .match_fn(&mut playlist_id, &plex, &plex_tracks, &spotify_track)
                .await;
            if result.is_ok() {
                let track_result = result.unwrap();
                if let MetadataType::Plex(meta) = &track_result.metadata {
                    if !dupe_list.contains_key(&meta.rating_key) {
                        dupe_list.insert(
                            meta.rating_key.clone(),
                            (spotify_track.clone(), track_result.clone()),
                        );
                    } else {
                        let matched = dupe_list.get(&meta.rating_key);
                        println!(
                            "Found a duplicate result: {:?} => {:?} {:?} {:?}",
                            matched, track_result.artist, track_result.album, track_result.track
                        );
                    }
                }
                continue 'outer;
            }
        }
        // if it gets here, we can't find a match.
        println!("No match found for {:?}", spotify_track);
        for plex_track in plex_tracks.iter() {
            if plex_track.artist == spotify_track.artist || plex_track.album == spotify_track.album
            {
                println!("Match from the artist/album: {:?}", plex_track);
            }
        }
        println!("===================================");
        /*

        println!("No match found for {:?}", spotify_track);
                println!("Closest match ({}): {:?}", distance, plex_track);
                // get all the matches with the same artist.
                for plex_track in plex_tracks.iter() {
                    if plex_track.artist == spotify_track.artist || plex_track.album == spotify_track.album
                    {
                        println!("Match from the artist/album: {:?}", plex_track);
                    }
                }
                println!("=========================================="); */
    }

    Ok(())
}

#[async_trait::async_trait]
trait Matcher: Send + Sync {
    async fn match_fn(
        &self,
        playlist_id: &mut String,
        plex: &Plex,
        plex_tracks: &Vec<TrackAlbumArtist>,
        spotify_track: &TrackAlbumArtist,
    ) -> Result<TrackAlbumArtist, anyhow::Error>;
}

struct MatchForwardBack;
#[async_trait::async_trait]
impl Matcher for MatchForwardBack {
    async fn match_fn(
        &self,
        playlist_id: &mut String,
        plex: &Plex,
        plex_tracks: &Vec<TrackAlbumArtist>,
        spotify_track: &TrackAlbumArtist,
    ) -> Result<TrackAlbumArtist, anyhow::Error> {
        for plex_track in plex_tracks.iter() {
            if (plex_track.artist.starts_with(&spotify_track.artist)
                && plex_track.track.starts_with(&spotify_track.track))
                || (spotify_track.artist.starts_with(&plex_track.artist)
                    && spotify_track.track.starts_with(&plex_track.track))
            {
                playlist(playlist_id, &plex, plex_track).await?;
                return Ok(plex_track.clone());
            }
        }
        Err(anyhow::anyhow!("No match found"))
    }
}

struct LevenshteinDistance;
#[async_trait::async_trait]
impl Matcher for LevenshteinDistance {
    async fn match_fn(
        &self,
        playlist_id: &mut String,
        plex: &Plex,
        plex_tracks: &Vec<TrackAlbumArtist>,
        spotify_track: &TrackAlbumArtist,
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
                playlist(playlist_id, &plex, plex_track).await?;
                return Ok(plex_track.clone());
            }
        };

        Err(anyhow::anyhow!("No match found"))
    }
}

struct MatchWithCharReplacements;
#[async_trait::async_trait]
impl Matcher for MatchWithCharReplacements {
    async fn match_fn(
        &self,
        playlist_id: &mut String,
        plex: &Plex,
        plex_tracks: &Vec<TrackAlbumArtist>,
        spotify_track: &TrackAlbumArtist,
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
                    track.artist = track.artist.replace(from, to);
                    track.track = track.track.replace(from, to);
                }
                track
            })
            .collect();

        let mut spotify_track = spotify_track.clone();
        for (from, to) in replacements.iter() {
            spotify_track.artist = spotify_track.artist.replace(from, to);
            spotify_track.track = spotify_track.track.replace(from, to);
        }

        let result = MatchForwardBack {}
            .match_fn(playlist_id, plex, &new_plex_tracks, &spotify_track)
            .await;
        if result.is_ok() {
            return result;
        }
        let result = LevenshteinDistance {}
            .match_fn(playlist_id, plex, &new_plex_tracks, &spotify_track)
            .await;
        if result.is_ok() {
            return result;
        }

        Err(anyhow::anyhow!("No match found"))
    }
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
                    &std::env::var("PLAYLIST_NAME").expect("PLAYLIST_NAME not set"),
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

    let playlist_id = std::env::var("PLAYLIST_ID").expect("PLAYLIST_ID not set");

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
                    artist: track.artists[0].name.to_lowercase().clone(),
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
                    artist: episode.show.publisher.to_lowercase().clone(),
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

async fn get_plex_tracks(plex: &Plex) -> Result<Vec<TrackAlbumArtist>, anyhow::Error> {
    let mut tracks = Vec::new();
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
    let mut i = 1;
    for artist in artists_final.iter() {
        //println!("artist: {:?}", artist.title);
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
            //println!("\talbum: {:?} ({})", meta.title, meta.rating_key);
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
                // track album artist.
                let track_album_artist = TrackAlbumArtist {
                    track: track.title.to_lowercase().clone(),
                    album: meta.title.to_lowercase().clone(),
                    artist: artist.title.to_lowercase(),
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
                tracks.push(track_album_artist);
                if track.original_title.is_some() {
                    // there might be an additional artist, so try to match that too?
                    let track_album_artist = TrackAlbumArtist {
                        track: track.title.to_lowercase().clone(),
                        album: meta.title.to_lowercase().clone(),
                        artist: track.original_title.as_ref().unwrap().to_lowercase(),
                        metadata: MetadataType::Plex(PlexMetadata {
                            machine_identifier: providers
                                .media_container
                                .machine_identifier
                                .clone(),
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
                    tracks.push(track_album_artist);
                }
            }
        }

        print!(
            "\rProcessing Plex Artist {} of {}, tracks found: {}               ",
            i,
            artists_final.len(),
            tracks.len()
        );
        i += 1;
    }
    println!("");

    Ok(tracks)
}
