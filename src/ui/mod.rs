use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    widgets::{Block, Borders, Paragraph, Tabs, List, ListItem, Gauge, Clear},
    Frame,
};
use crate::state::{AppState, InputMode};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ActivePane {
    Playlists,
    TrackList,
    Lyrics,
    Search,
    Devices,
    Queue,
}

impl ActivePane {
    pub fn next(self) -> Self {
        match self {
            Self::Playlists => Self::TrackList,
            Self::TrackList => Self::Lyrics,
            Self::Lyrics => Self::Search,
            Self::Search => Self::Devices,
            Self::Devices => Self::Queue,
            Self::Queue => Self::Playlists,
        }
    }

    pub fn prev(self) -> Self {
        match self {
            Self::Playlists => Self::Queue,
            Self::TrackList => Self::Playlists,
            Self::Lyrics => Self::TrackList,
            Self::Search => Self::Lyrics,
            Self::Devices => Self::Search,
            Self::Queue => Self::Devices,
        }
    }
}

pub fn render(f: &mut Frame, state: &mut AppState) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3), // Header
            Constraint::Min(0),    // Main area
            Constraint::Length(7), // Now Playing (Increased for art)
            Constraint::Length(1), // Status Bar
        ])
        .split(f.area());

    render_header(f, chunks[0], state);
    render_main(f, chunks[1], state);
    render_now_playing(f, chunks[2], state);
    render_status_bar(f, chunks[3], state);
    
    state.now_playing_area = chunks[2];

    if state.show_help {
        render_help(f, state);
    }
}

fn render_header(f: &mut Frame, area: Rect, state: &AppState) {
    let title = Paragraph::new(" Ferrum (Spotify TUI) ")
        .block(Block::default().borders(Borders::ALL));
    f.render_widget(title, area);
}

fn render_main(f: &mut Frame, area: Rect, state: &mut AppState) {
    match state.active_pane {
        ActivePane::Search => {
            state.search_area = area;
            render_search(f, area, state);
        }
        ActivePane::Devices => {
            state.devices_area = area;
            render_devices(f, area, state);
        }
        ActivePane::Queue => {
            state.queue_area = area;
            render_queue(f, area, state);
        }
        ActivePane::Lyrics => {
            state.lyrics_area = area;
            render_lyrics(f, area, state);
        }
        _ => {
            state.search_area = Rect::default();
            state.devices_area = Rect::default();
            state.queue_area = Rect::default();
            
            let mut constraints = vec![
                Constraint::Percentage(20), // Playlists
                Constraint::Min(0),         // Track List
                Constraint::Percentage(30), // Lyrics
            ];

            if area.width < 120 {
                constraints.pop(); // Remove Lyrics
                state.lyrics_area = Rect::default();
            }
            if area.width < 80 {
                constraints.remove(0); // Remove Playlists
                state.playlist_area = Rect::default();
            }

            let main_chunks = Layout::default()
                .direction(Direction::Horizontal)
                .constraints(constraints)
                .split(area);

            let mut current_chunk = 0;

            if area.width >= 80 {
                state.playlist_area = main_chunks[current_chunk];
                let items: Vec<ListItem> = state.playlists.iter().enumerate().map(|(i, p)| {
                    let style = if i == state.playlist_index && state.active_pane == ActivePane::Playlists {
                        Style::default().fg(state.theme.text_highlight).add_modifier(Modifier::BOLD)
                    } else {
                        Style::default().fg(state.theme.text)
                    };
                    ListItem::new(p.name.clone()).style(style)
                }).collect();

                let playlist_list = List::new(items)
                    .block(Block::default()
                        .title(" Playlists ")
                        .borders(Borders::ALL)
                        .border_style(if state.active_pane == ActivePane::Playlists {
                            Style::default().fg(state.theme.border_active)
                        } else {
                            Style::default().fg(state.theme.border)
                        }));
                f.render_widget(playlist_list, main_chunks[current_chunk]);
                current_chunk += 1;
            }

            state.track_list_area = main_chunks[current_chunk];
            let track_items: Vec<ListItem> = state.playlist_tracks.iter().enumerate().map(|(i, item)| {
                let name = match &item.track {
                    Some(rspotify::model::PlayableItem::Track(t)) => t.name.clone(),
                    Some(rspotify::model::PlayableItem::Episode(e)) => e.name.clone(),
                    None => "Unknown".to_string(),
                };
                let style = if i == state.track_index && state.active_pane == ActivePane::TrackList {
                    Style::default().fg(state.theme.text_highlight).add_modifier(Modifier::BOLD)
                } else {
                    Style::default().fg(state.theme.text)
                };
                ListItem::new(name).style(style)
            }).collect();

            let track_list = List::new(track_items)
                .block(Block::default()
                    .title(" Track List ")
                    .borders(Borders::ALL)
                    .border_style(if state.active_pane == ActivePane::TrackList {
                        Style::default().fg(state.theme.border_active)
                    } else {
                        Style::default().fg(state.theme.border)
                    }));
            f.render_widget(track_list, main_chunks[current_chunk]);
            current_chunk += 1;

            if area.width >= 120 {
                state.lyrics_area = main_chunks[current_chunk];
                render_lyrics(f, main_chunks[current_chunk], state);
            }
        }
    }
}

fn render_lyrics(f: &mut Frame, area: Rect, state: &AppState) {
    let block = Block::default()
        .title(" Lyrics ")
        .borders(Borders::ALL)
        .border_style(if state.active_pane == ActivePane::Lyrics {
            Style::default().fg(state.theme.border_active)
        } else {
            Style::default().fg(state.theme.border)
        });

    if state.lyrics_loading {
        f.render_widget(Paragraph::new("Loading lyrics...").block(block).style(Style::default().fg(state.theme.text_dim)), area);
        return;
    }

    if !state.synced_lyrics.is_empty() {
        let mut text = ratatui::text::Text::default();
        for (i, line) in state.synced_lyrics.iter().enumerate() {
            let style = if i == state.lyrics_scroll_offset as usize {
                Style::default().fg(state.theme.text_highlight).add_modifier(Modifier::BOLD)
            } else {
                Style::default().fg(state.theme.text)
            };
            text.lines.push(ratatui::text::Line::from(ratatui::text::Span::styled(line.text.clone(), style)));
        }
        
        let visible_height = (area.height as i32 - 2).max(1);
        let scroll = (state.lyrics_scroll_offset as i32 - visible_height / 2).max(0) as u16;

        let paragraph = Paragraph::new(text)
            .block(block)
            .scroll((scroll, 0));
        f.render_widget(paragraph, area);
    } else {
        let content = state.lyrics.as_deref().unwrap_or("No lyrics available");
        let paragraph = Paragraph::new(content)
            .block(block)
            .style(Style::default().fg(state.theme.text))
            .scroll((state.lyrics_scroll_offset, 0))
            .wrap(ratatui::widgets::Wrap { trim: true });
        f.render_widget(paragraph, area);
    }
}

fn render_search(f: &mut Frame, area: Rect, state: &AppState) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3), // Search bar
            Constraint::Length(3), // Tabs
            Constraint::Min(0),    // Results
        ])
        .split(area);

    let search_bar = Paragraph::new(state.search_query.as_str())
        .block(Block::default()
            .title(" Search ")
            .borders(Borders::ALL)
            .border_style(if state.input_mode == InputMode::Insert {
                Style::default().fg(state.theme.border_active)
            } else {
                Style::default().fg(state.theme.border)
            }));
    f.render_widget(search_bar, chunks[0]);

    let tabs = vec!["Tracks", "Albums", "Artists"];
    let tab_widget = Tabs::new(tabs)
        .block(Block::default().borders(Borders::ALL).title(" Tabs "))
        .select(state.search_active_tab)
        .highlight_style(Style::default().fg(state.theme.text_highlight).add_modifier(Modifier::BOLD))
        .style(Style::default().fg(state.theme.text));
    f.render_widget(tab_widget, chunks[1]);

    if let Some(results) = &state.search_results {
        match state.search_active_tab {
            0 => { // Tracks
                if let Some(tracks) = &results.tracks {
                    let items: Vec<ListItem> = tracks.items.iter().enumerate().map(|(i, t)| {
                        let style = if i == state.search_track_index {
                            Style::default().fg(state.theme.text_highlight).add_modifier(Modifier::BOLD)
                        } else {
                            Style::default().fg(state.theme.text)
                        };
                        ListItem::new(format!("{} - {}", t.name, t.artists.iter().map(|a| a.name.as_str()).collect::<Vec<_>>().join(", "))).style(style)
                    }).collect();
                    let list = List::new(items).block(Block::default().title(" Tracks ").borders(Borders::ALL).border_style(Style::default().fg(state.theme.border)));
                    f.render_widget(list, chunks[2]);
                }
            }
            1 => { // Albums
                if let Some(albums) = &results.albums {
                    let items: Vec<ListItem> = albums.items.iter().enumerate().map(|(i, a)| {
                        let style = if i == state.search_album_index {
                            Style::default().fg(state.theme.text_highlight).add_modifier(Modifier::BOLD)
                        } else {
                            Style::default().fg(state.theme.text)
                        };
                        ListItem::new(a.name.clone()).style(style)
                    }).collect();
                    let list = List::new(items).block(Block::default().title(" Albums ").borders(Borders::ALL).border_style(Style::default().fg(state.theme.border)));
                    f.render_widget(list, chunks[2]);
                }
            }
            2 => { // Artists
                if let Some(artists) = &results.artists {
                    let items: Vec<ListItem> = artists.items.iter().enumerate().map(|(i, a)| {
                        let style = if i == state.search_artist_index {
                            Style::default().fg(state.theme.text_highlight).add_modifier(Modifier::BOLD)
                        } else {
                            Style::default().fg(state.theme.text)
                        };
                        ListItem::new(a.name.clone()).style(style)
                    }).collect();
                    let list = List::new(items).block(Block::default().title(" Artists ").borders(Borders::ALL).border_style(Style::default().fg(state.theme.border)));
                    f.render_widget(list, chunks[2]);
                }
            }
            _ => {}
        }
    }
}

fn render_devices(f: &mut Frame, area: Rect, state: &AppState) {
    let items: Vec<ListItem> = state.devices.iter().enumerate().map(|(i, d)| {
        let style = if i == state.device_index {
            Style::default().fg(state.theme.text_highlight).add_modifier(Modifier::BOLD)
        } else {
            Style::default().fg(state.theme.text)
        };
        let active_str = if d.is_active { " (Active)" } else { "" };
        let vol_str = if let Some(v) = d.volume_percent { format!(" [Vol: {}%]", v) } else { "".to_string() };
        ListItem::new(format!("{}{}{}", d.name, active_str, vol_str)).style(style)
    }).collect();

    let list = List::new(items)
        .block(Block::default().title(" Devices ").borders(Borders::ALL).border_style(if state.active_pane == ActivePane::Devices { Style::default().fg(state.theme.border_active) } else { Style::default().fg(state.theme.border) }));
    f.render_widget(list, area);
}

fn render_queue(f: &mut Frame, area: Rect, state: &AppState) {
    let items: Vec<ListItem> = state.queue.iter().enumerate().map(|(i, t)| {
        let style = if i == state.queue_index {
            Style::default().fg(state.theme.text_highlight).add_modifier(Modifier::BOLD)
        } else {
            Style::default().fg(state.theme.text)
        };
        ListItem::new(format!("{} - {}", t.name, t.artists.iter().map(|a| a.name.as_str()).collect::<Vec<_>>().join(", "))).style(style)
    }).collect();

    let list = List::new(items)
        .block(Block::default().title(" Queue ").borders(Borders::ALL).border_style(Style::default().fg(state.theme.border)));
    f.render_widget(list, area);
}

fn render_now_playing(f: &mut Frame, area: Rect, state: &AppState) {
    let main_block = Block::default()
        .title(" Now Playing ")
        .borders(Borders::ALL)
        .border_style(Style::default().fg(state.theme.border));
    f.render_widget(main_block, area);

    let inner_area = area.inner(ratatui::layout::Margin { horizontal: 1, vertical: 1 });
    
    let mut constraints = vec![
        Constraint::Length(40), // Art
        Constraint::Min(0),      // Info
    ];

    if inner_area.width < 60 {
        constraints.remove(0);
    }

    let horizontal_chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints(constraints)
        .split(inner_area);

    let info_chunk_index = if inner_area.width < 60 { 0 } else { 1 };

    // Art
    if inner_area.width >= 60 {
        if let Some(art) = &state.album_art_ascii {
            let art_para = Paragraph::new(art.as_str());
            f.render_widget(art_para, horizontal_chunks[0]);
        } else if state.playback.is_some() {
            f.render_widget(Paragraph::new("\n\n   Loading Art...").style(Style::default().fg(state.theme.text_dim)), horizontal_chunks[0]);
        }
    }

    // Info
    let info_chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(1), // Title
            Constraint::Length(1), // Artist
            Constraint::Length(1), // Album
            Constraint::Length(1), // Spacer
            Constraint::Length(1), // Progress
        ])
        .split(horizontal_chunks[info_chunk_index]);

    if let Some(playback) = &state.playback {
        if let Some(item) = &playback.item {
            let (title, artist, album) = match item {
                rspotify::model::PlayableItem::Track(t) => {
                    (t.name.clone(), 
                     t.artists.iter().map(|a| a.name.as_str()).collect::<Vec<_>>().join(", "),
                     t.album.name.clone())
                }
                rspotify::model::PlayableItem::Episode(e) => {
                    (e.name.clone(), e.show.name.clone(), "Podcast".to_string())
                }
            };

            f.render_widget(Paragraph::new(format!(" Title:  {}", title)).style(Style::default().fg(state.theme.text_highlight).add_modifier(Modifier::BOLD)), info_chunks[0]);
            f.render_widget(Paragraph::new(format!(" Artist: {}", artist)).style(Style::default().fg(state.theme.text)), info_chunks[1]);
            f.render_widget(Paragraph::new(format!(" Album:  {}", album)).style(Style::default().fg(state.theme.text_dim)), info_chunks[2]);

            let duration = match item {
                rspotify::model::PlayableItem::Track(t) => t.duration,
                rspotify::model::PlayableItem::Episode(e) => e.duration,
            };
            let progress = playback.progress.unwrap_or_else(|| chrono::Duration::zero());
            let ratio = if duration.num_milliseconds() > 0 {
                (progress.num_milliseconds() as f64 / duration.num_milliseconds() as f64).min(1.0)
            } else {
                0.0
            };

            let gauge = Gauge::default()
                .gauge_style(Style::default().fg(state.theme.progress_bar).bg(state.theme.progress_bar_bg))
                .ratio(ratio)
                .label(format!("{}/{}", format_duration(progress), format_duration(duration)));
            f.render_widget(gauge, info_chunks[4]);
        }
    } else {
        f.render_widget(Paragraph::new(" Not connected to Spotify ").style(Style::default().fg(state.theme.text_dim)), horizontal_chunks[info_chunk_index]);
    }
}

fn format_duration(d: chrono::Duration) -> String {
    let total_secs = d.num_seconds();
    let mins = total_secs / 60;
    let secs = total_secs % 60;
    format!("{:02}:{:02}", mins, secs)
}

fn render_help(f: &mut Frame, state: &AppState) {
    let kb = &state.config.keybindings;
    let help_text = format!("
    [Normal Mode Bindings]
    {}  : Move up
    {}  : Move down
    {}  : Select / Enter
    Tab / S-Tab : Cycle panes
    {}  : Play / Pause
    {} / {} : Next / Previous track
    {} / {} : Volume up / down
    {} / {} : Seek backward / forward
    {}  : Search
    {}  : Command mode
    {}  : Devices pane
    {}  : Queue pane
    {}  : Toggle Help
    {}  : Quit

    [Commands]
    :q                  : Quit
    :logout             : Log out
    :theme <name>       : Switch theme
    :playlist-create <n>: Create playlist
    :playlist-delete    : Delete selected playlist
    :playlist-reorder <start> <before> : Reorder tracks
    ", 
    kb.move_up, kb.move_down, kb.select, kb.play_pause, kb.next_track, kb.prev_track, 
    kb.volume_up, kb.volume_down, kb.seek_backward, kb.seek_forward, kb.search, kb.command, 
    kb.devices, kb.queue, kb.help, kb.quit);

    let area = centered_rect(80, 80, f.area());
    f.render_widget(Clear, area); // This clears the area behind the help modal
    let block = Block::default()
        .title(" Help ")
        .borders(Borders::ALL)
        .border_style(Style::default().fg(state.theme.border_active));
    let paragraph = Paragraph::new(help_text)
        .block(block)
        .style(Style::default().fg(state.theme.text));
    f.render_widget(paragraph, area);
}

fn centered_rect(percent_x: u16, percent_y: u16, r: Rect) -> Rect {
    let popup_layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Percentage((100 - percent_y) / 2),
            Constraint::Percentage(percent_y),
            Constraint::Percentage((100 - percent_y) / 2),
        ])
        .split(r);

    Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage((100 - percent_x) / 2),
            Constraint::Percentage(percent_x),
            Constraint::Percentage((100 - percent_x) / 2),
        ])
        .split(popup_layout[1])[1]
}

fn render_status_bar(f: &mut Frame, area: Rect, state: &AppState) {
    let text = if state.input_mode == InputMode::Command {
        state.command_input.clone()
    } else {
        let mode_str = format!(" {:?} ", state.input_mode);
        let help_str = " [Tab] Cycle  [/] Search  [d] Devices  [Q] Queue  [a] Add to Queue  [q] Quit ";
        format!("{} | {}", mode_str, help_str)
    };
    
    f.render_widget(Paragraph::new(text).style(Style::default().fg(state.theme.status_bar)), area);
}
