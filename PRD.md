# Product Requirements Document
# Ferrum — Terminal Spotify Client (TUI)

**Version:** 1.0.0  
**Status:** Draft  
**Author:** Senior Product & Systems Design  
**Last Updated:** 2026-04-08  
**Stack:** Rust · Ratatui · rspotify · Tokio

---

## Table of Contents

1. [Executive Summary](#1-executive-summary)
2. [Problem Statement](#2-problem-statement)
3. [Goals and Non-Goals](#3-goals-and-non-goals)
4. [User Personas](#4-user-personas)
5. [User Stories](#5-user-stories)
6. [Functional Requirements](#6-functional-requirements)
7. [Non-Functional Requirements](#7-non-functional-requirements)
8. [System Architecture Overview](#8-system-architecture-overview)
9. [Component Breakdown](#9-component-breakdown)
10. [Data Flow Diagrams](#10-data-flow-diagrams)
11. [Key Technical Decisions](#11-key-technical-decisions)
12. [Risks and Mitigations](#12-risks-and-mitigations)
13. [Future Enhancements](#13-future-enhancements)
14. [Success Metrics](#14-success-metrics)

---

## 1. Executive Summary

**Ferrum** is a keyboard-driven, terminal-native music streaming client for Spotify, built in Rust using the Ratatui TUI framework. It targets developers, Linux/macOS power users, and terminal enthusiasts who spend most of their workflow in the terminal and prefer not to context-switch into a browser or Electron-based GUI for music control.

The application exposes the full Spotify Web API surface through a fast, composable, modal interface — drawing inspiration from Vim's modal editing model and tools like `lazygit` and `ncspot`. It renders album art as ASCII or SIXEL graphics, fetches and displays lyrics inline, and manages multi-device playback handoff, all without leaving the terminal.

Ferrum is written entirely in Rust for performance, correctness, and cross-platform portability. It uses an event-driven, async-first architecture powered by Tokio, with a centralized state machine that makes the system easy to extend for future streaming providers.

---

## 2. Problem Statement

### 2.1 Context

Spotify's official client is a multi-hundred-megabyte Electron application. Even its web client requires a browser context. For users who live in the terminal — particularly those running tiling window managers (i3, Hyprland, Sway, Aerospace) on Linux or macOS — context-switching to a GUI music client creates cognitive friction and breaks keyboard-centric workflows.

Existing terminal Spotify clients (e.g., `ncspot`, `spotify-tui`) address this need partially, but they suffer from one or more of the following:

- **Abandoned or slow maintenance** — `spotify-tui` is effectively unmaintained as of 2024.
- **Feature gaps** — Lyrics, SIXEL album art, and queue management are rarely first-class features.
- **Architectural limitations** — Some use synchronous rendering loops that cause flicker; others lack proper modal input systems.
- **Poor extensibility** — Hard-coded Spotify assumptions make multi-provider support difficult.

### 2.2 The Gap

There is no actively maintained, feature-complete, Rust-native terminal Spotify client that:

- Provides a Vim-modal keybinding system.
- Renders album art via both ASCII and SIXEL.
- Integrates lyrics inline with playback.
- Follows a clean, async, event-driven architecture that can support additional streaming providers.

### 2.3 Opportunity

The terminal music client market is a niche but loyal one. A high-quality, well-maintained Rust implementation would attract:

- Open-source contributors and Rust learners seeking a real-world async TUI project.
- Power users who currently use `ncspot` or raw `spotifyd` + `playerctl` scripts.
- Developers who want to customize and extend their music client through configuration.

---

## 3. Goals and Non-Goals

### 3.1 Goals (v1.0)

| # | Goal | Priority |
|---|------|----------|
| G1 | Full Spotify OAuth 2.0 PKCE authentication with token persistence | Must Have |
| G2 | Search tracks, albums, artists with navigable results | Must Have |
| G3 | Full playback control: play, pause, skip, previous, seek, volume | Must Have |
| G4 | Browse and play from user playlists | Must Have |
| G5 | View and navigate the current playback queue | Must Have |
| G6 | Fetch and display lyrics (static) via third-party API | Must Have |
| G7 | Render album art as ASCII art | Must Have |
| G8 | Multi-pane responsive TUI via Ratatui | Must Have |
| G9 | Vim-style modal keybindings (Normal / Insert / Command) | Must Have |
| G10 | Centralized async state management with event bus | Must Have |
| G11 | Transfer playback between Spotify-connected devices | Should Have |
| G12 | SIXEL album art rendering for supported terminals | Should Have |
| G13 | Synchronized lyrics (if Spotify internal API becomes accessible) | Nice to Have |

### 3.2 Non-Goals (v1.0)

- **No audio decoding or local playback.** Ferrum is a controller. Actual audio output is delegated to Spotify clients (desktop, web, `spotifyd`).
- **No podcast support.** Podcasts are explicitly out of scope for v1.
- **No social/collaborative features** (shared playlists, friend activity).
- **No offline mode.** All features require active internet and a Spotify Premium account.
- **No plugin system.** Extensibility is architectural (trait-based provider abstraction), not user-facing.
- **No Apple Music or Tidal in v1.** The architecture supports this; implementation is deferred.

---

## 4. User Personas

### Persona A — "The Terminal Native" (Primary)

**Name:** Aarav, 28, Backend Engineer  
**OS:** Arch Linux, Hyprland window manager  
**Shell:** Zsh + Starship  
**Music behavior:** Keeps a Spotify window in a scratchpad; switches to it ~20 times/day  
**Pain point:** Context-switching to the GUI breaks his flow state. He already uses `lazygit`, `btop`, `nvim`, and `yazi` — he wants music to fit the same model.  
**Goals:** Navigate playlists and queue without leaving the terminal. Search quickly. Read lyrics while coding.  
**Technical comfort:** High. Happy to configure a TOML file, add API keys, compile from source.

---

### Persona B — "The Power User" (Primary)

**Name:** Elif, 34, DevOps Engineer  
**OS:** macOS 15, iTerm2 + tmux  
**Music behavior:** Uses Spotify Premium. Maintains 40+ personal playlists.  
**Pain point:** Spotify's Mac app is heavy and takes focus. She has iTerm2 set up with SIXEL support and wants album art inline.  
**Goals:** Browse her library, manage the queue, see lyrics, all inside tmux.  
**Technical comfort:** Medium-high. Prefers documented defaults; tolerates some configuration.

---

### Persona C — "The Open Source Contributor" (Secondary)

**Name:** Marco, 22, CS Student  
**OS:** Ubuntu 22.04, Alacritty  
**Music behavior:** Casual Spotify user. More interested in the codebase than the product.  
**Pain point:** Wants a real-world Rust async TUI project to contribute to and learn from.  
**Goals:** Understand the architecture, add a feature (e.g., a new keybinding or a color theme), submit a PR.  
**Technical comfort:** High. Comfortable reading Rust, learning Tokio.

---

### Persona D — "The Minimalist" (Secondary)

**Name:** Priya, 31, Security Researcher  
**OS:** Debian, foot terminal, Sway  
**Music behavior:** Uses Spotify from the web. Dislikes running unnecessary GUI processes.  
**Pain point:** Wants to control music with the minimum possible surface area. No Electron. No browser tab.  
**Goals:** Play/pause, skip, see what's playing, in as little screen real estate as possible.  
**Technical comfort:** High. Will use it as-is; minimal configuration effort.

---

## 5. User Stories

### Authentication

| ID | Story | Acceptance Criteria |
|----|-------|---------------------|
| US-01 | As a new user, I want to authenticate with Spotify so I can use the app | OAuth PKCE flow opens browser, redirects to localhost callback, token stored to disk |
| US-02 | As a returning user, I want my session to persist so I don't re-authenticate every launch | Stored token refreshed automatically; user never sees auth screen if token valid |
| US-03 | As a user, I want to log out so I can switch Spotify accounts | Keychain/file token cleared; app returns to auth screen |

### Search

| ID | Story | Acceptance Criteria |
|----|-------|---------------------|
| US-04 | As a user, I want to type a query and see tracks, albums, and artists | Results rendered in tabbed list within 500ms of query submission |
| US-05 | As a user, I want to navigate results with keyboard | j/k scrolls; Enter selects; Esc dismisses |
| US-06 | As a user, I want to play a track directly from search results | Enter on a track starts playback; now-playing pane updates |

### Playback

| ID | Story | Acceptance Criteria |
|----|-------|---------------------|
| US-07 | As a user, I want to play/pause with a single keypress | Spacebar toggles play/pause; UI reflects state within 200ms |
| US-08 | As a user, I want to skip to the next/previous track | `>` / `<` trigger skip; now-playing updates |
| US-09 | As a user, I want to see the current track, artist, album, and progress | Progress bar and metadata displayed in now-playing pane; updates every second |
| US-10 | As a user, I want to adjust volume | `+` / `-` adjusts volume in 5% increments |
| US-11 | As a user, I want to seek within a track | Left/right arrow keys seek ±10s |
| US-12 | As a user, I want to transfer playback to another device | Device picker modal shows available devices; Enter transfers |

### Playlists

| ID | Story | Acceptance Criteria |
|----|-------|---------------------|
| US-13 | As a user, I want to see all my playlists | Playlist pane lists all playlists, paginated |
| US-14 | As a user, I want to open a playlist and browse its tracks | Enter on playlist shows track list in adjacent pane |
| US-15 | As a user, I want to play a track from a playlist | Enter on a track in the playlist view starts playback |

### Queue

| ID | Story | Acceptance Criteria |
|----|-------|---------------------|
| US-16 | As a user, I want to see what's coming up in my queue | Queue pane lists upcoming tracks in order |
| US-17 | As a user, I want to add a track to the queue from search or playlist | `a` key on a selected track adds it to queue; confirmation shown |

### Lyrics

| ID | Story | Acceptance Criteria |
|----|-------|---------------------|
| US-18 | As a user, I want to see lyrics for the current track | Lyrics pane shows text fetched from lyrics API |
| US-19 | As a user, I want lyrics to load without blocking the UI | Lyrics fetched async; pane shows "Loading…" then populates |
| US-20 | As a user, I want to scroll lyrics manually | j/k scrolls the lyrics pane when it is focused |

### Album Art

| ID | Story | Acceptance Criteria |
|----|-------|---------------------|
| US-21 | As a user, I want to see album art in the now-playing pane | ASCII art rendered for every track; updates on track change |
| US-22 | As a user on a SIXEL-capable terminal, I want full-color art | SIXEL rendered if terminal advertises support; falls back to ASCII |

### UI and Navigation

| ID | Story | Acceptance Criteria |
|----|-------|---------------------|
| US-23 | As a user, I want to navigate panes with Tab / Shift-Tab | Focus cycles through panes; active pane has visible border highlight |
| US-24 | As a user, I want a help overlay showing all keybindings | `?` key shows modal help overlay |
| US-25 | As a user, I want the UI to work in small terminal windows | Layout degrades gracefully; minimum viable size 80×24 |

---

## 6. Functional Requirements

### 6.1 Authentication Module

**FR-AUTH-01** — OAuth 2.0 PKCE Flow  
The application MUST implement the Spotify Authorization Code with PKCE flow. On first launch (or after token expiry with no refresh token), the app SHALL:
1. Generate a code verifier and challenge.
2. Open the system browser to the Spotify authorization URL.
3. Start a local HTTP server on `127.0.0.1:8888` to capture the redirect callback.
4. Exchange the authorization code for an access token and refresh token.
5. Store both tokens encrypted (or base64-encoded) in the OS keychain or a config-directory file (`~/.config/ferrum/tokens.json`).

**FR-AUTH-02** — Token Refresh  
The application MUST silently refresh the access token using the stored refresh token before it expires (typically 1 hour). Refresh MUST occur in the background without interrupting playback or UI interaction.

**FR-AUTH-03** — Multi-Account Support (v1.1 target)  
The token store MUST be keyed by Spotify user ID to enable future multi-account switching.

**FR-AUTH-04** — Logout  
The application MUST provide a command (`:logout`) that clears all stored tokens and returns the app to the unauthenticated state.

---

### 6.2 Search Module

**FR-SEARCH-01** — Query Input  
Pressing `/` in Normal mode MUST transition the app to Search Insert mode, moving focus to a search input widget. Pressing `Esc` cancels and returns to Normal mode.

**FR-SEARCH-02** — Result Categories  
Search results MUST be displayed in three tabbed sections:
- **Tracks** (default tab)
- **Albums**
- **Artists**

Tab key cycles between result categories. Each category shows up to 20 results per fetch; `G` at the bottom of a list triggers pagination (load next 20).

**FR-SEARCH-03** — Result Rendering  
Each result row MUST display:
- **Track:** Track name · Artist name · Album name · Duration
- **Album:** Album name · Artist name · Year · Track count
- **Artist:** Artist name · Genre tags (up to 3) · Follower count

**FR-SEARCH-04** — Actions on Results  
From a selected search result item, the following MUST be available:
- `Enter` — Play immediately (track) or open/browse (album, artist)
- `a` — Add track to queue
- `p` — Add to a playlist (modal picker)

---

### 6.3 Playback Control Module

**FR-PLAY-01** — Transport Controls  
The following keybindings MUST trigger the corresponding Spotify API calls:

| Key | Action | API Endpoint |
|-----|--------|--------------|
| `Space` | Toggle play/pause | `PUT /me/player/play` · `PUT /me/player/pause` |
| `>` | Next track | `POST /me/player/next` |
| `<` | Previous track | `POST /me/player/previous` |
| `+` | Volume up 5% | `PUT /me/player/volume` |
| `-` | Volume down 5% | `PUT /me/player/volume` |
| `→` | Seek +10s | `PUT /me/player/seek` |
| `←` | Seek -10s | `PUT /me/player/seek` |
| `s` | Toggle shuffle | `PUT /me/player/shuffle` |
| `r` | Toggle repeat (off/track/context) | `PUT /me/player/repeat` |

**FR-PLAY-02** — Now Playing Pane  
A persistent "Now Playing" pane MUST display:
- Track title (truncated with `…` if overflowing)
- Artist name(s), comma-separated
- Album name
- Album art (ASCII, or SIXEL if supported)
- Playback progress bar (current position / total duration)
- Playback state icon (▶ / ⏸ / ⏹)
- Shuffle and repeat state indicators
- Volume percentage

**FR-PLAY-03** — Progress Bar Polling  
The progress bar MUST update every 1 second via a Tokio interval. The app MUST poll `GET /me/player` every 5 seconds to re-sync position (handles skips, external control). Local interpolation is used between polls to avoid excessive API calls.

**FR-PLAY-04** — Device Transfer  
Pressing `d` MUST open a modal listing all devices returned by `GET /me/player/devices`. Selecting one calls `PUT /me/player` to transfer playback. The modal MUST indicate the currently active device.

---

### 6.4 Playlist Module

**FR-PLAYLIST-01** — Playlist List  
On startup (post-auth), the app MUST fetch the user's playlists via `GET /me/playlists` (paginated, up to 50 per page). Results are displayed in the Playlist pane as a scrollable list showing: playlist name, track count, owner.

**FR-PLAYLIST-02** — Playlist Detail View  
Pressing `Enter` on a playlist MUST load and display its tracks in the Track List pane. Each track row shows: index, track name, artist, album, duration.

**FR-PLAYLIST-03** — Play from Playlist  
Pressing `Enter` on a track in the playlist detail view MUST call `PUT /me/player/play` with `context_uri` set to the playlist URI and `offset` set to the selected track index.

**FR-PLAYLIST-04** — Liked Songs  
"Liked Songs" MUST appear as the first item in the playlist pane, fetched via `GET /me/tracks`.

---

### 6.5 Queue Module

**FR-QUEUE-01** — Queue Display  
The Queue pane MUST display the upcoming tracks from `GET /me/player/queue`. Each row shows: position, track name, artist, duration.

**FR-QUEUE-02** — Add to Queue  
Pressing `a` on any selectable track (in search results, playlist, or album view) MUST call `POST /me/player/queue?uri={track_uri}` and display a brief toast notification confirming the addition.

**FR-QUEUE-03** — Queue Refresh  
The queue MUST refresh automatically when a track changes (detected via polling or when the user manually presses `R` in the Queue pane).

---

### 6.6 Lyrics Module

**FR-LYRICS-01** — Lyrics Fetching  
When the current track changes, the app MUST asynchronously fetch lyrics via the configured lyrics provider (default: `lyrics.ovh` public API). The fetch uses artist + track name as search parameters.

**FR-LYRICS-02** — Display  
Fetched lyrics MUST be displayed in the Lyrics pane as plain text, line-wrapped to fit the pane width. The pane MUST show a "Loading lyrics…" placeholder while the fetch is in progress, and "Lyrics not found" if the API returns no result.

**FR-LYRICS-03** — Manual Scroll  
When the Lyrics pane is focused (Tab to focus), `j`/`k` scroll the text. `gg`/`G` jump to top/bottom.

**FR-LYRICS-04** — Provider Abstraction  
Lyrics fetching MUST be implemented behind a `LyricsProvider` trait so alternative providers can be swapped via configuration without code changes.

```
trait LyricsProvider: Send + Sync {
    async fn fetch(&self, artist: &str, title: &str) -> Result<String, LyricsError>;
}
```

---

### 6.7 Album Art Module

**FR-ART-01** — Image Fetching  
When the current track changes, the app MUST download the album cover image URL from the track metadata (Spotify provides 3 sizes; the app selects the smallest ≥ 300px for quality/performance balance).

**FR-ART-02** — ASCII Art Rendering  
The image MUST be converted to ASCII art using a brightness-mapped character ramp. The implementation MUST:
- Resize the image to fit the art pane dimensions (width × 0.5 for terminal aspect ratio).
- Map pixel luminance to a character set (e.g., ` .:-=+*#%@`).
- Support optional color output (256-color or truecolor ANSI codes) via configuration flag.

**FR-ART-03** — SIXEL Rendering  
If the terminal reports SIXEL support (via `DA1` terminal response or `TERM` / `COLORTERM` env vars), the app MUST render album art using SIXEL escape sequences. This is opt-in via config: `album_art.renderer = "sixel"` (default: `"ascii"`).

**FR-ART-04** — Caching  
Fetched and converted album art MUST be cached in memory (LRU cache, max 20 entries) keyed by Spotify album ID to prevent redundant network fetches and re-conversion during the same session.

---

### 6.8 Terminal UI Module

**FR-UI-01** — Layout  
The default layout MUST follow this pane arrangement:

```
┌─────────────────────────────────────────────────────────┐
│  Ferrum  [Search: _____________]  [Device: MacBook Pro]  │  ← Header Bar
├───────────────────┬─────────────────────┬───────────────┤
│                   │                     │               │
│   Playlists       │   Track List /      │   Lyrics      │
│   (scrollable)    │   Search Results    │   (scrollable)│
│                   │                     │               │
│                   │                     │               │
├───────────────────┴─────────────────────┴───────────────┤
│  [Album Art]  Track · Artist · Album  [=====>    ] 2:14  │  ← Now Playing
│               ▶  Shuffle: ON  Repeat: Context  Vol: 72%  │
└─────────────────────────────────────────────────────────┘
│  [Tab] Cycle panes  [?] Help  [/] Search  [q] Quit       │  ← Status Bar
└─────────────────────────────────────────────────────────┘
```

**FR-UI-02** — Responsive Layout  
At terminal widths below 120 columns, the Lyrics pane MUST collapse. Below 80 columns, the Playlist pane MUST collapse and the Track List takes full width. The Now Playing bar MUST always be visible.

**FR-UI-03** — Pane Focus  
`Tab` cycles focus clockwise through visible panes. `Shift+Tab` cycles counter-clockwise. The active pane border MUST use a distinct color (theme-configurable; default: cyan).

**FR-UI-04** — Flicker-Free Rendering  
Ratatui's diff-based rendering MUST be used. The app MUST NOT call `terminal.clear()` every frame. Full redraws MUST only occur on terminal resize events.

**FR-UI-05** — Color Themes  
The app MUST ship with a default dark theme. Colors MUST be defined in a `Theme` struct loaded from `~/.config/ferrum/theme.toml`. A built-in `catppuccin-mocha` theme MUST be included as an alternative.

**FR-UI-06** — Modals  
The following modals MUST be implemented as overlays:
- Help overlay (`?`) — full keybinding reference
- Device picker (`d`) — list of active Spotify devices
- Add to playlist (`p`) — playlist selector

---

### 6.9 Keybinding System

**FR-KEY-01** — Modes  
The app MUST implement three input modes:

| Mode | Description | Trigger |
|------|-------------|---------|
| **Normal** | Default navigation mode | `Esc` from any other mode |
| **Insert** | Text input (search box) | `/` in Normal mode |
| **Command** | `:` command line | `:` in Normal mode |

**FR-KEY-02** — Normal Mode Bindings

| Key | Action |
|-----|--------|
| `j` / `↓` | Move selection down |
| `k` / `↑` | Move selection up |
| `gg` | Jump to top of list |
| `G` | Jump to bottom of list |
| `Ctrl+d` | Scroll down half page |
| `Ctrl+u` | Scroll up half page |
| `Enter` | Select / confirm |
| `Esc` | Cancel / back |
| `/` | Enter search (Insert mode) |
| `:` | Enter command mode |
| `Tab` | Next pane |
| `Shift+Tab` | Previous pane |
| `Space` | Play / pause |
| `>` | Next track |
| `<` | Previous track |
| `+` / `-` | Volume up / down |
| `→` / `←` | Seek forward / backward |
| `s` | Toggle shuffle |
| `r` | Cycle repeat mode |
| `d` | Open device picker |
| `a` | Add to queue |
| `p` | Add to playlist |
| `?` | Toggle help overlay |
| `q` | Quit |

**FR-KEY-03** — Command Mode  
Command mode (`:`) MUST support:

| Command | Action |
|---------|--------|
| `:q` | Quit |
| `:logout` | Clear auth tokens and restart |
| `:theme <name>` | Switch color theme |
| `:device` | Open device picker |
| `:help` | Show help overlay |

**FR-KEY-04** — Key Sequence Detection  
Double-key sequences (`gg`) MUST be handled with a configurable timeout (default: 500ms). If the second key is not pressed within the timeout, the first key action fires independently (if mapped) or is discarded.

---

### 6.10 State Management

**FR-STATE-01** — Central AppState  
A single `AppState` struct MUST hold all mutable application state. No UI component MUST hold state independently. All state mutations MUST flow through the central state.

**FR-STATE-02** — Event Bus  
An async MPSC channel MUST serve as the event bus. All user inputs, timer ticks, and API responses MUST be modeled as `AppEvent` enum variants and sent to the main event loop.

**FR-STATE-03** — Async API Tasks  
All Spotify API calls MUST be dispatched as Tokio tasks. Results MUST be sent back to the event loop via the event bus. The UI MUST never block on network I/O.

---

## 7. Non-Functional Requirements

### 7.1 Performance

| Metric | Target |
|--------|--------|
| Frame render time | < 16ms (60fps target) |
| Search result display latency | < 500ms from query submission |
| Track change → art update | < 2s (network permitting) |
| Track change → lyrics update | < 3s (network permitting) |
| App startup to interactive | < 500ms (with valid cached token) |
| Memory footprint (idle) | < 50MB RSS |

### 7.2 Reliability

- The app MUST NOT crash on Spotify API errors. All API calls MUST return `Result<T, E>` and errors MUST be surfaced as non-blocking status bar messages.
- Rate limit responses (HTTP 429) MUST be handled with exponential backoff and a user-visible "Rate limited" indicator.
- Network loss MUST be handled gracefully: playback state is preserved; reconnection retried with backoff.
- Panics MUST be treated as bugs. The app MUST use `color-eyre` for human-readable panic messages and MUST NOT use `unwrap()` or `expect()` in non-test code paths.

### 7.3 Security

- OAuth tokens MUST NOT be logged, printed to stdout, or included in error messages.
- The local HTTP callback server (for OAuth redirect) MUST bind to `127.0.0.1` only and MUST shut down immediately after receiving the authorization code.
- The `client_secret` field in config MUST be marked as sensitive and excluded from debug output.

### 7.4 Cross-Platform Compatibility

| Platform | Status |
|----------|--------|
| Linux (x86_64, aarch64) | Fully supported |
| macOS (Apple Silicon, Intel) | Fully supported |
| Windows (native) | Best-effort (crossterm handles most things) |
| Windows (WSL2) | Supported; SIXEL may not work |

### 7.5 Accessibility

- All functional information MUST be conveyed in text (not color alone).
- The app MUST work correctly in 16-color terminals (graceful degradation from truecolor).
- Screen reader support is out of scope for v1 but the architecture MUST NOT preclude it.

### 7.6 Configuration

- All user-facing settings MUST be configurable via `~/.config/ferrum/config.toml`.
- The app MUST generate a documented default config on first launch.
- Config errors (invalid TOML, unknown keys) MUST produce descriptive error messages at startup and fall back to defaults.

---

## 8. System Architecture Overview

### 8.1 High-Level Architecture

```
┌───────────────────────────────────────────────────────────────┐
│                        Ferrum Process                         │
│                                                               │
│  ┌─────────────┐    ┌──────────────────────────────────────┐  │
│  │  Terminal   │    │              Event Loop               │  │
│  │  (crossterm)│◄───┤  tokio::select! {                    │  │
│  │             │    │    input_event,                       │  │
│  │  Key Events │───►│    tick_event,                       │  │
│  │  Resize     │    │    api_response,                     │  │
│  └─────────────┘    │    lyrics_response,                  │  │
│                     │    art_response,                      │  │
│  ┌─────────────┐    │  }                                   │  │
│  │  Ratatui    │◄───┤                                      │  │
│  │  Renderer   │    │  ┌─────────────┐  ┌───────────────┐  │  │
│  │             │    │  │  AppState   │  │  AppAction    │  │  │
│  └─────────────┘    │  │  (central)  │◄─│  Dispatcher   │  │  │
│                     │  └─────────────┘  └───────────────┘  │  │
│                     └──────────────────────────────────────┘  │
│                                    │                          │
│             ┌──────────────────────┼─────────────────┐        │
│             ▼                      ▼                 ▼        │
│  ┌──────────────────┐  ┌──────────────────┐  ┌────────────┐  │
│  │  Spotify Client  │  │  Lyrics Client   │  │ Art Render │  │
│  │  (rspotify)      │  │  (reqwest)       │  │ (image-rs) │  │
│  └──────────────────┘  └──────────────────┘  └────────────┘  │
│             │                      │                 │        │
└────────────────────────────────────────────────────────────┘
             │                      │                 │
             ▼                      ▼                 ▼
    Spotify Web API          lyrics.ovh API     CDN (art URLs)
```

### 8.2 Threading Model

```
Main Thread
│
├── Tokio Runtime (multi-thread, default)
│   ├── Event Loop Task (main)
│   ├── Input Listener Task
│   ├── Tick Timer Task (1s intervals)
│   ├── Poll Timer Task (5s intervals)
│   ├── API Task Pool (spawned per request)
│   ├── Lyrics Fetch Task (per track change)
│   └── Art Fetch+Render Task (per track change)
```

All tasks communicate exclusively via `tokio::sync::mpsc` channels. The event loop owns the `AppState` and is the only writer. Tasks are read-only consumers or produce events.

---

## 9. Component Breakdown

### 9.1 UI Components (`src/ui/`)

| Component | Description |
|-----------|-------------|
| `app.rs` | Root layout, pane arrangement, focus routing |
| `now_playing.rs` | Progress bar, metadata, transport state display |
| `playlist_pane.rs` | Scrollable playlist list, selection state |
| `track_list.rs` | Reusable scrollable track list (used for playlists, search, albums) |
| `search_bar.rs` | Input widget with Insert mode awareness |
| `lyrics_pane.rs` | Scrollable text with line-wrap |
| `art_pane.rs` | ASCII/SIXEL art rendering target |
| `queue_pane.rs` | Queue display |
| `modal.rs` | Generic modal overlay container |
| `help_modal.rs` | Static keybinding reference |
| `device_modal.rs` | Device list picker |
| `status_bar.rs` | Mode indicator, toast messages, error display |
| `theme.rs` | Color/style tokens loaded from config |

### 9.2 State (`src/state/`)

| Module | Description |
|--------|-------------|
| `app_state.rs` | Central `AppState` struct with all sub-state fields |
| `playback_state.rs` | Current track, position, volume, shuffle/repeat flags |
| `search_state.rs` | Query string, results (tracks/albums/artists), active tab |
| `playlist_state.rs` | Playlist list, selected playlist, loaded tracks |
| `queue_state.rs` | Queue track list |
| `lyrics_state.rs` | Lyrics text, scroll offset, loading flag |
| `art_state.rs` | Cached ASCII/SIXEL strings per album ID |
| `ui_state.rs` | Active pane, input mode, modal visibility |

### 9.3 Events (`src/events/`)

```rust
pub enum AppEvent {
    // Input
    Key(KeyEvent),
    Resize(u16, u16),

    // Timers
    Tick,          // 1s — progress bar update
    Poll,          // 5s — full state re-sync from Spotify

    // API Results
    PlaybackUpdated(Box<CurrentPlaybackContext>),
    PlaylistsLoaded(Vec<SimplifiedPlaylist>),
    PlaylistTracksLoaded(PlaylistId, Vec<PlaylistItem>),
    SearchResults(SearchResults),
    QueueLoaded(QueueContext),
    DevicesLoaded(Vec<Device>),
    ApiError(SpotifyError),

    // Lyrics
    LyricsLoaded(String),
    LyricsFailed,

    // Art
    ArtReady(AlbumId, ArtPayload),  // ArtPayload = Ascii(String) | Sixel(Vec<u8>)

    // Commands
    Quit,
}
```

### 9.4 API Layer (`src/api/`)

| Module | Description |
|--------|-------------|
| `spotify.rs` | Thin async wrapper around `rspotify` client with retry/backoff |
| `auth.rs` | PKCE flow, token storage, refresh scheduling |
| `lyrics.rs` | `LyricsProvider` trait + `LyricsOvhProvider` impl |
| `art.rs` | Image download, resize, ASCII conversion, SIXEL encoding |
| `error.rs` | Unified `ApiError` type with context |

### 9.5 Configuration (`src/config/`)

| Module | Description |
|--------|-------------|
| `config.rs` | `AppConfig` struct, TOML deserialization, defaults |
| `keymap.rs` | Rebindable key map loaded from config |
| `theme.rs` | `Theme` struct, default themes, TOML loading |

---

## 10. Data Flow Diagrams

### 10.1 User Input → Playback Action

```
User presses [Space]
       │
       ▼
Input Listener Task
  detects KeyEvent(Space)
       │
       ▼
AppEvent::Key(Space) sent to Event Loop channel
       │
       ▼
Event Loop receives event
  matches InputMode::Normal + Key(Space)
  dispatches AppAction::TogglePlayback
       │
       ▼
Action Dispatcher
  spawns Tokio task:
    spotify_client.toggle_playback()
       │
       ├── API call succeeds
       │       │
       │       ▼
       │   AppEvent::PlaybackUpdated(ctx) → Event Loop
       │       │
       │       ▼
       │   AppState.playback.is_playing updated
       │   Ratatui re-render triggered
       │
       └── API call fails
               │
               ▼
           AppEvent::ApiError(e) → Event Loop
               │
               ▼
           AppState.ui.toast = "Playback error: {e}"
           Status bar shows toast (3s auto-dismiss)
```

### 10.2 Track Change → Art + Lyrics Update

```
Tick/Poll detects track changed
       │
       ├──────────────────────────────────────┐
       ▼                                      ▼
Spawn LyricsFetch Task                 Spawn ArtFetch Task
  GET lyrics.ovh/{artist}/{title}        GET {art_url from track metadata}
       │                                      │
       ▼                                      ▼
  LyricsLoaded(text) or LyricsFailed    image bytes decoded
                                         resized to pane dimensions
                                         converted to ASCII (or SIXEL)
                                              │
                                              ▼
                                         ArtReady(album_id, payload)
                                              │
       ├──────────────────────────────────────┘
       ▼
Event Loop processes both events independently
AppState.lyrics / AppState.art updated
Ratatui re-render triggered
```

### 10.3 Authentication Flow

```
App Startup
    │
    ▼
Config loaded → token file checked
    │
    ├── Token valid → skip to Main Loop
    │
    ├── Token expired + refresh_token present
    │       │
    │       ▼
    │   POST /api/token (refresh grant)
    │       │
    │       ├── Success → save new token → Main Loop
    │       └── Failure → trigger full re-auth
    │
    └── No token / re-auth needed
            │
            ▼
        Generate PKCE code_verifier + code_challenge
            │
            ▼
        Open browser to Spotify auth URL
            │
            ▼
        Local HTTP server on 127.0.0.1:8888 listens
            │
            ▼
        User authorizes → redirect with ?code=...
            │
            ▼
        POST /api/token (code exchange)
            │
            ▼
        Store access_token + refresh_token
            │
            ▼
        Shut down local HTTP server
            │
            ▼
        Main Loop
```

---

## 11. Key Technical Decisions

### 11.1 Ratatui over tui-rs

**Decision:** Use `ratatui` (the active fork of `tui-rs`).  
**Rationale:** `tui-rs` is unmaintained. `ratatui` is actively developed, has a growing ecosystem, and has fixed several rendering bugs. The API is backward-compatible.

### 11.2 rspotify over raw HTTP

**Decision:** Use `rspotify` crate with the `client-reqwest` feature, not hand-rolled HTTP calls.  
**Rationale:** `rspotify` covers the full Spotify Web API surface with typed responses. The OAuth PKCE flow implementation is provided and tested. Hand-rolling this introduces maintenance burden and auth security risk.  
**Trade-off:** `rspotify` is occasionally behind the Spotify API; we may need to submit PRs or work around gaps for newer endpoints (e.g., `GET /me/player/queue`, which was added post-initial release).

### 11.3 MPSC Event Bus over Direct State Mutation

**Decision:** All state mutations happen exclusively via the event loop; components send events, not direct mutations.  
**Rationale:** This enforces unidirectional data flow, makes state transitions auditable (all events can be logged), and prevents race conditions from concurrent async tasks writing to shared state.  
**Trade-off:** Slight latency overhead per event round-trip; negligible at this scale.

### 11.4 ASCII Art via Luminance Mapping (image-rs)

**Decision:** Use `image` crate for decoding/resizing; implement ASCII conversion ourselves.  
**Rationale:** Dedicated ASCII art crates (e.g., `viuer`) handle terminal-specific rendering but are opinionated about output format. Implementing conversion ourselves gives full control over character ramp, color mode, and aspect ratio correction.  
**SIXEL:** Use `sixel-rs` or `sixel-tokenizer` crate. Detect support via `TERM` / `VTE_VERSION` env and `DA1` terminal query.

### 11.5 Lyrics via lyrics.ovh (Default)

**Decision:** Default to `lyrics.ovh` public API; abstract behind `LyricsProvider` trait.  
**Rationale:** `lyrics.ovh` is free, requires no API key, and has broad coverage. Abstracting it means users can configure alternatives (Genius API, AZLyrics scraper) without upstream changes.  
**Risk:** `lyrics.ovh` has reliability issues. The `LyricsProvider` trait means the failure cost is low — swap the provider.

### 11.6 Tokio Multi-Thread Runtime

**Decision:** Use `tokio::main` with the multi-thread scheduler.  
**Rationale:** API calls and image processing are CPU/IO bound. Multi-thread allows parallelism without blocking the event loop. The single-threaded scheduler would require careful manual task yielding to keep the UI responsive during image decode.

### 11.7 config.toml over CLI flags

**Decision:** Persistent configuration lives in `~/.config/ferrum/config.toml`; CLI flags are only for overrides and launch options.  
**Rationale:** TUI apps are session-persistent; users set preferences once. CLI flags are appropriate for one-shot overrides (`--theme`, `--config-path`) but not primary configuration.

---

## 12. Risks and Mitigations

| # | Risk | Probability | Impact | Mitigation |
|---|------|-------------|--------|------------|
| R1 | Spotify API breaking changes / deprecations | Medium | High | Pin rspotify version; monitor Spotify developer changelog; wrap API calls to isolate changes |
| R2 | lyrics.ovh downtime or shutdown | High | Medium | `LyricsProvider` trait abstracts the dependency; add Genius as fallback in v1.1 |
| R3 | SIXEL support inconsistency across terminals | High | Low | SIXEL is optional; ASCII is always the fallback; detect support at runtime |
| R4 | OAuth PKCE callback port (8888) conflict | Low | Medium | Make callback port configurable in `config.toml`; retry on next available port |
| R5 | rspotify missing or broken `GET /me/player/queue` | Medium | Medium | Implement raw `reqwest` fallback for this endpoint specifically |
| R6 | Terminal resize causes layout panic | Medium | High | Clamp all layout calculations to minimum safe dimensions; test with 80×24 |
| R7 | Spotify Premium required for playback control | Certain | Medium | Document clearly in README; graceful error message for free-tier users |
| R8 | Cross-platform input edge cases (Windows terminal) | Medium | Low | `crossterm` abstracts most; add Windows-specific testing in CI via GitHub Actions |
| R9 | Event loop contention during burst API calls | Low | Medium | Bound API task concurrency with a semaphore (`tokio::sync::Semaphore`, max 5 concurrent) |
| R10 | Config schema churn between versions | Medium | Medium | Use `serde(default)` for all fields; old configs remain valid; emit deprecation warnings |

---

## 13. Future Enhancements

### v1.1 — Quality of Life

- **Genius lyrics integration** — Authenticated Genius API as secondary lyrics provider; richer metadata (annotations).
- **Keybinding remapping** — Full keymap customization via `config.toml`.
- **Mouse support** — Optional click-to-focus and scroll via crossterm mouse events.
- **Playlist editing** — Create, rename, reorder, and delete playlists from within the app.
- **History view** — Recently played tracks via `GET /me/player/recently-played`.

### v1.2 — Advanced Playback

- **Spotify Connect full support** — Volume control per device; remote device management.
- **Crossfade + gapless simulation** — Pre-fetch next track metadata to reduce perceived gap.
- **Radio / recommendations** — "Play radio" seeded from current track via Spotify recommendations API.
- **Synchronized lyrics** — If Spotify's internal lyrics endpoint becomes accessible, implement line-highlight sync with playback position.

### v2.0 — Multi-Provider

The architecture is designed for this from day one. The required abstraction:

```rust
pub trait MusicProvider: Send + Sync {
    async fn search(&self, query: &str) -> Result<SearchResults>;
    async fn get_playback(&self) -> Result<PlaybackContext>;
    async fn play(&self, uri: &str, offset: Option<usize>) -> Result<()>;
    async fn pause(&self) -> Result<()>;
    async fn next(&self) -> Result<()>;
    async fn get_playlists(&self) -> Result<Vec<Playlist>>;
    async fn get_queue(&self) -> Result<Vec<Track>>;
    // ... etc
}
```

Planned providers:
- **Apple Music** (MusicKit JS via web API or native macOS MediaPlayer framework)
- **Tidal** (unofficial REST API; no official SDK)
- **Last.fm scrobbling** — Cross-provider scrobble overlay (not a music source, but a listener)
- **Local files** — Via `symphonia` audio decoder; would introduce actual audio output (breaking the "controller only" model — explicit design decision for v2)

### v3.0+ — Ecosystem

- **Plugin API** — Lua or WASM-based plugin system for custom keybindings, UI widgets, and data sources.
- **Multiplayer / Shared sessions** — Listen-along via WebRTC or relay server.
- **Ferrum Daemon** — Headless background service for scrobbling and playback control via IPC, enabling scriptability without opening the TUI.

---

## 14. Success Metrics

### 14.1 Adoption Metrics (6 months post-launch)

| Metric | Target |
|--------|--------|
| GitHub Stars | 1,000+ |
| crates.io downloads | 5,000+ |
| Active contributors | 10+ |
| Open issues resolved within 2 weeks | ≥ 70% |

### 14.2 Quality Metrics

| Metric | Target |
|--------|--------|
| Crash rate (panics in the wild) | < 0.1% of sessions |
| Test coverage (unit + integration) | ≥ 70% |
| CI green rate (main branch) | ≥ 98% |
| Clippy warnings in CI | Zero (enforced) |

### 14.3 Performance Metrics (measured with `cargo bench` + manual testing)

| Metric | Target |
|--------|--------|
| Frame render time (80×24 terminal) | < 5ms |
| Frame render time (220×50 terminal) | < 16ms |
| Memory footprint after 1hr session | < 80MB RSS |
| Cold startup to interactive | < 500ms |

### 14.4 User Satisfaction (measured via GitHub Discussions)

- Zero open "app feels laggy" complaints after v1.0 stabilization.
- Keybinding system rated intuitive by ≥ 80% of surveyed users.
- Users migrate from `ncspot`/`spotify-tui` and report feature parity or improvement.

---

## Appendix A — Configuration Schema Reference

```toml
# ~/.config/ferrum/config.toml

[auth]
client_id = "YOUR_SPOTIFY_CLIENT_ID"   # Required
redirect_port = 8888                    # OAuth callback port

[playback]
volume_step = 5                         # Percent per +/- keypress
seek_step_secs = 10                     # Seconds per seek keypress
poll_interval_secs = 5                  # How often to sync with Spotify API

[ui]
theme = "default"                       # "default" | "catppuccin-mocha" | path to file
show_lyrics = true
show_queue = true
min_width = 80
min_height = 24

[album_art]
renderer = "ascii"                      # "ascii" | "sixel" | "none"
color = true                            # Enable ANSI color in ASCII mode
max_cache_entries = 20

[lyrics]
provider = "lyrics_ovh"                # "lyrics_ovh" | "genius"
genius_api_key = ""                    # Required only if provider = "genius"

[keybindings]
# Override defaults — see docs for full list
play_pause = "space"
next_track = ">"
prev_track = "<"
```

---

## Appendix B — Crate Dependency Summary

| Crate | Version | Purpose |
|-------|---------|---------|
| `ratatui` | 0.28+ | TUI framework |
| `crossterm` | 0.28+ | Terminal backend (input/output) |
| `rspotify` | 0.13+ | Spotify Web API client |
| `tokio` | 1.x (full features) | Async runtime |
| `reqwest` | 0.12+ | HTTP client (lyrics, art) |
| `image` | 0.25+ | Image decode/resize |
| `sixel-rs` | latest | SIXEL encoding |
| `serde` / `toml` | latest | Config deserialization |
| `keyring` | 2.x | OS keychain token storage |
| `color-eyre` | 0.6+ | Error reporting |
| `tracing` / `tracing-subscriber` | latest | Structured logging |
| `lru` | 0.12+ | LRU cache for art |
| `webbrowser` | latest | Open auth URL in system browser |
| `tiny_http` | 0.12+ | Local OAuth callback HTTP server |

---

*End of Document — Ferrum PRD v1.0.0*