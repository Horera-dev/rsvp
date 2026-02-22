use std::{
    io::Write,
    process::{Command, Stdio},
};

use ab_glyph::FontRef;

use crate::{config::Config, renderer, rsvp::determine_frame_duration};

/// Sets up the FFmpeg Command with the necessary pipe arguments
pub fn spawn_ffmpeg_process(config: &Config) -> Result<std::process::Child, std::io::Error> {
    Command::new("ffmpeg")
        .args([
            "-y",
            "-f",
            "rawvideo",
            "-pixel_format",
            "rgb24",
            "-video_size",
            &format!("{}x{}", config.width, config.height),
            "-framerate",
            &config.fps.to_string(),
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

/// Handles the core RSVP logic: iterating words and writing to the pipe
pub fn process_phrase_to_pipe(
    stdin: &mut std::process::ChildStdin,
    config: &Config,
    font: &FontRef,
) -> Result<(), Box<dyn std::error::Error>> {
    let base_frames = config.frames_per_word();

    for word in config.phrase.split_whitespace() {
        let clean_word = word.trim_matches(|c: char| !c.is_alphanumeric());

        // Duration logic
        let adjusted_frame_len = determine_frame_duration(clean_word, base_frames);

        // Render the word
        let frame_data =
            renderer::draw_word_to_frame(clean_word, config.width, config.height, 100.0, font);

        // Write to pipe
        for _ in 0..adjusted_frame_len {
            stdin.write_all(&frame_data)?;
        }
    }
    Ok(())
}

/// Final status reporting
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
