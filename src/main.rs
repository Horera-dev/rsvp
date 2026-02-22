mod config;
mod processing;
mod renderer;
mod rsvp;

use ab_glyph::FontRef;

use crate::{
    config::Config,
    processing::{handle_completion, process_phrase_to_pipe, spawn_ffmpeg_process},
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

    // --- Configuration ---
    let config = Config {
        width: 1920,
        height: 1280,
        fps: 60.0,
        wpm: 200.0,
        phrase: "Rust is a multi-paradigm, general-purpose programming language. It is designed for performance and safety.".to_string(),
    };

    // Load Font (Include bytes at compile time for simplicity)
    let font_data = include_bytes!("../assets/Roboto-Black.ttf");
    let font = FontRef::try_from_slice(font_data).expect("Error loading font");

    // 1. Spawn FFmpeg process
    let mut child = spawn_ffmpeg_process(&config)?;
    let mut stdin = child.stdin.take().ok_or("Failed to open stdin")?;

    // 4. Processing Loop
    // We use a block here to ensure stdin is dropped (closed)
    // before we wait, otherwise ffmpeg will wait forever for more data.
    let render_result = process_phrase_to_pipe(&mut stdin, &config, &font);

    // 3. Cleanup: Close stdin and wait for the process to finish
    drop(stdin);
    let status = child.wait()?;

    handle_completion(status, render_result)
}
