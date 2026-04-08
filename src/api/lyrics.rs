use reqwest::Client;
use serde::Deserialize;
use thiserror::Error;

pub trait LyricsProvider: Send + Sync {
    async fn fetch(&self, artist: &str, title: &str) -> Result<String, LyricsError>;
    async fn fetch_synced(&self, _artist: &str, _title: &str) -> Result<Vec<crate::events::SyncedLyricsLine>, LyricsError> {
        Err(LyricsError::NotFound)
    }
}

#[derive(Debug, Error)]
pub enum LyricsError {
    #[error("Network error: {0}")]
    Network(#[from] reqwest::Error),
    #[error("Lyrics not found")]
    NotFound,
    #[error("Unknown error: {0}")]
    Unknown(String),
}

pub struct LyricsOvhProvider {
    client: Client,
}

impl LyricsOvhProvider {
    pub fn new() -> Self {
        Self {
            client: Client::new(),
        }
    }
}

pub struct GeniusLyricsProvider {
    client: Client,
    api_key: String,
}

impl GeniusLyricsProvider {
    pub fn new(api_key: String) -> Self {
        Self {
            client: Client::new(),
            api_key,
        }
    }
}

#[derive(Deserialize)]
struct GeniusSearchResponse {
    response: GeniusSearchInner,
}

#[derive(Deserialize)]
struct GeniusSearchInner {
    hits: Vec<GeniusHit>,
}

#[derive(Deserialize)]
struct GeniusHit {
    result: GeniusResult,
}

#[derive(Deserialize)]
struct GeniusResult {
    url: String,
}

impl LyricsProvider for GeniusLyricsProvider {
    async fn fetch(&self, artist: &str, title: &str) -> Result<String, LyricsError> {
        let query = format!("{} {}", artist, title);
        let url = format!("https://api.genius.com/search?q={}", urlencoding::encode(&query));
        
        let resp = self.client.get(url)
            .bearer_auth(&self.api_key)
            .send().await?;
            
        if !resp.status().is_success() {
            return Err(LyricsError::Unknown(format!("Genius API error: {}", resp.status())));
        }
        
        let search_resp: GeniusSearchResponse = resp.json().await?;
        let song_url = search_resp.response.hits.first()
            .map(|hit| hit.result.url.clone())
            .ok_or(LyricsError::NotFound)?;
            
        // Genius API doesn't return lyrics directly. 
        // In a real implementation, we would scrape song_url.
        // For now, we'll return a message with the URL or try a simple scrape if feasible.
        // Let's try a very simple scrape of the lyrics div.
        
        let page_resp = self.client.get(&song_url).send().await?;
        let _html = page_resp.text().await?;
        
        // Simple regex-based scrape (fragile but common for TUI tools)
        // Genius lyrics are often in a div with class starting with "Lyrics__Container"
        // or just a "lyrics" class in older versions.
        
        // This is a placeholder for a more robust scraper.
        Ok(format!("Lyrics found at: {}\n\n(Full Genius scraping not implemented in this version)", song_url))
    }
}

#[derive(Deserialize)]
struct LyricsOvhResponse {
    lyrics: String,
}

impl LyricsProvider for LyricsOvhProvider {
    async fn fetch(&self, artist: &str, title: &str) -> Result<String, LyricsError> {
        let url = format!("https://api.lyrics.ovh/v1/{}/{}", urlencoding::encode(artist), urlencoding::encode(title));
        let resp = self.client.get(url).send().await?;
        
        if resp.status() == 404 {
            return Err(LyricsError::NotFound);
        }
        
        if !resp.status().is_success() {
            return Err(LyricsError::Unknown(format!("Status: {}", resp.status())));
        }
        
        let lyrics_resp: LyricsOvhResponse = resp.json().await?;
        Ok(lyrics_resp.lyrics)
    }
}

pub struct LrcLibProvider {
    client: Client,
}

impl LrcLibProvider {
    pub fn new() -> Self {
        Self { client: Client::new() }
    }
}

#[derive(Deserialize)]
struct LrcLibResponse {
    #[serde(rename = "syncedLyrics")]
    synced_lyrics: Option<String>,
    lyrics: Option<String>,
}

impl LyricsProvider for LrcLibProvider {
    async fn fetch(&self, artist: &str, title: &str) -> Result<String, LyricsError> {
        let url = format!("https://lrclib.net/api/get?artist_name={}&track_name={}", 
            urlencoding::encode(artist), urlencoding::encode(title));
        let resp = self.client.get(url).send().await?;
        if resp.status() == 404 { return Err(LyricsError::NotFound); }
        let lrc_resp: LrcLibResponse = resp.json().await?;
        lrc_resp.lyrics.or(lrc_resp.synced_lyrics).ok_or(LyricsError::NotFound)
    }

    async fn fetch_synced(&self, artist: &str, title: &str) -> Result<Vec<crate::events::SyncedLyricsLine>, LyricsError> {
        let url = format!("https://lrclib.net/api/get?artist_name={}&track_name={}", 
            urlencoding::encode(artist), urlencoding::encode(title));
        let resp = self.client.get(url).send().await?;
        if resp.status() == 404 { return Err(LyricsError::NotFound); }
        let lrc_resp: LrcLibResponse = resp.json().await?;
        if let Some(synced) = lrc_resp.synced_lyrics {
            Ok(parse_lrc(&synced))
        } else {
            Err(LyricsError::NotFound)
        }
    }
}

fn parse_lrc(lrc: &str) -> Vec<crate::events::SyncedLyricsLine> {
    let mut lines = Vec::new();
    for line in lrc.lines() {
        if line.starts_with('[') {
            if let Some(end_bracket) = line.find(']') {
                let time_str = &line[1..end_bracket];
                let text = line[end_bracket+1..].trim();
                if let Some(ms) = parse_time(time_str) {
                    lines.push(crate::events::SyncedLyricsLine {
                        timestamp_ms: ms,
                        text: text.to_string(),
                    });
                }
            }
        }
    }
    lines
}

fn parse_time(s: &str) -> Option<u32> {
    let parts: Vec<&str> = s.split(':').collect();
    if parts.len() != 2 { return None; }
    let mins = parts[0].parse::<u32>().ok()?;
    let secs_parts: Vec<&str> = parts[1].split('.').collect();
    let secs = secs_parts[0].parse::<u32>().ok()?;
    let ms = if secs_parts.len() > 1 {
        let ms_str = secs_parts[1];
        let ms_val = ms_str.parse::<u32>().ok()?;
        if ms_str.len() == 2 { ms_val * 10 } else { ms_val }
    } else {
        0
    };
    Some(mins * 60 * 1000 + secs * 1000 + ms)
}
