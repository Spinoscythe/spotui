use crate::events::{AppEvent, ArtPayload, SyncedLyricsLine};
use tokio::sync::mpsc;
use crate::ui::ActivePane;
use crate::config::Config;
use crate::config::theme::Theme;
use lru::LruCache;
use std::num::NonZeroUsize;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum InputMode {
    Normal,
    Insert,
    Command,
}

use ratatui::layout::Rect;

pub struct AppState {
    pub input_mode: InputMode,
    pub active_pane: ActivePane,
    pub is_running: bool,
    pub event_tx: mpsc::UnboundedSender<AppEvent>,
    
    // Areas for mouse support
    pub playlist_area: Rect,
    pub track_list_area: Rect,
    pub lyrics_area: Rect,
    pub search_area: Rect,
    pub devices_area: Rect,
    pub queue_area: Rect,
    pub now_playing_area: Rect,
    pub playback: Option<Box<rspotify::model::CurrentPlaybackContext>>,
    pub devices: Vec<rspotify::model::Device>,
    pub queue: Vec<rspotify::model::FullTrack>,
    
    // Library state
    pub playlists: Vec<rspotify::model::SimplifiedPlaylist>,
    pub playlist_tracks: Vec<rspotify::model::PlaylistItem>,
    pub current_playlist_uri: Option<String>,
    
    // Search state
    pub search_query: String,
    pub search_results: Option<rspotify::model::SearchMultipleResult>,
    pub search_active_tab: usize,
    pub search_track_index: usize,
    pub search_album_index: usize,
    pub search_artist_index: usize,
    
    // UI selection state
    pub playlist_index: usize,
    pub track_index: usize,
    pub device_index: usize,
    pub queue_index: usize,
    
    // UI state
    pub scroll_offset: u16,
    pub last_error: Option<String>,
    pub command_input: String,
    pub show_help: bool,
    
    // Phase 4 additions
    pub theme: Theme,
    pub config: Config,
    pub lyrics: Option<String>,
    pub synced_lyrics: Vec<SyncedLyricsLine>,
    pub lyrics_loading: bool,
    pub lyrics_scroll_offset: u16,
    pub album_art_ascii: Option<String>,
    pub album_art_sixel: Option<Vec<u8>>,
    pub art_cache: LruCache<String, ArtPayload>,
}

impl AppState {
    pub fn new(event_tx: mpsc::UnboundedSender<AppEvent>, config: Config) -> Self {
        let theme = match config.ui.theme.as_str() {
            "catppuccin-mocha" => Theme::catppuccin_mocha(),
            _ => Theme::default(),
        };

        Self {
            input_mode: InputMode::Normal,
            active_pane: ActivePane::Playlists,
            is_running: true,
            event_tx,
            playback: None,
            devices: Vec::new(),
            queue: Vec::new(),
            playlists: Vec::new(),
            playlist_tracks: Vec::new(),
            current_playlist_uri: None,
            search_query: String::new(),
            search_results: None,
            search_active_tab: 0,
            search_track_index: 0,
            search_album_index: 0,
            search_artist_index: 0,
            playlist_index: 0,
            track_index: 0,
            device_index: 0,
            queue_index: 0,
            scroll_offset: 0,
            playlist_area: Rect::default(),
            track_list_area: Rect::default(),
            lyrics_area: Rect::default(),
            search_area: Rect::default(),
            devices_area: Rect::default(),
            queue_area: Rect::default(),
            now_playing_area: Rect::default(),
            last_error: None,
            command_input: String::new(),
            show_help: false,
            theme,
            config,
            lyrics: None,
            synced_lyrics: Vec::new(),
            lyrics_loading: false,
            lyrics_scroll_offset: 0,
            album_art_ascii: None,
            album_art_sixel: None,
            art_cache: LruCache::new(NonZeroUsize::new(20).unwrap()),
        }
    }
}
