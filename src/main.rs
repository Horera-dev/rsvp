mod config;
mod io;
mod processing;
mod renderer;
mod rsvp;
mod spiral;
mod tests;

use std::{error::Error, process::ChildStdin};

fn main() -> Result<(), Box<dyn Error>> {
    let config = io::load_config("configuration.toml")?;

    // Create a closure for the rendering logic so we don't repeat ourselves
    let render_logic = |stdin: &mut ChildStdin| processing::process_blocks(stdin, &config);

    match config.settings.renderer {
        config::RenderMode::Gif => processing::spawn_ffmpeg_process_gif(&config, render_logic)?,
        config::RenderMode::Video => processing::spawn_ffmpeg_process_video(&config, render_logic)?,
    }

    println!("✨ Done!");
    Ok(())
}
