use std::{
    error::Error,
    io::Write,
    process::{ChildStdin, Command, Stdio},
};

use ab_glyph::FontRef;

use crate::{config::Config, renderer, rsvp::get_current_wpm};

pub fn spawn_ffmpeg_process(config: &Config) -> Result<std::process::Child, std::io::Error> {
    Command::new("ffmpeg")
        .args([
            "-y",
            "-f",
            "rawvideo",
            "-pixel_format",
            "rgb24",
            "-video_size",
            &format!("{}x{}", config.settings.width, config.settings.height),
            "-framerate",
            &config.settings.fps.to_string(),
            "-i",
            "-",
            "-c:v",
            "libx264",
            "-pix_fmt",
            "yuv420p",
            "output.mp4",
        ])
        .stdin(Stdio::piped())
        .spawn()
}

pub fn process_blocks(
    stdin: &mut ChildStdin,
    config: &Config,
    font: &FontRef,
) -> Result<(), Box<dyn Error>> {
    for block in &config.blocks {
        let words: Vec<&str> = block.text.split_whitespace().collect();
        let mut block_elapsed_ms = 0.0;

        for word in words {
            // 1. Calculate speed for THIS word
            let current_wpm = get_current_wpm(block, block_elapsed_ms);

            // 2. Convert WPM to Frames
            // frames = (seconds_per_word) * fps
            let frames = ((60.0 / current_wpm) * config.settings.fps) as u32;

            // 3. Render and Pipe
            let frame_data = renderer::draw_word_to_frame(
                word,
                config.settings.width,
                config.settings.height,
                100.0,
                font,
            );
            for _ in 0..frames {
                stdin.write_all(&frame_data)?;
            }

            // 4. Update elapsed time for the next word
            let word_duration_ms = (60.0 / current_wpm) * 1000.0;
            block_elapsed_ms += word_duration_ms;
        }
    }
    Ok(())
}

pub fn handle_completion(
    status: std::process::ExitStatus,
    render_result: Result<(), Box<dyn std::error::Error>>,
) -> Result<(), Box<dyn std::error::Error>> {
    if !status.success() {
        eprintln!("FFmpeg exited with an error status: {}", status);
    }

    render_result?; // Propagate any pipe errors

    println!("Video generated successfully.");
    Ok(())
}
