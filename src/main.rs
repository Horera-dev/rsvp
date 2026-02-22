mod config;
mod io;
mod processing;
mod renderer;
mod rsvp;

use ab_glyph::FontRef;
use anyhow::Context;

use crate::{
    io::{load_config, load_font_data},
    processing::{handle_completion, process_blocks, spawn_ffmpeg_process},
};

fn draw_basic() {
    // Load Font (Include bytes at compile time for simplicity)
    let font_data = include_bytes!("../assets/Roboto-Black.ttf");
    let font = FontRef::try_from_slice(font_data).expect("Error loading font");
    let frame_bytes = renderer::draw_word_to_frame("Rust", 1920, 1280, 100.0, &font);
    let img = image::RgbImage::from_raw(1920, 1280, frame_bytes).unwrap();
    img.save("test_frame.png").unwrap();
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Test
    draw_basic();

    let config = load_config("configuration.toml")?;
    let font_data = load_font_data(&config)?;
    let font: FontRef = FontRef::try_from_slice(&font_data)
        .with_context(|| "Font file loaded but corrupted. Or not a valid font file.")?;
    let mut child = spawn_ffmpeg_process(&config)?;
    let mut stdin = child.stdin.take().ok_or("Failed to open stdin")?;
    let render_result = process_blocks(&mut stdin, &config, &font);
    handle_completion(&mut child, stdin, render_result)
}
