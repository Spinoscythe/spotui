use rspotify::model::{CurrentPlaybackContext, SimplifiedPlaylist, PlaylistItem, SearchMultipleResult, Device};
use crossterm::event::{KeyEvent, MouseEvent};

#[derive(Debug, Clone)]
pub struct SyncedLyricsLine {
    pub timestamp_ms: u32,
    pub text: String,
}

pub enum AppEvent {
    // Input
    Key(KeyEvent),
    Mouse(MouseEvent),
    Resize(u16, u16),

    // Timers
    Tick,          // 1s — progress bar update
    Poll,          // 5s — full state re-sync from Spotify

    // API Results
    PlaybackUpdated(Box<CurrentPlaybackContext>),
    PlaylistsLoaded(Vec<SimplifiedPlaylist>),
    PlaylistTracksLoaded(String, Vec<PlaylistItem>), // String is playlist URI
    SearchResults(SearchMultipleResult),
    DevicesLoaded(Vec<Device>),
    QueueLoaded(Vec<rspotify::model::FullTrack>),
    ApiError(String),

    // Lyrics
    LyricsLoaded(String),
    SyncedLyricsLoaded(Vec<SyncedLyricsLine>),
    LyricsFailed,

    // Art
    ArtReady(String, ArtPayload), // Changed from AlbumId to String for simpler lifetime management

    // Commands
    Quit,
}

#[derive(Debug, Clone)]
pub enum ArtPayload {
    Ascii(String),
    Sixel(Vec<u8>),
}
