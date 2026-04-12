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
    audio::generate_and_write_wav,
    config::Config,
    io, renderer,
    rsvp::generate_random_mask,
    scheduler::{
        AudioInstruction, FrameInstruction, Schedule, compute_padding, compute_schedule,
        dump_schedule,
    },
    spiral::{self, SpiralCache, create_spiral_cache, wpm_to_tint},
    utils,
};

pub fn spawn_ffmpeg_process_gif(
    config: &Config,
    schedule: &Schedule,
    render_logic: impl FnOnce(&mut ChildStdin, &Schedule) -> Result<(), Box<dyn Error>>,
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
    render_logic(&mut stdin, schedule)?; // Run your word-loop here
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
    schedule: &Schedule,
    render_logic: impl FnOnce(&mut ChildStdin, &Schedule) -> Result<(), Box<dyn Error>>,
) -> Result<(), Box<dyn Error>> {
    let active_config = config.settings.active_format();
    let sample_rate = config.settings.binaural.sample_rate;

    // Generate audio before spawning ffmpeg
    let audio_path = std::path::Path::new("out/rsvp_audio.wav");
    generate_and_write_wav(audio_path, &schedule.audio, active_config.fps, sample_rate)?;
    println!("Audio written to {:?} ", audio_path);

    let mut child = Command::new("ffmpeg")
        .args([
            "-y",
            "-thread_queue_size",
            "512",
            // video input from stdin pipe
            "-f",
            "rawvideo",
            "-pixel_format",
            "rgb24",
            "-video_size",
            &format!("{}x{}", active_config.width, active_config.height),
            "-framerate",
            &active_config.fps.to_string(),
            "-i",
            "pipe:0",
            "-i",
            audio_path.to_str().unwrap(),
            // output
            "-c:v",
            "libx265",
            "-crf",
            "28", // try 28-32, spiral compression artifacts are barely visible
            "-preset",
            "slow", // slower encode = better compression, same quality
            "-c:a",
            "aac",
            "-b:a",
            "64k", // half the default, tones are simple signals
            "-pix_fmt",
            "yuv420p",
            "-shortest",
            "-movflags",
            "+faststart", // moves metadata to front — better for download/streaming
            "out/output.mp4",
        ])
        .stdin(Stdio::piped())
        .spawn()?;

    let mut stdin = child.stdin.take().ok_or("Failed to open stdin")?;
    render_logic(&mut stdin, schedule)?;

    drop(stdin);
    child.wait()?;

    std::fs::remove_file(audio_path).ok();
    println!("Video generated successfully.");

    Ok(())
}

/// Phase 1: pure computation, no I/O
pub fn build_schedule(config: &Config) -> Schedule {
    let mut schedule = compute_schedule(config);
    let padding = compute_padding(config, schedule.video.len() as u32);
    schedule.video.extend(padding.instructions);
    // pad audio to match
    for _ in 0..padding.remainder {
        schedule.audio.push(AudioInstruction::Silence);
    }
    schedule
}

/// Phase 2: pure rendering, no schedule logic
pub fn render_all(
    stdin: &mut ChildStdin,
    schedule: &Schedule,
    config: &Config,
) -> Result<(), Box<dyn Error>> {
    let font_data = io::load_font_data(config)?;
    let font = FontRef::try_from_slice(&font_data)
        .context("Font file loaded but corrupted or invalid.")?;
    let active = config.settings.active_format();
    let spiral_cache = create_spiral_cache(active.width, active.height);
    let start = Instant::now();

    let padding = compute_padding(config, schedule.video.len() as u32);

    let mut elapsed = start.elapsed();
    dump_schedule(
        &schedule.video,
        "out/debug_schedule.txt",
        padding.period,
        padding.remainder,
        elapsed,
    )?;

    for instruction in &schedule.video {
        let img = render_instruction(instruction, config, &spiral_cache, &font)?;
        stdin.write_all(&img.into_raw())?;
    }

    elapsed = start.elapsed();
    let n = schedule.video.len() as u32;
    println!(
        "Total: {:?} | Frames: {} | Avg: {:?} | FPS: {:.2}",
        elapsed,
        n,
        elapsed / n,
        1.0 / (elapsed / n).as_secs_f32()
    );
    Ok(())
}

pub fn render_instruction(
    instruction: &FrameInstruction,
    config: &Config,
    spiral_cache: &SpiralCache,
    font: &FontRef,
) -> Result<image::ImageBuffer<image::Rgb<u8>, Vec<u8>>, Box<dyn Error>> {
    let active = config.settings.active_format();

    // let test_glyph = font.glyph_id('C').with_scale(48.0);
    // console_log!("Font Loaded! 'A' glyph id: {:?}", test_glyph);

    let mut img: image::ImageBuffer<image::Rgb<u8>, Vec<u8>> =
        RgbImage::new(active.width, active.height);

    // Every frame has a spiral — only the tint and overlay differ
    let draw_spiral = |img: &mut RgbImage, time_secs: f32, wpm: f32| {
        let tint = wpm_to_tint(wpm, &config.spiral);
        spiral::draw_spiral_fast_with_cache(img, &config.spiral, time_secs, spiral_cache, tint);
    };

    match instruction {
        FrameInstruction::Word {
            time_secs,
            word,
            scale,
            wpm,
        } => {
            draw_spiral(&mut img, *time_secs, *wpm);
            renderer::draw_word(&mut img, word, *scale, font);
        }
        FrameInstruction::Mask {
            time_secs,
            word_len,
            scale,
            wpm,
        } => {
            draw_spiral(&mut img, *time_secs, *wpm);
            let mask = generate_random_mask(*word_len); // randomness lives here
            renderer::draw_word(&mut img, &mask, *scale, font);
        }
        FrameInstruction::Padding { time_secs } => {
            draw_spiral(&mut img, *time_secs, 0.0);
        }
        FrameInstruction::FlashWhite {
            time_secs,
            word,
            scale,
            settings,
            wpm: _,
        } => {
            draw_spiral(&mut img, *time_secs, 0.0);
            renderer::wash_to_background(&mut img, settings.bg_color, 1.0); // instant cut to white
            renderer::draw_word_colored(&mut img, word, *scale, font, settings.accent_color);
        }
        FrameInstruction::FlashFade {
            time_secs,
            word,
            scale,
            settings,
            fade_t,
            wpm,
        } => {
            draw_spiral(&mut img, *time_secs, *wpm);
            let amount = utils::smoothstep(1.0 - fade_t);
            renderer::wash_to_background(&mut img, settings.bg_color, amount); // fades as fade_t → 1.0
            renderer::draw_word_colored(&mut img, word, *scale, font, settings.accent_color);
        }
    }

    Ok(img)
}
