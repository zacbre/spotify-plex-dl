use client::Plex;
use models::{metadata::Metadata, providers::ProviderMediaContainer, MediaContainerWrapper};

use crate::track_album_artist::{MetadataType, PlexMetadata, TrackAlbumArtist};

pub mod client;
pub mod matcher;
pub mod models;

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

pub async fn get_plex_tracks(plex: &Plex) -> Result<Vec<TrackAlbumArtist>, anyhow::Error> {
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

pub(crate) async fn playlist(
    playlist_id: &mut String,
    plex: &Plex,
    plex_track: &TrackAlbumArtist,
    playlist_name: &String,
) -> Result<(), anyhow::Error> {
    if playlist_id.len() <= 0 {
        if let MetadataType::Plex(meta) = &plex_track.metadata {
            let playlist = plex
                .create_playlist(
                    playlist_name,
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
