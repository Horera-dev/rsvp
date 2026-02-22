mod renderer;

use std::io::{self, Write};

fn draw_basic() {
    let frame_bytes = renderer::draw_word_to_frame("Rust", 1280, 720);
    let img = image::RgbImage::from_raw(1280, 720, frame_bytes).unwrap();
    img.save("test_frame.png").unwrap();
}
fn main() -> io::Result<()> {
    draw_basic();
    return Ok(());

    let words = "Rust is incredibly fast for video processing tasks";
    let width = 1920;
    let height = 1280;

    let mut stdout = io::stdout().lock();
    for word in words.split_whitespace() {
        // Create a frame for each word
        let frame_data = renderer::draw_word_to_frame(word, width, height);
        for _ in 0..15 {
            let _ = stdout.write_all(&frame_data);
        }
    }

    Ok(())
}
