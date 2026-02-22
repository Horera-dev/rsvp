use std::{
    error::Error,
    io::Write,
    process::{Child, ChildStdin, Command, Stdio},
};

use ab_glyph::FontRef;

use crate::{
    config::Config,
    renderer,
    rsvp::{apply_easing_wpm, apply_punctuation, clean_word, compute_progress},
};

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
        let scale_to_use = block.get_scale(config.settings.scale);

        // We use a float to track fractional frames to prevent "micro-stutters"
        let mut fractional_frames_buffer: f32 = 0.0;

        for (i, word) in words.iter().enumerate() {
            // Compute the speed for this specific word
            let progress = compute_progress(words.len(), i);
            let current_wpm = apply_easing_wpm(block, progress);

            // Convert WPM to Frames
            // Calculation: (60 sec / WPM) * FPS * bonus
            let word_duration_base = (config.settings.fps / current_wpm) * config.settings.fps;
            let word_duration_weighted = word_duration_base * apply_punctuation(word);

            // Handle frame distribution
            // We add the fractional remainder from the last word to the current word
            let total_frames_raw = word_duration_weighted + fractional_frames_buffer;
            let frames_to_render = total_frames_raw.round();
            fractional_frames_buffer = total_frames_raw - frames_to_render;

            if frames_to_render < 1.0 {
                continue;
            }

            // Render
            let cleaned_word = clean_word(word);
            let frame_data = renderer::draw_word_to_frame(
                cleaned_word,
                config.settings.width,
                config.settings.height,
                scale_to_use,
                font,
            );

            // Pipe
            for _ in 0..(frames_to_render as u32) {
                stdin.write_all(&frame_data)?;
            }
        }
    }

    Ok(())
}

pub fn handle_completion(
    child: &mut Child,
    stdin: ChildStdin,
    render_result: Result<(), Box<dyn std::error::Error>>,
) -> Result<(), Box<dyn std::error::Error>> {
    // Cleanup: Close stdin and wait for the process to finish
    drop(stdin);
    let status = child.wait()?;

    if !status.success() {
        eprintln!("FFmpeg exited with an error status: {}", status);
    }

    render_result?; // Propagate any pipe errors

    println!("Video generated successfully.");
    Ok(())
}
