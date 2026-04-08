use std::sync::mpsc;
use tiny_http::{Response, Server};
use url::Url;
use keyring::Entry;
use rspotify::Token;
use serde_json;

pub struct CallbackServer {
    pub port: u16,
}

pub fn store_token(token: &Token) -> color_eyre::Result<()> {
    let entry = Entry::new("spotui", "spotify_token")?;
    let json = serde_json::to_string(token)?;
    entry.set_password(&json)?;
    Ok(())
}

pub fn load_token() -> Option<Token> {
    let entry = Entry::new("spotui", "spotify_token").ok()?;
    let json = entry.get_password().ok()?;
    serde_json::from_str(&json).ok()
}

pub fn clear_token() -> color_eyre::Result<()> {
    let entry = Entry::new("spotui", "spotify_token")?;
    entry.delete_credential()?;
    Ok(())
}

impl CallbackServer {
    pub fn new(port: u16) -> Self {
        Self { port }
    }

    pub fn start(&self) -> mpsc::Receiver<String> {
        let (tx, rx) = mpsc::channel();
        let addr = format!("127.0.0.1:{}", self.port);
        let server = Server::http(addr).expect("Could not start callback server");

        std::thread::spawn(move || {
            for request in server.incoming_requests() {
                let url = format!("http://localhost{}", request.url());
                let parsed_url = Url::parse(&url);
                
                if let Ok(parsed) = parsed_url {
                    let mut pairs = parsed.query_pairs();
                    let code = pairs
                        .find(|(key, _)| key == "code")
                        .map(|(_, value)| value.into_owned());
                    
                    if let Some(c) = code {
                        let response = Response::from_string("Authentication successful! You can close this window now.");
                        let _ = request.respond(response);
                        let _ = tx.send(c);
                        break;
                    }
                }
                
                let response = Response::from_string("Waiting for Spotify authorization...");
                let _ = request.respond(response);
            }
        });

        rx
    }
}
