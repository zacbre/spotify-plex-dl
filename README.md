# spotify-plex-dl
All in one software to sync spotify playlist to plex, music included (using spotydl).

### Plex
- To get your plex token, follow this link: [Here](https://support.plex.tv/articles/204059436-finding-an-authentication-token-x-plex-token/)

### Spotify
- To get your spotify client id and secret, sign up and create an application: [Here](https://developer.spotify.com/dashboard/applications)
- Set the spotify auth callback url to `http://localhost:8888/callback`.

### Usage

`PLEX_URL` can be set to `http://your.plex.ip.here:32400` if you are running this within a local network.

secrets.env file:
```
export PLEX_URL=
export PLEX_TOKEN=

export SPOTIFY_CLIENT_ID=
export SPOTIFY_SECRET_TOKEN=
```

Run with:
```
source secrets.env && ./spotify-plex-dl --playlist-id 3334ksjHmasdhjhA --playlist-name "My Playlist"
```
