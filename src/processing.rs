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
    rsvp::{self, generate_random_mask},
    spiral::{self, create_spiral_cache},
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
    let fps = active_config.fps;
    let spiral_cache = create_spiral_cache(active_config.width, active_config.height);
    let mut frame_count = 0;
    for block in &config.blocks {
        let words: Vec<&str> = block.text.split_whitespace().collect();
        let scale = block.get_scale(active_config.scale);
        let easing = block.get_easing(active_config.easing.clone());

        // We use a float to track fractional frames to prevent "micro-stutters"
        let mut fractional_frames_buffer: f32 = 0.0;

        for (i, word) in words.iter().enumerate() {
            // Compute the speed for this specific word
            let progress = rsvp::compute_progress(words.len(), i);
            let current_wpm = rsvp::apply_easing(&easing, block.wpm_from, block.wpm_to, progress);

            // Convert WPM to Frames
            // Calculation: (60 sec / WPM) * FPS * bonus
            let word_duration_base = (60.0 / current_wpm) * active_config.fps;
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

            let word_frames = frames_to_render as u32 - config.settings.masking_frames;
            for _ in 0..word_frames {
                let frame_start = Instant::now(); // Timer for a single frame

                let mut img = RgbImage::new(active_config.width, active_config.height);
                spiral::draw_spiral_fast_with_cache(
                    &mut img,
                    &config.spiral,
                    frame_count,
                    fps,
                    &spiral_cache,
                );
                renderer::draw_word(&mut img, cleaned_word, scale, &font);
                let frame_data = img.into_raw();
                stdin.write_all(&frame_data)?;
                frame_count += 1;

                // Optional: Log every 100 frames to see if performance degrades
                if frame_count % 100 == 0 {
                    let elapsed = frame_start.elapsed();
                    println!("Frame {} took: {:?}", frame_count, elapsed);
                }
            }

            for _ in 0..config.settings.masking_frames {
                let mut img = RgbImage::new(active_config.width, active_config.height);
                spiral::draw_spiral_fast_with_cache(
                    &mut img,
                    &config.spiral,
                    frame_count,
                    fps,
                    &spiral_cache,
                );
                let mask = generate_random_mask(word.len());
                renderer::draw_word(&mut img, &mask, scale, &font);
                let frame_data = img.into_raw();
                stdin.write_all(&frame_data)?;
                frame_count += 1;
                // println!("{}", frame_count);
            }
        }
    }

    // If we are working with a gif, we want to make a seamless loop.
    // So we pad the end with spiraly frames
    // Function we use is arctan, that goes from -1 to 1, so it's a range of two.
    // Our rotation offset is multiplied by FPS.
    let padding = ((2.0 * fps) / (config.spiral.speed)).round() as u32;
    println!("padding:{}", padding);
    let remainder = padding - (frame_count % padding);
    println!("remainder:{}", remainder);

    for _ in 0..remainder {
        let mut img = RgbImage::new(active_config.width, active_config.height);
        spiral::draw_spiral_fast_with_cache(
            &mut img,
            &config.spiral,
            frame_count,
            fps,
            &spiral_cache,
        );

        let mut img_path = String::from("out/frame_");
        img_path.push_str(frame_count.to_string().as_str());
        img_path.push_str("_padding.jpg");
        img.save(img_path).unwrap();

        let frame_data = img.into_raw();
        stdin.write_all(&frame_data)?;
        frame_count += 1;
    }

    let total_elapsed = start_time.elapsed();
    let avg_per_frame = total_elapsed / frame_count;

    println!("--- Performance Report ---");
    println!("Total Render Time: {:?}", total_elapsed);
    println!("Total Frames: {}", frame_count);
    println!("Average Time per Frame: {:?}", avg_per_frame);
    println!("Effective FPS: {:.2}", 1.0 / avg_per_frame.as_secs_f32());
    println!("--- End of Performance Report ---");

    Ok(())
}
