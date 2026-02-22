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
    const PUNCTUATION: [char; 5] = ['.', '!', '?', ';', ','];

    for block in &config.blocks {
        let words: Vec<&str> = block.text.split_whitespace().collect();
        let scale_to_use = block.get_scale(config.settings.scale);

        // 1. Calculate the total "Time-Weight" of the block.
        // Instead of just characters, we account for speed (WPM) at each step.
        let mut total_time_weight: f32 = 0.0;
        let mut word_weights = Vec::with_capacity(words.len());

        for (i, word) in words.iter().enumerate() {
            let progress = i as f32 / words.len() as f32;
            let current_wpm = get_current_wpm(block, progress);

            // Weight = Length / Speed.
            // Punctuation bonus: slightly more weight for sentence ends
            let punctuation_bonus = if word.ends_with(PUNCTUATION) {
                1.3
            } else {
                1.0
            };

            // A long word at slow speed has a huge weight.
            // A short word at high speed has a tiny weight.
            let weight = (word.len() as f32 * punctuation_bonus) / current_wpm;
            word_weights.push(weight);
            total_time_weight += weight;
        }

        // 2. Determine total frame pool
        let total_duration_secs = block.duration_ms as f32 / 1000.0;
        let total_frames = (total_duration_secs * config.settings.fps) as u32;

        let mut cumulative_weight = 0.0;
        let mut frames_already_piped = 0;

        for (i, word) in words.iter().enumerate() {
            cumulative_weight += word_weights[i];

            // Instead of rounding word-by-word, we calculate where we SHOULD be
            // in the total timeline at the end of this word.
            let target_total_frames =
                ((cumulative_weight / total_time_weight) * total_frames as f32).round() as u32;

            // The frames for THIS word is the difference
            let word_frames = target_total_frames - frames_already_piped;
            frames_already_piped += word_frames;

            if word_frames == 0 {
                continue;
            }

            // Render and Pipe
            let frame_data = renderer::draw_word_to_frame(
                word,
                config.settings.width,
                config.settings.height,
                scale_to_use,
                font,
            );

            for _ in 0..word_frames {
                stdin.write_all(&frame_data)?;
            }
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
