mod config;
mod io;
mod processing;
mod renderer;
mod rsvp;
mod spiral;
mod tests;

use std::{error::Error, process::ChildStdin};

use ab_glyph::FontRef;
use anyhow::Context;
use image::RgbImage;

use crate::{
    io::{load_config, load_font_data},
    processing::{process_blocks, spawn_ffmpeg_process_gif, spawn_ffmpeg_process_video},
};

fn draw_basic() {
    // Load Font (Include bytes at compile time for simplicity)
    let font_data = include_bytes!("../assets/Roboto-Black.ttf");
    let font = FontRef::try_from_slice(font_data).expect("Error loading font");
    let mut img = RgbImage::new(1920, 1280);
    renderer::draw_word(&mut img, "Rust", 100.0, &font);
    img.save("test_frame.png").unwrap();
}

fn main() -> Result<(), Box<dyn Error>> {
    // Test
    draw_basic();

    let config = load_config("configuration.toml")?;
    let font_data = load_font_data(&config)?;
    let font: FontRef = FontRef::try_from_slice(&font_data)
        .with_context(|| "Font file loaded but corrupted. Or not a valid font file.")?;

    // Create a closure for the rendering logic so we don't repeat ourselves
    let render_logic = |stdin: &mut ChildStdin| process_blocks(stdin, &config, &font);

    match config.settings.renderer {
        config::RenderMode::Gif => spawn_ffmpeg_process_gif(&config, render_logic)?,
        config::RenderMode::Video => spawn_ffmpeg_process_video(&config, render_logic)?,
    }

    println!("✨ Done!");
    Ok(())
}
