pub mod theme;
pub mod keymap;

use std::path::{Path, PathBuf};
use std::fs;
use serde::{Deserialize, Serialize};
use crate::config::theme::Theme;
use crate::config::keymap::KeybindingsConfig;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    pub auth: AuthConfig,
    pub playback: PlaybackConfig,
    pub ui: UiConfig,
    pub lyrics: LyricsConfig,
    pub keybindings: KeybindingsConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LyricsConfig {
    pub provider: String,
    pub genius_api_key: String,
}

impl Default for LyricsConfig {
    fn default() -> Self {
        Self {
            provider: "lrclib".to_string(),
            genius_api_key: "".to_string(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuthConfig {
    pub client_id: String,
    pub redirect_port: u16,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlaybackConfig {
    pub volume_step: u8,
    pub seek_step_secs: u32,
    pub poll_interval_secs: u64,
    pub crossfade_secs: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UiConfig {
    pub theme: String,
    pub show_lyrics: bool,
    pub show_queue: bool,
    pub min_width: u16,
    pub min_height: u16,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            auth: AuthConfig {
                client_id: "PLACEHOLDER".to_string(),
                redirect_port: 8888,
            },
            playback: PlaybackConfig {
                volume_step: 5,
                seek_step_secs: 10,
                poll_interval_secs: 5,
                crossfade_secs: 0,
            },
            ui: UiConfig {
                theme: "default".to_string(),
                show_lyrics: true,
                show_queue: true,
                min_width: 80,
                min_height: 24,
            },
            lyrics: LyricsConfig::default(),
            keybindings: KeybindingsConfig::default(),
        }
    }
}

impl Config {
    pub fn load() -> Self {
        let config_path = Self::config_path();
        if config_path.exists() {
            let content = fs::read_to_string(&config_path).unwrap_or_default();
            toml::from_str(&content).unwrap_or_else(|_| {
                // If it fails to parse, return default but maybe we should log an error?
                Self::default()
            })
        } else {
            let config = Self::default();
            let _ = config.save();
            config
        }
    }

    pub fn save(&self) -> color_eyre::Result<()> {
        let config_path = Self::config_path();
        if let Some(parent) = config_path.parent() {
            fs::create_dir_all(parent)?;
        }
        let content = toml::to_string_pretty(self)?;
        fs::write(config_path, content)?;
        Ok(())
    }

    fn config_path() -> PathBuf {
        let mut path = if cfg!(windows) {
            PathBuf::from(std::env::var("USERPROFILE").unwrap_or_else(|_| ".".to_string()))
        } else {
            PathBuf::from(std::env::var("HOME").unwrap_or_else(|_| ".".to_string()))
        };
        path.push(".config");
        path.push("spotui");
        path.push("config.toml");
        path
    }
}
