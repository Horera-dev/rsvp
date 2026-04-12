mod audio;
mod color;
mod config;
mod constant;
mod content_parser;
mod io;
mod processing;
mod renderer;
mod rsvp;
mod scheduler;
mod spiral;
mod tests;
mod utils;

use std::error::Error;

fn main() -> Result<(), Box<dyn Error>> {
    let config = io::load_config("configuration.toml")?;

    // Schedule is computed once here, shared by both audio and video
    let schedule = processing::build_schedule(&config);

    // Create a closure for the rendering logic so we don't repeat ourselves
    // let render_logic = |stdin: &mut ChildStdin| processing::process_blocks(stdin, &config);

    match config.settings.renderer {
        config::RenderMode::Gif => {
            processing::spawn_ffmpeg_process_gif(&config, &schedule, |stdin, schedule| {
                processing::render_all(stdin, schedule, &config)
            })?
        }
        config::RenderMode::Video => {
            processing::spawn_ffmpeg_process_video(&config, &schedule, |stdin, schedule| {
                processing::render_all(stdin, schedule, &config)
            })?
        }
    }

    println!("✨ Done!");
    Ok(())
}
