use std::{
    error::Error,
    io::Write,
    process::{ChildStdin, Command, Stdio},
};

use ab_glyph::FontRef;
use image::RgbImage;

use crate::{config::Config, renderer, rsvp};

pub fn spawn_ffmpeg_process_gif(
    config: &Config,
    render_logic: impl FnOnce(&mut ChildStdin) -> Result<(), Box<dyn Error>>,
) -> Result<(), Box<dyn Error>> {
    // 1. First Pass: Save raw data to a temporary high-quality mkv (lossless)
    // This is much faster than encoding a GIF directly.
    let active_config = config.settings.active_format();
    let mut child = Command::new("ffmpeg")
        .args([
            "-y",
            "-f",
            "rawvideo",
            "-pixel_format",
            "rgb24",
            "-video_size",
            &format!("{}x{}", active_config.width, active_config.height),
            "-framerate",
            &active_config.fps.to_string(),
            "-i",
            "-",
            "-c:v",
            "libx264",
            "-crf",
            "0",
            "-preset",
            "ultrafast",
            "temp_buffer.mkv",
        ])
        .stdin(Stdio::piped())
        .spawn()?;

    let mut stdin = child.stdin.take().ok_or("Failed to open stdin")?;
    render_logic(&mut stdin)?; // Run your word-loop here
    drop(stdin);
    child.wait()?;

    // 2. Second Pass: Generate Palette from the intermediate file
    Command::new("ffmpeg")
        .args([
            "-y",
            "-i",
            "temp_buffer.mkv",
            "-vf",
            "palettegen=max_colors=128:stats_mode=diff",
            "-frames:v",
            "1", // Tell FFmpeg we only want ONE frame for the palette
            "-update",
            "1", // Tell the image2 muxer to treat it as a single file
            "palette.png",
        ])
        .status()?;

    // 3. Third Pass: Use Palette to create final GIF
    Command::new("ffmpeg")
        .args([
            "-y",
            "-i",
            "temp_buffer.mkv",
            "-video_size",
            &format!("{}x{}", active_config.width, active_config.height),
            "-framerate",
            &active_config.fps.to_string(),
            "-i",
            "palette.png",
            "-filter_complex",
            "paletteuse=dither=bayer:bayer_scale=5:diff_mode=rectangle",
            "output.gif",
        ])
        .status()?;

    // Cleanup
    let _ = std::fs::remove_file("temp_buffer.mkv");
    let _ = std::fs::remove_file("palette.png");

    println!("Video generated successfully.");
    Ok(())
}

pub fn spawn_ffmpeg_process_video(
    config: &Config,
    render_logic: impl FnOnce(&mut ChildStdin) -> Result<(), Box<dyn Error>>,
) -> Result<(), Box<dyn Error>> {
    let active_config = config.settings.active_format();
    let mut child = Command::new("ffmpeg")
        .args([
            "-y",
            "-f",
            "rawvideo",
            "-pixel_format",
            "rgb24",
            "-video_size",
            &format!("{}x{}", active_config.width, active_config.height),
            "-framerate",
            &active_config.fps.to_string(),
            "-i",
            "-",
            "-c:v",
            "libx264",
            "-pix_fmt",
            "yuv420p",
            "output.mp4",
        ])
        .stdin(Stdio::piped())
        .spawn()?;

    let mut stdin = child.stdin.take().ok_or("Failed to open stdin")?;
    render_logic(&mut stdin)?; // Run your word-loop here

    drop(stdin);
    child.wait()?;

    println!("Video generated successfully.");
    Ok(())
}

pub fn process_blocks(
    stdin: &mut ChildStdin,
    config: &Config,
    font: &FontRef,
) -> Result<(), Box<dyn Error>> {
    let active_config = config.settings.active_format();
    let mut frame_count = 0;
    for block in &config.blocks {
        let words: Vec<&str> = block.text.split_whitespace().collect();
        let scale_to_use = block.get_scale(active_config.scale);
        let easing_to_use = block.get_easing(active_config.easing.clone());

        // We use a float to track fractional frames to prevent "micro-stutters"
        let mut fractional_frames_buffer: f32 = 0.0;

        for (i, word) in words.iter().enumerate() {
            // Compute the speed for this specific word
            let progress = rsvp::compute_progress(words.len(), i);
            let current_wpm =
                rsvp::apply_easing(&easing_to_use, block.wpm_from, block.wpm_to, progress);

            // Convert WPM to Frames
            // Calculation: (60 sec / WPM) * FPS * bonus
            let word_duration_base = (active_config.fps / current_wpm) * active_config.fps;
            let word_duration_weighted = word_duration_base * rsvp::apply_punctuation(word);

            // Handle frame distribution
            // We add the fractional remainder from the last word to the current word
            let total_frames_raw = word_duration_weighted + fractional_frames_buffer;
            let frames_to_render = total_frames_raw.round();
            fractional_frames_buffer = total_frames_raw - frames_to_render;

            if frames_to_render < 1.0 {
                continue;
            }

            // Render
            let cleaned_word = rsvp::clean_word(word);

            // Pipe
            for _ in 0..(frames_to_render as u32) {
                let mut img = RgbImage::new(active_config.width, active_config.height);
                renderer::draw_spiral(&mut img, &config.spiral, frame_count, active_config.fps);
                renderer::draw_word(&mut img, cleaned_word, scale_to_use, font);
                let frame_data = img.into_raw();
                stdin.write_all(&frame_data)?;
                frame_count += 1;
            }
        }
    }

    Ok(())
}
