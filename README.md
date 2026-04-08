# Ferrum (SpotUI) — Terminal Spotify Client

Ferrum is a keyboard-driven, terminal-native music streaming client for Spotify, built in Rust using Ratatui.

## Features

- **Vim-style modal interface**: Normal, Insert, and Command modes.
- **Search**: Tracks, Albums, and Artists.
- **Library Management**: Access your playlists and liked songs.
- **Lyrics Integration**: View lyrics from `lyrics.ovh`.
- **Album Art**: ASCII rendering of album covers.
- **Multi-pane Layout**: Responsive design that adapts to terminal size.

## Installation

### Prerequisites

- A Spotify Premium account.
- A Spotify Developer Application (get a `Client ID` at [developer.spotify.com](https://developer.spotify.com/dashboard)).
- Set the Redirect URI of your Spotify App to `http://localhost:8888/callback`.

### From Source

```bash
git clone https://github.com/youruser/spotui
cd spotui
cargo build --release
```

## Setup

On first launch, the app will generate a default configuration file at:
- Windows: `%USERPROFILE%\.config\spotui\config.toml`
- Linux/macOS: `~/.config/spotui/config.toml`

Edit this file and add your `client_id`:

```toml
[auth]
client_id = "your_spotify_client_id_here"
redirect_port = 8888
```

You can also set the `SPOTIFY_CLIENT_ID` environment variable.

## Keybindings

### Normal Mode

| Key | Action |
|-----|--------|
| `j` / `↓` | Move selection down |
| `k` / `↑` | Move selection up |
| `Tab` | Next pane |
| `Shift+Tab` | Previous pane |
| `Space` | Play / Pause |
| `n` | Next track |
| `p` | Previous track |
| `+` / `-` | Volume up / down |
| `h` / `l` | Seek backward / forward (10s) |
| `/` | Enter Search mode |
| `:` | Enter Command mode |
| `d` | Open Devices pane |
| `Q` | Open Queue pane |
| `a` | Add selected track to queue |
| `?` | Toggle help overlay |
| `q` | Quit |

### Search Mode

| Key | Action |
|-----|--------|
| `Enter` | Submit search / Select result |
| `Esc` | Return to Normal mode |
| `Tab` | Cycle search result categories (Tracks/Albums/Artists) |

### Command Mode

| Command | Action |
|---------|--------|
| `:q` | Quit |
| `:logout` | Log out and clear tokens |
| `:theme <name>` | Change theme (`dark`, `catppuccin-mocha`) |
| `:device` | Open device picker |
| `:help` | Show help overlay |

## Theming

Ferrum supports TOML-based themes. You can change the theme in your `config.toml` or via the `:theme` command.

Currently supported built-in themes:
- `default` / `dark`
- `catppuccin-mocha`

## License

MIT
