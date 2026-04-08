use crate::events::AppEvent;
use color_eyre::Result;
use futures::StreamExt;
use rspotify::{
    prelude::*,
    scopes,
    AuthCodePkceSpotify,
    Config,
    Credentials,
};
use tokio::sync::mpsc;

pub mod auth;
pub mod lyrics;
pub mod art;

pub struct SpotifyClient {
    pub client: AuthCodePkceSpotify,
    event_tx: mpsc::UnboundedSender<AppEvent>,
}

impl Clone for SpotifyClient {
    fn clone(&self) -> Self {
        Self {
            client: self.client.clone(),
            event_tx: self.event_tx.clone(),
        }
    }
}

impl SpotifyClient {
    pub fn new(client_id: &str, event_tx: mpsc::UnboundedSender<AppEvent>) -> Self {
        let creds = Credentials::new_pkce(client_id);
        let oauth = rspotify::OAuth {
            redirect_uri: "http://localhost:8888/callback".to_string(),
            scopes: scopes!(
                "user-read-playback-state",
                "user-modify-playback-state",
                "user-read-currently-playing",
                "playlist-read-private",
                "playlist-read-collaborative",
                "playlist-modify-public",
                "playlist-modify-private",
                "user-library-read",
                "user-read-playback-position",
                "user-read-recently-played",
                "user-top-read"
            ),
            ..Default::default()
        };
        let config = Config {
            token_cached: true,
            ..Default::default()
        };
        let client = AuthCodePkceSpotify::with_config(creds, oauth, config);
        
        Self {
            client,
            event_tx,
        }
    }

    pub async fn authenticate(&mut self) -> Result<()> {
        // 1. Try to load token from keyring if not already in client
        {
            let token_lock = self.client.get_token();
            let mut token_guard = token_lock.lock().await.unwrap();
            if token_guard.is_none() {
                if let Some(token) = auth::load_token() {
                    *token_guard = Some(token);
                }
            }
        }

        // 2. Check if token is valid or refreshable
        let token_lock = self.client.get_token();
        if let Ok(token_guard) = token_lock.lock().await {
            if let Some(token) = token_guard.as_ref() {
                if !token.is_expired() {
                    return Ok(());
                }
                // If expired, try refreshing
                if let Some(_refresh_token) = &token.refresh_token {
                    drop(token_guard); // Release lock before calling refresh_token
                    if self.client.refresh_token().await.is_ok() {
                        // Store the refreshed token
                        if let Ok(new_token_guard) = self.client.get_token().lock().await {
                            if let Some(new_token) = new_token_guard.as_ref() {
                                let _ = auth::store_token(new_token);
                            }
                        }
                        return Ok(());
                    }
                }
            }
        }

        // 3. Start full PKCE flow
        let url = self.client.get_authorize_url(None)?;
        let callback_server = auth::CallbackServer::new(8888);
        let rx = callback_server.start();

        webbrowser::open(&url)?;

        // Wait for the code from the callback server
        if let Ok(code) = rx.recv() {
            self.client.request_token(&code).await?;
            // Store the new token
            if let Ok(token_guard) = self.client.get_token().lock().await {
                if let Some(token) = token_guard.as_ref() {
                    let _ = auth::store_token(token);
                }
            }
            return Ok(());
        }

        Err(color_eyre::eyre::eyre!("Authentication failed"))
    }

    pub async fn update_playback(&self) {
        match self.client.current_playback(None, None::<Vec<_>>).await {
            Ok(Some(playback)) => {
                let _ = self.event_tx.send(AppEvent::PlaybackUpdated(Box::new(playback)));
            }
            Ok(None) => {}
            Err(e) => {
                let _ = self.event_tx.send(AppEvent::ApiError(e.to_string()));
            }
        }
    }

    pub async fn play_pause(&self) {
        if let Ok(Some(playback)) = self.client.current_playback(None, None::<Vec<_>>).await {
            if playback.is_playing {
                let _ = self.client.pause_playback(None).await;
            } else {
                let _ = self.client.resume_playback(None, None).await;
            }
            self.update_playback().await;
        }
    }

    pub async fn next_track(&self) {
        let _ = self.client.next_track(None).await;
        self.update_playback().await;
    }

    pub async fn previous_track(&self) {
        let _ = self.client.previous_track(None).await;
        self.update_playback().await;
    }

    pub async fn seek(&self, position_ms: u32) {
        let _ = self.client.seek_track(chrono::Duration::milliseconds(position_ms as i64), None).await;
        self.update_playback().await;
    }

    pub async fn volume(&self, volume_percent: u8, device_id: Option<&str>) {
        let _ = self.client.volume(volume_percent, device_id).await;
        self.update_playback().await;
    }

    pub async fn play_radio(&self, seed_track_id: &str) {
        let id = match rspotify::model::TrackId::from_id(seed_track_id) {
            Ok(id) => id,
            Err(_) => return,
        };
        let result = self.client.recommendations(
            std::iter::empty::<rspotify::model::RecommendationsAttribute>(),
            None::<Vec<rspotify::model::ArtistId>>,
            None::<Vec<&str>>,
            Some(vec![id]),
            None,
            Some(20),
        ).await;

        if let Ok(recs) = result {
            let uris: Vec<rspotify::model::PlayableId> = recs.tracks.into_iter()
                .filter_map(|t| t.id.map(rspotify::model::PlayableId::Track))
                .collect();
            if !uris.is_empty() {
                let _ = self.client.start_uris_playback(uris, None, None, None).await;
                self.update_playback().await;
            }
        }
    }

    pub async fn play_track(&self, track_id: &str, context_uri: Option<&str>) {
        let id = rspotify::model::TrackId::from_id(track_id).unwrap();
        
        if let Some(uri) = context_uri {
            let context = rspotify::model::PlayContextId::Playlist(rspotify::model::PlaylistId::from_id(uri).unwrap());
            let _ = self.client.start_context_playback(
                context,
                None,
                Some(rspotify::model::Offset::Uri(id.to_string())),
                None
            ).await;
        } else {
            let _ = self.client.start_uris_playback(
                vec![rspotify::model::PlayableId::Track(id)],
                None,
                None,
                None
            ).await;
        }
        self.update_playback().await;
    }

    pub async fn get_user_playlists(&self) {
        let mut playlists = Vec::new();
        let mut stream = self.client.current_user_playlists();
        while let Some(item) = stream.next().await {
            if let Ok(playlist) = item {
                playlists.push(playlist);
            }
        }
        let _ = self.event_tx.send(AppEvent::PlaylistsLoaded(playlists));
    }

    pub async fn get_playlist_tracks(&self, playlist_id: &str) {
        let mut tracks = Vec::new();
        let id = match rspotify::model::PlaylistId::from_id(playlist_id) {
            Ok(id) => id,
            Err(_) => return,
        };
        let mut stream = self.client.playlist_items(id, None, None);
        while let Some(item) = stream.next().await {
            if let Ok(track) = item {
                tracks.push(track);
            }
        }
        let _ = self.event_tx.send(AppEvent::PlaylistTracksLoaded(playlist_id.to_string(), tracks));
    }

    pub async fn search(&self, query: &str) {
        use rspotify::model::SearchType;
        let types = [SearchType::Track, SearchType::Album, SearchType::Artist];
        let result = self.client.search_multiple(
            query,
            types,
            None,
            None,
            Some(20),
            None
        ).await;

        match result {
            Ok(search_result) => {
                let _ = self.event_tx.send(AppEvent::SearchResults(search_result));
            }
            Err(e) => {
                let _ = self.event_tx.send(AppEvent::ApiError(e.to_string()));
            }
        }
    }

    pub async fn get_devices(&self) {
        match self.client.device().await {
            Ok(devices) => {
                let _ = self.event_tx.send(AppEvent::DevicesLoaded(devices));
            }
            Err(e) => {
                let _ = self.event_tx.send(AppEvent::ApiError(e.to_string()));
            }
        }
    }

    pub async fn transfer_playback(&self, device_id: &str) {
        let _ = self.client.transfer_playback(device_id, None).await;
        self.get_devices().await;
        self.update_playback().await;
    }

    pub async fn add_to_queue(&self, track_id: &str) {
        let id = rspotify::model::TrackId::from_id(track_id).unwrap();
        let _ = self.client.add_item_to_queue(rspotify::model::PlayableId::Track(id), None).await;
    }

    pub async fn create_playlist(&self, name: &str, public: bool) {
        if let Ok(user) = self.client.current_user().await {
            let _ = self.client.user_playlist_create(user.id, name, Some(public), None, None).await;
            self.get_user_playlists().await;
        }
    }

    pub async fn delete_playlist(&self, playlist_id: &str) {
        if let Ok(id) = rspotify::model::PlaylistId::from_id(playlist_id) {
            let _ = self.client.playlist_unfollow(id).await;
            self.get_user_playlists().await;
        }
    }

    pub async fn reorder_playlist_tracks(&self, playlist_id: &str, range_start: u32, insert_before: u32) {
        if let Ok(id) = rspotify::model::PlaylistId::from_id(playlist_id) {
            let _ = self.client.playlist_reorder_items(id, Some(range_start as i32), None, Some(insert_before), None).await;
            self.get_playlist_tracks(playlist_id).await;
        }
    }

    pub async fn get_queue(&self) {
        match self.client.current_user_queue().await {
            Ok(queue) => {
                let mut tracks = Vec::new();
                for item in queue.queue {
                    if let rspotify::model::PlayableItem::Track(t) = item {
                        tracks.push(t);
                    }
                }
                let _ = self.event_tx.send(AppEvent::QueueLoaded(tracks));
            }
            Err(e) => {
                let _ = self.event_tx.send(AppEvent::ApiError(e.to_string()));
            }
        }
    }
}
