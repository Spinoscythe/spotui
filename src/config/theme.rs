use ratatui::style::Color;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Theme {
    pub border: Color,
    pub border_active: Color,
    pub text: Color,
    pub text_dim: Color,
    pub text_highlight: Color,
    pub progress_bar: Color,
    pub progress_bar_bg: Color,
    pub status_bar: Color,
    pub error: Color,
}

impl Default for Theme {
    fn default() -> Self {
        Self::dark()
    }
}

impl Theme {
    pub fn dark() -> Self {
        Self {
            border: Color::DarkGray,
            border_active: Color::Cyan,
            text: Color::White,
            text_dim: Color::Gray,
            text_highlight: Color::Green,
            progress_bar: Color::Green,
            progress_bar_bg: Color::DarkGray,
            status_bar: Color::Blue,
            error: Color::Red,
        }
    }

    pub fn catppuccin_mocha() -> Self {
        Self {
            border: Color::Rgb(88, 91, 112),        // Surface 1
            border_active: Color::Rgb(137, 180, 250), // Blue
            text: Color::Rgb(205, 214, 244),         // Text
            text_dim: Color::Rgb(147, 153, 178),     // Overlay 0
            text_highlight: Color::Rgb(166, 227, 161),// Green
            progress_bar: Color::Rgb(166, 227, 161),  // Green
            progress_bar_bg: Color::Rgb(49, 50, 68),  // Surface 0
            status_bar: Color::Rgb(203, 166, 247),    // Mauve
            error: Color::Rgb(243, 139, 168),         // Red
        }
    }

    pub fn from_file(path: &std::path::Path) -> Result<Self, Box<dyn std::error::Error>> {
        let content = std::fs::read_to_string(path)?;
        let theme: Theme = toml::from_str(&content)?;
        Ok(theme)
    }
}
