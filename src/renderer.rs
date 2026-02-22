use ab_glyph::{Font, FontRef, Glyph, GlyphId, Point, PxScale, ScaleFont};
use image::{Rgb, RgbImage};

pub fn determine_orp(word: &[char]) -> usize {
    match word.len() {
        0..=1 => 0,
        2..=5 => 1,
        6..=9 => 2,
        10..13 => 3,
        _ => 4,
    }
}

pub fn draw_word_to_frame(word: &str, width: u32, height: u32, font: &FontRef) -> Vec<u8> {
    // --- Prepare font settings ---
    let mut img = RgbImage::new(width, height);
    let scale = PxScale::from(100.0);
    let scaled_font = font.as_scaled(scale);

    // --- Determine orp ---
    let chars: Vec<char> = word.chars().collect();
    let orp_index = determine_orp(&chars);

    // --- Layout glyphs horizontally ---
    let mut glyphs: Vec<Glyph> = Vec::new();
    let mut last_glyph_id: Option<GlyphId> = None;
    let mut x_cursor = 0.0;
    let mut orp_center_x = 0.0;
    for (i, &c) in chars.iter().enumerate() {
        let glyph_id = font.glyph_id(c);

        // Use scaled_font for kerning and advance
        if let Some(last_id) = last_glyph_id {
            x_cursor += scaled_font.kern(last_id, glyph_id);
        }

        let glyph_width = scaled_font.h_advance(glyph_id);

        if i == orp_index {
            orp_center_x = x_cursor + (glyph_width)
        }

        let glyph = glyph_id.with_scale_and_position(
            scale,
            Point {
                x: x_cursor,
                y: scaled_font.ascent(),
            },
        );
        glyphs.push(glyph);
        x_cursor += glyph_width;
        last_glyph_id = Some(glyph_id);
    }

    // --- Calculate Offsets ---
    // Instead of centering the whole block, we shift the word
    // so that orp_center_x is at width / 2
    let x_offset = (width as f32 / 2.0) - orp_center_x;
    let y_offset = (height as f32 - (scaled_font.ascent() - scaled_font.descent())) / 2.0;

    // --- Rasterize and Draw ---
    for (i, glyph) in glyphs.into_iter().enumerate() {
        if let Some(outlined) = font.outline_glyph(glyph) {
            let bounds = outlined.px_bounds();
            outlined.draw(|x, y, c| {
                // c is the coverage (0.0 to 1.0) for anti-aliasing
                let px = (x as f32 + bounds.min.x + x_offset) as u32;
                let py = (y as f32 + bounds.min.y + y_offset) as u32;

                if px < width && py < height {
                    let val = (c * 255.0) as u8;

                    // Highlight the ORP in red, others in white
                    let color = if i == orp_index {
                        Rgb([val, (c * 50.0) as u8, (c * 50.0) as u8])
                    } else {
                        Rgb([val, val, val])
                    };

                    img.put_pixel(px, py, color);
                }
            });
        }
    }

    img.into_raw()
}
