use std::io;
use std::time::Duration;

use color_eyre::Result;
use crossterm::event::{self, Event, KeyCode, MouseEventKind};
use crossterm::terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen};
use crossterm::ExecutableCommand;
use ratatui::backend::CrosstermBackend;
use ratatui::Terminal;
use tokio::sync::mpsc;
use tokio::time::interval;

mod events;
mod state;
mod ui;
mod api;
mod config;

use ratatui::layout::Rect;
use crate::events::AppEvent;
use crate::state::{AppState, InputMode};
use crate::api::SpotifyClient;

#[tokio::main]
async fn main() -> Result<()> {
    // Setup logging/error handling
    color_eyre::install()?;
    
    // Setup terminal
    enable_raw_mode()?;
    io::stdout().execute(EnterAlternateScreen)?;
    io::stdout().execute(event::EnableMouseCapture)?;
    let mut terminal = Terminal::new(CrosstermBackend::new(io::stdout()))?;

    // Setup event channel
    let (event_tx, mut event_rx) = mpsc::unbounded_channel::<AppEvent>();

    // Input listener task
    let input_tx = event_tx.clone();
    tokio::spawn(async move {
        loop {
            if event::poll(Duration::from_millis(100)).unwrap_or(false) {
                match event::read() {
                    Ok(Event::Key(key)) => {
                        if let Err(_) = input_tx.send(AppEvent::Key(key)) {
                            break;
                        }
                    }
                    Ok(Event::Resize(w, h)) => {
                        if let Err(_) = input_tx.send(AppEvent::Resize(w, h)) {
                            break;
                        }
                    }
                    Ok(Event::Mouse(mouse)) => {
                        if let Err(_) = input_tx.send(AppEvent::Mouse(mouse)) {
                            break;
                        }
                    }
                    _ => {}
                }
            }
        }
    });

    // Tick task (1s)
    let tick_tx = event_tx.clone();
    tokio::spawn(async move {
        let mut interval = interval(Duration::from_secs(1));
        loop {
            interval.tick().await;
            if let Err(_) = tick_tx.send(AppEvent::Tick) {
                break;
            }
        }
    });

    // Poll task (5s)
    let poll_tx = event_tx.clone();
    tokio::spawn(async move {
        let mut interval = interval(Duration::from_secs(5));
        loop {
            interval.tick().await;
            if let Err(_) = poll_tx.send(AppEvent::Poll) {
                break;
            }
        }
    });

    let config = crate::config::Config::load();
    let mut state = AppState::new(event_tx.clone(), config.clone());

    // Initialize Spotify client
    let client_id = if config.auth.client_id != "PLACEHOLDER" {
        config.auth.client_id.clone()
    } else {
        std::env::var("SPOTIFY_CLIENT_ID").unwrap_or_else(|_| "PLACEHOLDER".to_string())
    };
    let mut spotify = SpotifyClient::new(&client_id, event_tx.clone());

    // Try to authenticate before entering the main loop
    // Note: In a real app, we might want to do this asynchronously or in a way that doesn't block the UI if it takes long.
    // But for the foundation, let's try it here.
    if client_id != "PLACEHOLDER" {
        if let Err(e) = spotify.authenticate().await {
            state.last_error = Some(format!("Auth failed: {}", e));
        } else {
            // Fetch initial data
            let client = spotify.clone();
            tokio::spawn(async move {
                client.get_user_playlists().await;
                client.update_playback().await;
            });
        }
    } else {
        state.last_error = Some("SPOTIFY_CLIENT_ID not set".to_string());
    }

    // Main loop
    loop {
        terminal.draw(|f| {
            ui::render(f, &mut state);
        })?;

        if !state.is_running {
            break;
        }

        if let Some(event) = event_rx.recv().await {
            match event {
                AppEvent::PlaybackUpdated(playback) => {
                    let old_track_id = state.playback.as_ref()
                        .and_then(|p| p.item.as_ref())
                        .map(|i| match i {
                            rspotify::model::PlayableItem::Track(t) => t.id.as_ref().map(|id| id.to_string()),
                            rspotify::model::PlayableItem::Episode(e) => Some(e.id.to_string()),
                        })
                        .flatten();
                    
                    let new_track_id = playback.item.as_ref()
                        .map(|i| match i {
                            rspotify::model::PlayableItem::Track(t) => t.id.as_ref().map(|id| id.to_string()),
                            rspotify::model::PlayableItem::Episode(e) => Some(e.id.to_string()),
                        })
                        .flatten();

                    state.playback = Some(playback);

                    if new_track_id != old_track_id && new_track_id.is_some() {
                        // Track changed! Fetch lyrics and art
                        let track_info = state.playback.as_ref().and_then(|p| p.item.as_ref());
                        if let Some(rspotify::model::PlayableItem::Track(t)) = track_info {
                            let artist = t.artists.first().map(|a| a.name.clone()).unwrap_or_default();
                            let title = t.name.clone();
                            let album_id = t.album.id.as_ref().map(|id| id.to_string()).unwrap_or_default();
                            let art_url = t.album.images.iter()
                                .find(|i| i.width.unwrap_or(0) >= 300)
                                .or(t.album.images.first())
                                .map(|i| i.url.clone());

                            // Fetch lyrics
                            let tx = event_tx.clone();
                            state.lyrics_loading = true;
                            state.lyrics = None;
                            state.synced_lyrics = Vec::new();
                            let provider_name = state.config.lyrics.provider.clone();
                            let genius_key = state.config.lyrics.genius_api_key.clone();
                            
                            tokio::spawn(async move {
                                use crate::api::lyrics::{LyricsProvider, LyricsOvhProvider, GeniusLyricsProvider, LrcLibProvider};
                                
                                match provider_name.as_str() {
                                    "genius" if !genius_key.is_empty() => {
                                        let provider = GeniusLyricsProvider::new(genius_key);
                                        if let Ok(synced) = provider.fetch_synced(&artist, &title).await {
                                            let _ = tx.send(AppEvent::SyncedLyricsLoaded(synced));
                                        } else if let Ok(lyrics) = provider.fetch(&artist, &title).await {
                                            let _ = tx.send(AppEvent::LyricsLoaded(lyrics));
                                        } else {
                                            let _ = tx.send(AppEvent::LyricsFailed);
                                        }
                                    }
                                    "lrclib" => {
                                        let provider = LrcLibProvider::new();
                                        if let Ok(synced) = provider.fetch_synced(&artist, &title).await {
                                            let _ = tx.send(AppEvent::SyncedLyricsLoaded(synced));
                                        } else if let Ok(lyrics) = provider.fetch(&artist, &title).await {
                                            let _ = tx.send(AppEvent::LyricsLoaded(lyrics));
                                        } else {
                                            let _ = tx.send(AppEvent::LyricsFailed);
                                        }
                                    }
                                    _ => {
                                        let provider = LyricsOvhProvider::new();
                                        if let Ok(synced) = provider.fetch_synced(&artist, &title).await {
                                            let _ = tx.send(AppEvent::SyncedLyricsLoaded(synced));
                                        } else if let Ok(lyrics) = provider.fetch(&artist, &title).await {
                                            let _ = tx.send(AppEvent::LyricsLoaded(lyrics));
                                        } else {
                                            let _ = tx.send(AppEvent::LyricsFailed);
                                        }
                                    }
                                }
                            });

                            // Fetch art
                            if let Some(url) = art_url {
                                if let Some(cached) = state.art_cache.get(&album_id) {
                                    let _ = event_tx.send(AppEvent::ArtReady(album_id.clone(), cached.clone()));
                                } else {
                                    let tx = event_tx.clone();
                                    let client = reqwest::Client::new();
                                    tokio::spawn(async move {
                                        match crate::api::art::fetch_and_process_art(&url, &client, 40, 15, true).await {
                                            Ok(payload) => { let _ = tx.send(AppEvent::ArtReady(album_id, payload)); }
                                            Err(_) => {}
                                        }
                                    });
                                }
                            }
                        }
                    }
                }
                AppEvent::LyricsLoaded(lyrics) => {
                    state.lyrics = Some(lyrics);
                    state.synced_lyrics = Vec::new();
                    state.lyrics_loading = false;
                    state.lyrics_scroll_offset = 0;
                }
                AppEvent::SyncedLyricsLoaded(synced) => {
                    state.synced_lyrics = synced;
                    state.lyrics = None;
                    state.lyrics_loading = false;
                    state.lyrics_scroll_offset = 0;
                }
                AppEvent::LyricsFailed => {
                    state.lyrics = Some("Lyrics not found".to_string());
                    state.lyrics_loading = false;
                }
                AppEvent::ArtReady(album_id, payload) => {
                    match &payload {
                        crate::events::ArtPayload::Ascii(ascii) => {
                            state.album_art_ascii = Some(ascii.clone());
                            state.album_art_sixel = None;
                        }
                        crate::events::ArtPayload::Sixel(sixel) => {
                            state.album_art_sixel = Some(sixel.clone());
                            state.album_art_ascii = None;
                        }
                    }
                    state.art_cache.put(album_id, payload);
                }
                AppEvent::PlaylistsLoaded(playlists) => {
                    state.playlists = playlists;
                }
                AppEvent::PlaylistTracksLoaded(uri, tracks) => {
                    state.playlist_tracks = tracks;
                    state.current_playlist_uri = Some(uri);
                    state.track_index = 0;
                }
                AppEvent::SearchResults(results) => {
                    state.search_results = Some(results);
                    state.search_track_index = 0;
                    state.search_album_index = 0;
                    state.search_artist_index = 0;
                }
                AppEvent::DevicesLoaded(devices) => {
                    state.devices = devices;
                    state.device_index = 0;
                }
                AppEvent::QueueLoaded(queue) => {
                    state.queue = queue;
                    state.queue_index = 0;
                }
                AppEvent::ApiError(e) => {
                    state.last_error = Some(e);
                }
                AppEvent::Key(key) => {
                    let kb = &state.config.keybindings;
                    use crate::config::keymap::matches_key;
                    
                    match state.input_mode {
                        InputMode::Normal => {
                            if matches_key(key, &kb.help) {
                                state.show_help = !state.show_help;
                            } else if key.code == KeyCode::Esc {
                                if state.show_help {
                                    state.show_help = false;
                                }
                            } else if matches_key(key, &kb.quit) {
                                break;
                            } else if matches_key(key, &kb.search) {
                                state.active_pane = ui::ActivePane::Search;
                                state.input_mode = InputMode::Insert;
                            } else if matches_key(key, &kb.devices) {
                                state.active_pane = ui::ActivePane::Devices;
                                let client = spotify.clone();
                                tokio::spawn(async move { client.get_devices().await; });
                            } else if matches_key(key, &kb.queue) {
                                state.active_pane = ui::ActivePane::Queue;
                                let client = spotify.clone();
                                tokio::spawn(async move { client.get_queue().await; });
                            } else if matches_key(key, &kb.command) {
                                state.input_mode = InputMode::Command;
                                state.command_input = ":".to_string();
                            } else if matches_key(key, &kb.next_pane) {
                                state.active_pane = state.active_pane.next();
                            } else if matches_key(key, &kb.prev_pane) {
                                state.active_pane = state.active_pane.prev();
                            } else if matches_key(key, &kb.play_pause) {
                                let client = spotify.clone();
                                tokio::spawn(async move { client.play_pause().await; });
                            } else if matches_key(key, &kb.next_track) {
                                let client = spotify.clone();
                                tokio::spawn(async move { client.next_track().await; });
                            } else if matches_key(key, &kb.prev_track) {
                                let client = spotify.clone();
                                tokio::spawn(async move { client.previous_track().await; });
                            } else if matches_key(key, &kb.seek_forward) {
                                if let Some(playback) = &state.playback {
                                    let client = spotify.clone();
                                    let seek_ms = state.config.playback.seek_step_secs * 1000;
                                    let pos = playback.progress.unwrap_or_else(|| chrono::Duration::zero()).num_milliseconds() as u32 + seek_ms;
                                    tokio::spawn(async move { client.seek(pos).await; });
                                }
                            } else if matches_key(key, &kb.seek_backward) {
                                if let Some(playback) = &state.playback {
                                    let client = spotify.clone();
                                    let seek_ms = state.config.playback.seek_step_secs * 1000;
                                    let pos = (playback.progress.unwrap_or_else(|| chrono::Duration::zero()).num_milliseconds() as i32 - seek_ms as i32).max(0) as u32;
                                    tokio::spawn(async move { client.seek(pos).await; });
                                }
                            } else if matches_key(key, &kb.volume_up) {
                                let client = spotify.clone();
                                if state.active_pane == ui::ActivePane::Devices && !state.devices.is_empty() {
                                    let device = &state.devices[state.device_index];
                                    let step = state.config.playback.volume_step;
                                    let vol = (device.volume_percent.unwrap_or(50) as i32 + step as i32).min(100) as u8;
                                    let device_id = device.id.clone();
                                    tokio::spawn(async move { client.volume(vol, device_id.as_deref()).await; });
                                } else if let Some(playback) = &state.playback {
                                    let step = state.config.playback.volume_step;
                                    let vol = (playback.device.volume_percent.unwrap_or(50) as i32 + step as i32).min(100) as u8;
                                    tokio::spawn(async move { client.volume(vol, None).await; });
                                }
                            } else if matches_key(key, &kb.volume_down) {
                                let client = spotify.clone();
                                if state.active_pane == ui::ActivePane::Devices && !state.devices.is_empty() {
                                    let device = &state.devices[state.device_index];
                                    let step = state.config.playback.volume_step;
                                    let vol = (device.volume_percent.unwrap_or(50) as i32 - step as i32).max(0) as u8;
                                    let device_id = device.id.clone();
                                    tokio::spawn(async move { client.volume(vol, device_id.as_deref()).await; });
                                } else if let Some(playback) = &state.playback {
                                    let step = state.config.playback.volume_step;
                                    let vol = (playback.device.volume_percent.unwrap_or(50) as i32 - step as i32).max(0) as u8;
                                    tokio::spawn(async move { client.volume(vol, None).await; });
                                }
                            } else if matches_key(key, &kb.play_radio) {
                                if let Some(playback) = &state.playback {
                                    if let Some(rspotify::model::PlayableItem::Track(t)) = &playback.item {
                                        if let Some(id) = &t.id {
                                            let client = spotify.clone();
                                            let id_str = id.to_string();
                                            tokio::spawn(async move { client.play_radio(&id_str).await; });
                                        }
                                    }
                                }
                            } else if matches_key(key, &kb.move_down) || key.code == KeyCode::Down {
                                match state.active_pane {
                                    ui::ActivePane::Lyrics => {
                                        state.lyrics_scroll_offset = state.lyrics_scroll_offset.saturating_add(1);
                                    }
                                    ui::ActivePane::Playlists => {
                                        if !state.playlists.is_empty() {
                                            state.playlist_index = (state.playlist_index + 1) % state.playlists.len();
                                        }
                                    }
                                    ui::ActivePane::TrackList => {
                                        if !state.playlist_tracks.is_empty() {
                                            state.track_index = (state.track_index + 1) % state.playlist_tracks.len();
                                        }
                                    }
                                    ui::ActivePane::Search => {
                                        if let Some(results) = &state.search_results {
                                            match state.search_active_tab {
                                                0 => if let Some(t) = &results.tracks { if !t.items.is_empty() { state.search_track_index = (state.search_track_index + 1) % t.items.len(); } },
                                                1 => if let Some(a) = &results.albums { if !a.items.is_empty() { state.search_album_index = (state.search_album_index + 1) % a.items.len(); } },
                                                2 => if let Some(ar) = &results.artists { if !ar.items.is_empty() { state.search_artist_index = (state.search_artist_index + 1) % ar.items.len(); } },
                                                _ => {}
                                            }
                                        }
                                    }
                                    ui::ActivePane::Devices => {
                                        if !state.devices.is_empty() {
                                            state.device_index = (state.device_index + 1) % state.devices.len();
                                        }
                                    }
                                    ui::ActivePane::Queue => {
                                        if !state.queue.is_empty() {
                                            state.queue_index = (state.queue_index + 1) % state.queue.len();
                                        }
                                    }
                                }
                            } else if matches_key(key, &kb.move_up) || key.code == KeyCode::Up {
                                match state.active_pane {
                                    ui::ActivePane::Lyrics => {
                                        state.lyrics_scroll_offset = state.lyrics_scroll_offset.saturating_sub(1);
                                    }
                                    ui::ActivePane::Playlists => {
                                        if !state.playlists.is_empty() {
                                            state.playlist_index = (state.playlist_index + state.playlists.len() - 1) % state.playlists.len();
                                        }
                                    }
                                    ui::ActivePane::TrackList => {
                                        if !state.playlist_tracks.is_empty() {
                                            state.track_index = (state.track_index + state.playlist_tracks.len() - 1) % state.playlist_tracks.len();
                                        }
                                    }
                                    ui::ActivePane::Search => {
                                        if let Some(results) = &state.search_results {
                                            match state.search_active_tab {
                                                0 => if let Some(t) = &results.tracks { if !t.items.is_empty() { state.search_track_index = (state.search_track_index + t.items.len() - 1) % t.items.len(); } },
                                                1 => if let Some(a) = &results.albums { if !a.items.is_empty() { state.search_album_index = (state.search_album_index + a.items.len() - 1) % a.items.len(); } },
                                                2 => if let Some(ar) = &results.artists { if !ar.items.is_empty() { state.search_artist_index = (state.search_artist_index + ar.items.len() - 1) % ar.items.len(); } },
                                                _ => {}
                                            }
                                        }
                                    }
                                    ui::ActivePane::Devices => {
                                        if !state.devices.is_empty() {
                                            state.device_index = (state.device_index + state.devices.len() - 1) % state.devices.len();
                                        }
                                    }
                                    ui::ActivePane::Queue => {
                                        if !state.queue.is_empty() {
                                            state.queue_index = (state.queue_index + state.queue.len() - 1) % state.queue.len();
                                        }
                                    }
                                }
                            } else if matches_key(key, &kb.select) {
                                if state.active_pane == ui::ActivePane::Playlists && !state.playlists.is_empty() {
                                    if let Some(playlist) = state.playlists.get(state.playlist_index) {
                                        let client = spotify.clone();
                                        let id = playlist.id.to_string();
                                        tokio::spawn(async move { client.get_playlist_tracks(&id).await; });
                                        state.active_pane = ui::ActivePane::TrackList;
                                    }
                                } else if state.active_pane == ui::ActivePane::TrackList && !state.playlist_tracks.is_empty() {
                                    if let Some(item) = state.playlist_tracks.get(state.track_index) {
                                        if let Some(rspotify::model::PlayableItem::Track(track)) = &item.track {
                                            if let Some(track_id) = &track.id {
                                                let client = spotify.clone();
                                                let id_str = track_id.to_string();
                                                let context = state.current_playlist_uri.clone();
                                                tokio::spawn(async move { client.play_track(&id_str, context.as_deref()).await; });
                                            }
                                        }
                                    }
                                } else if state.active_pane == ui::ActivePane::Search && !state.search_query.is_empty() {
                                    if let Some(results) = &state.search_results {
                                        match state.search_active_tab {
                                            0 => {
                                                if let Some(tracks) = &results.tracks {
                                                    if let Some(track) = tracks.items.get(state.search_track_index) {
                                                        if let Some(track_id) = &track.id {
                                                            let client = spotify.clone();
                                                            let id_str = track_id.to_string();
                                                            tokio::spawn(async move { client.play_track(&id_str, None).await; });
                                                        }
                                                    }
                                                }
                                            }
                                            _ => {}
                                        }
                                    }
                                } else if state.active_pane == ui::ActivePane::Devices && !state.devices.is_empty() {
                                    if let Some(device) = state.devices.get(state.device_index) {
                                        if let Some(device_id) = &device.id {
                                            let client = spotify.clone();
                                            let id = device_id.clone();
                                            tokio::spawn(async move { client.transfer_playback(&id).await; });
                                        }
                                    }
                                }
                            } else if key.code == KeyCode::Char('a') {
                                if state.active_pane == ui::ActivePane::TrackList && !state.playlist_tracks.is_empty() {
                                    if let Some(item) = state.playlist_tracks.get(state.track_index) {
                                        if let Some(rspotify::model::PlayableItem::Track(track)) = &item.track {
                                            if let Some(track_id) = &track.id {
                                                let client = spotify.clone();
                                                let id_str = track_id.to_string();
                                                tokio::spawn(async move { client.add_to_queue(&id_str).await; });
                                            }
                                        }
                                    }
                                } else if state.active_pane == ui::ActivePane::Search && state.search_active_tab == 0 {
                                    if let Some(results) = &state.search_results {
                                        if let Some(tracks) = &results.tracks {
                                            if let Some(track) = tracks.items.get(state.search_track_index) {
                                                if let Some(track_id) = &track.id {
                                                    let client = spotify.clone();
                                                    let id_str = track_id.to_string();
                                                    tokio::spawn(async move { client.add_to_queue(&id_str).await; });
                                                }
                                            }
                                        }
                                    }
                                }
                            }
                        }
                        InputMode::Command => {
                            match key.code {
                                KeyCode::Esc => {
                                    state.input_mode = InputMode::Normal;
                                }
                                KeyCode::Enter => {
                                    let cmd = state.command_input.clone();
                                    state.input_mode = InputMode::Normal;
                                    state.command_input.clear();
                                    
                                    if cmd == ":q" {
                                        state.is_running = false;
                                    } else if cmd == ":logout" {
                                        let _ = crate::api::auth::clear_token();
                                        state.is_running = false;
                                    } else if cmd.starts_with(":theme ") {
                                        let theme_name = cmd.trim_start_matches(":theme ");
                                        match theme_name {
                                            "catppuccin-mocha" => state.theme = crate::config::theme::Theme::catppuccin_mocha(),
                                            "dark" | "default" => state.theme = crate::config::theme::Theme::dark(),
                                            _ => state.last_error = Some(format!("Unknown theme: {}", theme_name)),
                                        }
                                    } else if cmd.starts_with(":playlist-create ") {
                                        let name = cmd.trim_start_matches(":playlist-create ");
                                        let client = spotify.clone();
                                        let name_owned = name.to_string();
                                        tokio::spawn(async move { client.create_playlist(&name_owned, false).await; });
                                    } else if cmd == ":playlist-delete" {
                                        if state.active_pane == ui::ActivePane::Playlists && !state.playlists.is_empty() {
                                            if let Some(playlist) = state.playlists.get(state.playlist_index) {
                                                let client = spotify.clone();
                                                let id = playlist.id.to_string();
                                                tokio::spawn(async move { client.delete_playlist(&id).await; });
                                            }
                                        }
                                    } else if cmd.starts_with(":playlist-reorder ") {
                                        let args: Vec<&str> = cmd.trim_start_matches(":playlist-reorder ").split_whitespace().collect();
                                        if args.len() == 2 {
                                            if let (Ok(start), Ok(before)) = (args[0].parse::<u32>(), args[1].parse::<u32>()) {
                                                if state.active_pane == ui::ActivePane::Playlists && !state.playlists.is_empty() {
                                                    if let Some(playlist) = state.playlists.get(state.playlist_index) {
                                                        let client = spotify.clone();
                                                        let id = playlist.id.to_string();
                                                        tokio::spawn(async move { client.reorder_playlist_tracks(&id, start, before).await; });
                                                    }
                                                }
                                            }
                                        }
                                    } else if cmd == ":device" {
                                        state.active_pane = ui::ActivePane::Devices;
                                        let client = spotify.clone();
                                        tokio::spawn(async move { client.get_devices().await; });
                                    } else if cmd == ":help" {
                                        state.show_help = true;
                                    }
                                }
                                KeyCode::Char(c) => {
                                    state.command_input.push(c);
                                }
                                KeyCode::Backspace => {
                                    if state.command_input.len() > 1 {
                                        state.command_input.pop();
                                    } else {
                                        state.input_mode = InputMode::Normal;
                                    }
                                }
                                _ => {}
                            }
                        }
                        InputMode::Insert => {
                            match key.code {
                                KeyCode::Esc => {
                                    state.input_mode = InputMode::Normal;
                                }
                                KeyCode::Enter => {
                                    if state.active_pane == ui::ActivePane::Search && !state.search_query.is_empty() {
                                        let client = spotify.clone();
                                        let query = state.search_query.clone();
                                        tokio::spawn(async move { client.search(&query).await; });
                                        state.input_mode = InputMode::Normal;
                                    } else {
                                        state.input_mode = InputMode::Normal;
                                    }
                                }
                                KeyCode::Char(c) => {
                                    if state.active_pane == ui::ActivePane::Search {
                                        state.search_query.push(c);
                                    }
                                }
                                KeyCode::Backspace => {
                                    if state.active_pane == ui::ActivePane::Search {
                                        state.search_query.pop();
                                    }
                                }
                                _ => {}
                            }
                        }
                    }
                }
                AppEvent::Mouse(mouse) => {
                    match mouse.kind {
                        MouseEventKind::Down(crossterm::event::MouseButton::Left) => {
                            let x = mouse.column;
                            let y = mouse.row;
                            
                            if state.playlist_area.intersects(Rect::new(x, y, 1, 1)) {
                                state.active_pane = ui::ActivePane::Playlists;
                            } else if state.track_list_area.intersects(Rect::new(x, y, 1, 1)) {
                                state.active_pane = ui::ActivePane::TrackList;
                            } else if state.lyrics_area.intersects(Rect::new(x, y, 1, 1)) {
                                state.active_pane = ui::ActivePane::Lyrics;
                            } else if state.search_area.intersects(Rect::new(x, y, 1, 1)) {
                                state.active_pane = ui::ActivePane::Search;
                                state.input_mode = InputMode::Insert;
                            } else if state.devices_area.intersects(Rect::new(x, y, 1, 1)) {
                                state.active_pane = ui::ActivePane::Devices;
                            } else if state.queue_area.intersects(Rect::new(x, y, 1, 1)) {
                                state.active_pane = ui::ActivePane::Queue;
                            }
                        }
                        MouseEventKind::ScrollDown => {
                            match state.active_pane {
                                ui::ActivePane::Lyrics => state.lyrics_scroll_offset = state.lyrics_scroll_offset.saturating_add(1),
                                ui::ActivePane::Playlists => if !state.playlists.is_empty() { state.playlist_index = (state.playlist_index + 1) % state.playlists.len() },
                                ui::ActivePane::TrackList => if !state.playlist_tracks.is_empty() { state.track_index = (state.track_index + 1) % state.playlist_tracks.len() },
                                _ => {}
                            }
                        }
                        MouseEventKind::ScrollUp => {
                            match state.active_pane {
                                ui::ActivePane::Lyrics => state.lyrics_scroll_offset = state.lyrics_scroll_offset.saturating_sub(1),
                                ui::ActivePane::Playlists => if !state.playlists.is_empty() { state.playlist_index = (state.playlist_index + state.playlists.len() - 1) % state.playlists.len() },
                                ui::ActivePane::TrackList => if !state.playlist_tracks.is_empty() { state.track_index = (state.track_index + state.playlist_tracks.len() - 1) % state.playlist_tracks.len() },
                                _ => {}
                            }
                        }
                        _ => {}
                    }
                }
                AppEvent::Resize(_, _) => {}
                AppEvent::Tick => {
                    if let Some(playback) = &mut state.playback {
                        if playback.is_playing {
                            if let Some(progress) = &mut playback.progress {
                                *progress = *progress + chrono::Duration::seconds(1);

                                // Auto-scroll synced lyrics
                                if !state.synced_lyrics.is_empty() {
                                    let current_ms = progress.num_milliseconds() as u32;
                                    if let Some(idx) = state.synced_lyrics.iter().position(|l| l.timestamp_ms > current_ms) {
                                        if idx > 0 {
                                            state.lyrics_scroll_offset = (idx - 1) as u16;
                                        }
                                    } else if !state.synced_lyrics.is_empty() {
                                        state.lyrics_scroll_offset = (state.synced_lyrics.len() - 1) as u16;
                                    }
                                }

                                // Crossfade / Gapless pre-fetch simulation
                                if let Some(item) = &playback.item {
                                    let duration = match item {
                                        rspotify::model::PlayableItem::Track(t) => t.duration,
                                        rspotify::model::PlayableItem::Episode(e) => e.duration,
                                    };
                                    let remaining = duration - *progress;
                                    if remaining.num_seconds() <= 5 && remaining.num_seconds() > 4 {
                                        // Pre-fetch next track or update playback soon
                                        let client = spotify.clone();
                                        tokio::spawn(async move { client.update_playback().await; });
                                    }
                                }
                            }
                        }
                    }
                }
                AppEvent::Poll => {
                    let client = spotify.clone();
                    tokio::spawn(async move { client.update_playback().await; });
                }
                AppEvent::Quit => break,
            }
        }
        
        if !state.is_running {
            break;
        }
    }

    // Restore terminal
    disable_raw_mode()?;
    io::stdout().execute(event::DisableMouseCapture)?;
    io::stdout().execute(LeaveAlternateScreen)?;
    
    Ok(())
}
