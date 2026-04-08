use image::{GenericImageView, DynamicImage, imageops::FilterType};
use crate::events::ArtPayload;
use reqwest::Client;
use std::error::Error;

pub async fn fetch_and_process_art(url: &str, client: &Client, width: u32, height: u32, colored: bool) -> Result<ArtPayload, Box<dyn Error + Send + Sync>> {
    let bytes = client.get(url).send().await?.bytes().await?;
    let img = image::load_from_memory(&bytes)?;
    
    // ASCII conversion (default)
    let ascii = convert_to_ascii(&img, width, height, colored);
    Ok(ArtPayload::Ascii(ascii))
}

fn convert_to_ascii(img: &DynamicImage, width: u32, height: u32, colored: bool) -> String {
    let resized = img.resize_exact(width, height, FilterType::Nearest);
    let mut ascii = String::new();
    let ramp = " .:-=+*#%@";
    
    for y in 0..height {
        for x in 0..width {
            let pixel = resized.get_pixel(x, y);
            let r = pixel[0];
            let g = pixel[1];
            let b = pixel[2];
            let luminance = 0.2126 * r as f32 + 0.7152 * g as f32 + 0.0722 * b as f32;
            let index = (luminance / 255.0 * (ramp.len() - 1) as f32).clamp(0.0, (ramp.len() - 1) as f32) as usize;
            let char = ramp.chars().nth(index).unwrap_or(' ');
            
            if colored {
                // ANSI truecolor: \x1b[38;2;R;G;Bm
                ascii.push_str(&format!("\x1b[38;2;{};{};{}m{}", r, g, b, char));
            } else {
                ascii.push(char);
            }
        }
        if colored {
            ascii.push_str("\x1b[0m\n");
        } else {
            ascii.push('\n');
        }
    }
    ascii
}
