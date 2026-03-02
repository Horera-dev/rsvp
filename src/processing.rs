use std::{
    error::Error,
    io::Write,
    process::{ChildStdin, Command, Stdio},
    time::Instant,
};

use ab_glyph::FontRef;
use anyhow::Context;
use image::RgbImage;

use crate::{
    config::Config,
    io, renderer,
    rsvp::generate_random_mask,
    scheduler::{FrameInstruction, compute_padding, compute_schedule, dump_schedule},
    spiral::{self, SpiralCache, create_spiral_cache, wpm_to_tint},
};

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
            "out/output.gif",
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
            "out/output.mp4",
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

pub fn process_blocks(stdin: &mut ChildStdin, config: &Config) -> Result<(), Box<dyn Error>> {
    let font_data = io::load_font_data(config)?;
    let font: FontRef = FontRef::try_from_slice(&font_data)
        .with_context(|| "Font file loaded but corrupted. Or not a valid font file.")?;
    let start_time = Instant::now(); // Start the stopwatch
    let active_config = config.settings.active_format();
    let spiral_cache = create_spiral_cache(active_config.width, active_config.height);

    // Phase 1: figure out what to render
    let mut instructions = compute_schedule(config);
    let padding = compute_padding(config, instructions.len() as u32);
    instructions.extend(padding.instructions);

    let mut elapsed = start_time.elapsed();
    dump_schedule(
        &instructions,
        "out/debug_schedule.txt",
        padding.period,
        padding.remainder,
        elapsed,
    )?;

    // Phase 2: render all instructions
    for instruction in instructions.iter() {
        render_instruction(stdin, instruction, config, &spiral_cache, &font)?;
    }

    elapsed = start_time.elapsed();
    let n = instructions.len() as u32;
    println!("--- Performance Report ---");
    println!(
        "Total: {:?} | Frames: {} | Avg: {:?} | FPS: {:.2}",
        elapsed,
        n,
        elapsed / n,
        1.0 / (elapsed / n).as_secs_f32()
    );
    println!("--- End of Performance Report ---");

    Ok(())
}

fn render_instruction(
    stdin: &mut ChildStdin,
    instruction: &FrameInstruction,
    config: &Config,
    spiral_cache: &SpiralCache,
    font: &FontRef,
) -> Result<(), Box<dyn Error>> {
    let active = config.settings.active_format();

    let mut img = RgbImage::new(active.width, active.height);

    match instruction {
        FrameInstruction::Word {
            time_secs,
            word,
            scale,
            wpm,
        } => {
            let tint = wpm_to_tint(*wpm, &config.spiral);
            spiral::draw_spiral_fast_with_cache(
                &mut img,
                &config.spiral,
                *time_secs,
                spiral_cache,
                tint,
            );
            renderer::draw_word(&mut img, word, *scale, font);
        }
        FrameInstruction::Mask {
            time_secs,
            word_len,
            scale,
            wpm,
        } => {
            let tint = wpm_to_tint(*wpm, &config.spiral);
            spiral::draw_spiral_fast_with_cache(
                &mut img,
                &config.spiral,
                *time_secs,
                spiral_cache,
                tint,
            );
            let mask = generate_random_mask(*word_len); // randomness lives here
            renderer::draw_word(&mut img, &mask, *scale, font);
        }
        FrameInstruction::Padding { time_secs } => {
            let tint = wpm_to_tint(0.0, &config.spiral);
            spiral::draw_spiral_fast_with_cache(
                &mut img,
                &config.spiral,
                *time_secs,
                spiral_cache,
                tint,
            );
            // no text — spiral only
        }
    }

    stdin.write_all(&img.into_raw())?;

    Ok(())
}
