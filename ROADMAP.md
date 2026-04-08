# Development Roadmap — Ferrum (Spotify TUI)

This document outlines the development phases for **Ferrum**, derived from the Product Requirements Document (PRD) v1.0.0.

---

## Phase 1: Core Architecture & Authentication (The Foundation)
**Goal:** Establish the async event loop and secure connection to Spotify.

- [x] **Infrastructure Setup**
    - [x] Initialize project with Ratatui, Tokio, and rspotify.
    - [x] Implement the centralized `AppState` and MPSC Event Bus (FR-STATE-01, FR-STATE-02).
- [x] **Authentication (G1)**
    - [x] Implement OAuth 2.0 PKCE flow with local callback server (FR-AUTH-01).
    - [x] Secure token storage using OS keychain via `keyring` (FR-AUTH-02).
    - [x] Background token refresh logic.
- [x] **TUI Shell (G8, G9)**
    - [x] Multi-pane responsiv  e layout with focus routing (FR-UI-01, FR-UI-02, FR-UI-03).
    - [x] Vim-style modal input system: Normal, Insert, and Command modes (FR-KEY-01).

---

## Phase 2: Playback & Library Navigation
**Goal:** Enable music control and browsing of personal playlists.

- [x] **Transport Controls (G3)**
    - [x] Play/pause, skip, seek, and volume adjustment (FR-PLAY-01).
    - [x] Now Playing pane with metadata and progress bar (FR-PLAY-02).
    - [x] Periodic polling and local interpolation for progress sync (FR-PLAY-03).
- [x] **Library Access (G4)**
    - [x] Fetch and display user playlists (FR-PLAYLIST-01).
    - [x] Detailed track view for playlists and "Liked Songs" (FR-PLAYLIST-02, FR-PLAYLIST-04).
    - [x] Contextual playback from playlist tracks (FR-PLAYLIST-03).

---

## Phase 3: Search, Queue & Device Management
**Goal:** Full control over what plays next and where it plays.

- [x] **Search Engine (G2)**
    - [x] Modal search input with tabbed results for Tracks, Albums, and Artists (FR-SEARCH-01, FR-SEARCH-02).
    - [x] Navigation and immediate playback from search results (FR-SEARCH-03, FR-SEARCH-04).
- [x] **Queue Management (G5)**
    - [x] Display current playback queue (FR-QUEUE-01).
    - [x] "Add to Queue" functionality from any track list (FR-QUEUE-02).
- [x] **Device Transfer (G11)**
    - [x] Device picker modal to hand off playback between Spotify clients (FR-PLAY-04).

---

## Phase 4: Enhanced Visuals & Polish
**Goal:** Rich media integration and UI customization.

- [x] **Album Art (G7, G12)**
    - [x] ASCII art rendering with brightness mapping (FR-ART-02).
    - [x] SIXEL support for high-resolution art in compatible terminals (FR-ART-03). (Note: ASCII fallback used)
    - [x] LRU caching for downloaded and converted art (FR-ART-04).
- [x] **Lyrics Integration (G6)**
    - [x] Async fetching from `lyrics.ovh` (FR-LYRICS-01).
    - [x] Scrollable lyrics pane with auto-wrap (FR-LYRICS-02, FR-LYRICS-03).
- [x] **Theming (FR-UI-05)**
    - [x] TOML-based theme configuration.
    - [x] Inclusion of default dark and `catppuccin-mocha` themes.

---

## Phase 5: v1.0 Stabilization & Documentation
**Goal:** Finalizing for public release and ensuring reliability.

- [x] **Help & Commands**
    - [x] Help overlay (`?`) with full keybinding reference (FR-UI-06).
    - [x] Command mode (`:`) for management tasks (FR-KEY-03).
- [x] **Reliability & Performance**
    - [x] Error handling with `color-eyre` (7.2).
    - [x] Graceful degradation for small terminal windows (FR-UI-02).
    - [x] Verification of success metrics (Section 14).
- [x] **Documentation**
    - [x] Generation of default `config.toml` (7.6).
    - [x] README with setup instructions and keybinding guide.

---

## Future Roadmap (v1.1+)

### v1.1 — Quality of Life
- Genius lyrics integration (authenticated).
- Keybinding remapping via `config.toml`.
- Mouse support for navigation.
- Playlist management (create/delete/reorder).

### v1.2 — Advanced Playback
- [x] Full Spotify Connect remote management.
- [x] Crossfade and gapless playback simulation.
- [x] Radio/Recommendations seeding.
- [x] Synchronized lyrics (scrolling with playback).

### v2.0 — Multi-Provider (The "Ferrum" Vision)
- **Apple Music** and **Tidal** integration via `MusicProvider` trait.
- **Local Files** support via `symphonia` (optional audio decoding).
- Last.fm scrobbling across all providers.

### v3.0 — Ecosystem
- Lua/WASM plugin API for custom extensions.
- Shared listening sessions.
- Ferrum Daemon for headless background control.
