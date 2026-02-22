mod renderer;
mod rsvp;

use std::{
    io::Write,
    process::{Command, Stdio},
};

use ab_glyph::FontRef;

fn draw_basic() {
    // Load Font (Include bytes at compile time for simplicity)
    let font_data = include_bytes!("../assets/Roboto-Black.ttf");
    let font = FontRef::try_from_slice(font_data).expect("Error loading font");
    let frame_bytes = renderer::draw_word_to_frame("Rust", 1920, 1280, 100.0, &font);
    let img = image::RgbImage::from_raw(1920, 1280, frame_bytes).unwrap();
    img.save("test_frame.png").unwrap();
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Test
    draw_basic();

    // --- Configuration ---
    let phrase = "Rust is a multi-paradigm, general-purpose programming language. 
                  It is designed for performance and safety, especially safe concurrency.";
    let width = 1920;
    let height = 1280;
    let fps = 60.0;
    let wpm = 200.0;
    let frames_per_word = (60.0 / wpm * fps) as u32;

    // Load Font (Include bytes at compile time for simplicity)
    let font_data = include_bytes!("../assets/Roboto-Black.ttf");
    let font = FontRef::try_from_slice(font_data).expect("Error loading font");

    // 1. Spawn FFmpeg process
    let mut child = Command::new("ffmpeg")
        .args([
            "-y", // Overwrite output file
            "-f",
            "rawvideo", // Input format is raw pixels
            "-pixel_format",
            "rgb24",
            "-video_size",
            &format!("{}x{}", width, height),
            "-framerate",
            &fps.to_string(),
            "-i",
            "-", // Read from stdin
            "-c:v",
            "libx264", // Use H.264 codec
            "-pix_fmt",
            "yuv420p", // Broad compatibility format
            "output.mp4",
        ])
        .stdin(Stdio::piped())
        .spawn()?;

    // 4. Processing Loop
    // We use a block here to ensure stdin is dropped (closed)
    // before we wait, otherwise ffmpeg will wait forever for more data.
    let result = (|| -> Result<(), Box<dyn std::error::Error>> {
        let mut ffmpeg_stdin = child.stdin.take().expect("Failed to open stdin");
        for word in phrase.split_whitespace() {
            // Clean the word (remove surrounding punctuation if desired)
            let clean_word = word.trim_matches(|c: char| !c.is_alphanumeric());
            let word_len = clean_word.len();
            let adjusted_frame_len = match word_len {
                len if len <= 3 => frames_per_word.saturating_sub(2),
                len if len >= 10 => frames_per_word.saturating_sub(3),
                _ => frames_per_word,
            };
            let frame_data = renderer::draw_word_to_frame(clean_word, width, height, 100.0, &font);

            for _ in 0..adjusted_frame_len {
                ffmpeg_stdin.write_all(&frame_data)?;
            }
        }
        Ok(())
    })();

    // Even if 'result' is an Error, we must wait for the child to exit
    let status = child.wait()?;

    if !status.success() {
        eprintln!("FFmpeg exited with an error status: {}", status);
    }

    // Now return the result of the piping operation
    result?;

    println!("Video generated successfully.");
    Ok(())
}
