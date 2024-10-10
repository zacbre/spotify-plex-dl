mod plex;
mod spotify;
mod track_album_artist;

use std::collections::BTreeMap;

use clap::{arg, command, Parser};
use plex::{
    client::Plex,
    get_plex_tracks,
    matcher::{
        character_replacement::MatchWithCharReplacements, forward_backward::MatchForwardBack,
        levenshtein::LevenshteinDistance, remove_sections::RemoveSections, Matcher,
    },
};
use spotify::get_spotify_tracks;
use track_album_artist::{MetadataType, TrackAlbumArtist};

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    #[arg(short = 'i', long)]
    playlist_id: String,
    #[arg(short = 'n', long)]
    playlist_name: String,

    #[arg(short = 'c', long, required = false)]
    spotify_client_id: Option<String>,
    #[arg(short = 's', long, required = false)]
    spotify_client_secret: Option<String>,

    #[arg(short = 'u', long, required = false)]
    plex_url: Option<String>,
    #[arg(short = 't', long, required = false)]
    plex_token: Option<String>,

    #[arg(short = 'd', long, required = false)]
    dump_tracks: bool,
}

#[tokio::main]
async fn main() -> Result<(), anyhow::Error> {
    // You can use any logger for debugging.
    env_logger::init();

    let args = Args::parse();

    let spotify_client_id = match args.spotify_client_id {
        Some(client_id) => client_id,
        None => std::env::var("SPOTIFY_CLIENT_ID").expect("SPOTIFY_CLIENT_ID not set"),
    };

    let spotify_client_secret = match args.spotify_client_secret {
        Some(client_secret) => client_secret,
        None => std::env::var("SPOTIFY_CLIENT_SECRET").expect("SPOTIFY_CLIENT_SECRET not set"),
    };

    let plex_url = match args.plex_url {
        Some(plex_url) => plex_url,
        None => std::env::var("PLEX_URL").expect("PLEX_URL not set"),
    };

    let plex_token = match args.plex_token {
        Some(plex_token) => plex_token,
        None => std::env::var("PLEX_TOKEN").expect("PLEX_TOKEN not set"),
    };

    let plex = plex::client::Plex::new(plex_url, plex_token);
    let spotify_tracks =
        get_spotify_tracks(spotify_client_id, spotify_client_secret, args.playlist_id).await?;
    let plex_tracks = get_plex_tracks(&plex).await?;

    if args.dump_tracks {
        let file = std::fs::File::create("plex_tracks.json")?;
        serde_json::to_writer(&file, &plex_tracks)?;
        let file = std::fs::File::create("spotify_tracks.json")?;
        serde_json::to_writer(&file, &spotify_tracks)?;
        return Ok(());
    }

    // try to find a single match?
    find_matches_and_update_playlist(&plex, &spotify_tracks, &plex_tracks, &args.playlist_name)
        .await?;

    Ok(())
}

async fn find_matches_and_update_playlist(
    plex: &Plex,
    spotify_tracks: &Vec<TrackAlbumArtist>,
    plex_tracks: &Vec<TrackAlbumArtist>,
    playlist_name: &String,
) -> Result<(), anyhow::Error> {
    let mut playlist_id = String::default();
    let mut dupe_list: BTreeMap<String, (TrackAlbumArtist, TrackAlbumArtist)> = BTreeMap::new();
    'outer: for spotify_track in spotify_tracks.iter() {
        let list_of_fns: Vec<Box<dyn Matcher>> = vec![
            Box::new(MatchForwardBack {}),
            Box::new(LevenshteinDistance {}),
            Box::new(MatchWithCharReplacements {}),
            Box::new(RemoveSections {}),
        ];

        for fun in list_of_fns {
            let result = fun
                .match_fn(
                    &mut playlist_id,
                    &plex,
                    &plex_tracks,
                    &spotify_track,
                    &playlist_name,
                )
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
                        let matched = dupe_list.get(&meta.rating_key).unwrap();
                        println!(
                            "Found a duplicate result: ([{:?} - {}] already matched [{:?} - {}]) => new match: {:?} - {}",
                            matched.0.artist, matched.0.track, matched.1.artist, matched.1.track, track_result.artist, track_result.track
                        );
                    }
                }
                continue 'outer;
            }
        }
        println!("===================================");
        println!(
            "No match found for: {:?} - {}",
            spotify_track.artist, spotify_track.track
        );
        let mut immediate_dupe_list = BTreeMap::new();
        for plex_track in plex_tracks.iter() {
            if plex_track.artist == spotify_track.artist || plex_track.album == spotify_track.album
            {
                let key = format!("{:?}{}", plex_track.artist, plex_track.album);
                if immediate_dupe_list.contains_key(&key) {
                    continue;
                }
                immediate_dupe_list.insert(key, plex_track);
                println!(
                    "Match from the artist/album: {:?} - {}",
                    plex_track.artist, plex_track.track
                );
            }
        }
        println!("===================================");
    }

    Ok(())
}
