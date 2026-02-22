mod renderer;

use std::io::{self, Write};

use ab_glyph::FontRef;

fn draw_basic() {
    // Load Font (Include bytes at compile time for simplicity)
    let font_data = include_bytes!("../assets/Roboto-Black.ttf");
    let font = FontRef::try_from_slice(font_data).expect("Error loading font");
    let frame_bytes = renderer::draw_word_to_frame("Rust", 1920, 1280, &font);
    let img = image::RgbImage::from_raw(1920, 1280, frame_bytes).unwrap();
    img.save("test_frame.png").unwrap();
}
fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Test
    draw_basic();

    // 1. Initialize Video-RS
    video_rs::init()?;

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

    // 3. Setup Encoder
    let settings = EncoderSettings::for_h264_yuv420p(width as usize, height as usize, fps as usize);
    let mut encoder = Encoder::new(&PathBuf::from("output.mp4").into(), settings)?;

    let phrase = "Rust makes video encoding surprisingly approachable.";
    let mut current_time = Time::zero();

    // 4. Processing Loop
    for word in phrase.split_whitespace() {
        // Clean the word (remove surrounding punctuation if desired)
        let clean_word = word.trim_matches(|c: char| !c.is_alphanumeric());
        let word_len = clean_word.len();
        let adjusted_frame_len = match word_len {
            len if len <= 3 => frames_per_word.saturating_sub(2),
            len if len >= 10 => frames_per_word.saturating_sub(3),
            _ => frames_per_word,
        };
        let frame_bytes = renderer::draw_word_to_frame(clean_word, width, height, &font);

        // Convert raw Vec<u8> to ndarray (H, W, C)
        let frame_array = Array3::from_shape_vec((height, width, 3), frame_bytes)?;

        for _ in 0..adjusted_frame_len {
            encoder.encode(&frame_array, &current_time)?;
            current_time = current_time
                .aligned_with(&duration_per_word)
                .add(&duration_per_word);
        }
    }

    // 5. Finalize (Flushes the encoder)
    encoder.finish()?;
    println!("Video saved to output.mp4");

    return Ok(());

    let mut stdout = io::stdout().lock();

    for word in phrase.split_whitespace() {
        // Create a frame for each word
        let frame_data = renderer::draw_word_to_frame(clean_word, width, height, &font);

        // Write the same frame multiple times to create "duration"
        for _ in 0..adjusted_frames {
            stdout.write_all(&frame_data)?;
        }
    }

    Ok(())
}
