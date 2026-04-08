use serde::{Deserialize, Serialize};
use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KeybindingsConfig {
    pub play_pause: String,
    pub next_track: String,
    pub prev_track: String,
    pub seek_forward: String,
    pub seek_backward: String,
    pub volume_up: String,
    pub volume_down: String,
    pub next_pane: String,
    pub prev_pane: String,
    pub quit: String,
    pub help: String,
    pub search: String,
    pub devices: String,
    pub queue: String,
    pub command: String,
    pub move_up: String,
    pub move_down: String,
    pub select: String,
    pub play_radio: String,
}

impl Default for KeybindingsConfig {
    fn default() -> Self {
        Self {
            play_pause: " ".to_string(),
            next_track: "n".to_string(),
            prev_track: "p".to_string(),
            seek_forward: "l".to_string(),
            seek_backward: "h".to_string(),
            volume_up: "+".to_string(),
            volume_down: "-".to_string(),
            next_pane: "tab".to_string(),
            prev_pane: "backtab".to_string(),
            quit: "q".to_string(),
            help: "?".to_string(),
            search: "/".to_string(),
            devices: "d".to_string(),
            queue: "Q".to_string(),
            command: ":".to_string(),
            move_up: "k".to_string(),
            move_down: "j".to_string(),
            select: "enter".to_string(),
            play_radio: "G".to_string(),
        }
    }
}

pub fn parse_key(key_str: &str) -> Option<KeyEvent> {
    let key = match key_str.to_lowercase().as_str() {
        "enter" => KeyCode::Enter,
        "esc" | "escape" => KeyCode::Esc,
        "tab" => KeyCode::Tab,
        "backtab" => KeyCode::BackTab,
        "space" | " " => KeyCode::Char(' '),
        "up" => KeyCode::Up,
        "down" => KeyCode::Down,
        "left" => KeyCode::Left,
        "right" => KeyCode::Right,
        s if s.len() == 1 => KeyCode::Char(s.chars().next().unwrap()),
        _ => return None,
    };

    Some(KeyEvent::new(key, KeyModifiers::empty()))
}

pub fn matches_key(key: KeyEvent, binding: &str) -> bool {
    if let Some(target) = parse_key(binding) {
        // Special case for BackTab since it's often Shift+Tab
        if key.code == KeyCode::BackTab && target.code == KeyCode::BackTab {
            return true;
        }
        
        // General comparison (ignoring modifiers for simplicity for now, as requested by basic remapping)
        key.code == target.code
    } else {
        false
    }
}
