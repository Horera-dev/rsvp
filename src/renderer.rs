use crate::rsvp::determine_orp;
use ab_glyph::{Font, FontRef, Glyph, OutlinedGlyph, Point, PxScale, PxScaleFont, ScaleFont};
use image::{Rgb, RgbImage};

pub fn draw_word_to_frame(
    word: &str,
    width: u32,
    height: u32,
    scale: f32,
    font: &FontRef,
) -> Vec<u8> {
    // --- Prepare font settings ---
    let mut img = RgbImage::new(width, height);
    let px_scale = PxScale::from(scale);
    let scaled_font = font.as_scaled(px_scale);
    let chars: Vec<char> = word.chars().collect();
    let orp_index = determine_orp(&chars);

    // --- Position glyphs relative to (0,0) ---
    let (glyphs, orp_center_x) = layout_word(word, orp_index, px_scale, &scaled_font);

    // --- Calculate where to place the word on the screen---
    let (x_offset, y_offset) = calculate_alignment(&mut img, orp_center_x, &scaled_font);

    // --- Draw the glyphs onto the image buffer ---
    render_glyphs_to_image(&mut img, glyphs, orp_index, x_offset, y_offset, font);

    img.into_raw()
}

/// Handles horizontal positioning and kerning
fn layout_word(
    word: &str,
    orp_index: usize,
    scale: PxScale,
    scaled_font: &PxScaleFont<&FontRef>,
) -> (Vec<Glyph>, f32) {
    let mut glyphs = Vec::new();
    let mut x_cursor = 0.0;
    let mut orp_center_x = 0.0;
    let mut last_glyph_id = None;

    for (i, c) in word.chars().enumerate() {
        let glyph_id = scaled_font.glyph_id(c);

        if let Some(last_id) = last_glyph_id {
            x_cursor += scaled_font.kern(last_id, glyph_id);
        }

        let glyph_width = scaled_font.h_advance(glyph_id);
        if i == orp_index {
            orp_center_x = x_cursor + (glyph_width / 2.0);
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

    (glyphs, orp_center_x)
}

/// Calculates the shift needed to align the ORP to the center of the screen
fn calculate_alignment(
    img: &mut RgbImage,
    orp_center_x: f32,
    scaled_font: &PxScaleFont<&FontRef>,
) -> (f32, f32) {
    let (width, height) = img.dimensions();
    let x_offset = (width as f32 / 2.0) - orp_center_x;
    let text_height = scaled_font.ascent() - scaled_font.descent();
    let y_offset = (height as f32 - text_height) / 2.0;
    (x_offset, y_offset)
}

/// Final rasterization step
fn render_glyphs_to_image(
    img: &mut RgbImage,
    glyphs: Vec<Glyph>,
    orp_index: usize,
    x_offset: f32,
    y_offset: f32,
    font: &FontRef,
) {
    let (width, height) = img.dimensions();
    for (i, glyph) in glyphs.into_iter().enumerate() {
        if let Some(outlined) = font.outline_glyph(glyph) {
            let is_orp = i == orp_index;
            draw_outlined_glyph(img, outlined, x_offset, y_offset, width, height, is_orp);
        }
    }
}

/// Draws a single glyph with anti-aliasing
fn draw_outlined_glyph(
    img: &mut RgbImage,
    outlined: OutlinedGlyph,
    x_offset: f32,
    y_offset: f32,
    width: u32,
    height: u32,
    is_orp: bool,
) {
    let bounds = outlined.px_bounds();
    outlined.draw(|x, y, c| {
        let px = (x as f32 + bounds.min.x + x_offset) as i32;
        let py = (y as f32 + bounds.min.y + y_offset) as i32;

        if px >= 0 && px < width as i32 && py >= 0 && py < height as i32 {
            let val = (c * 255.0) as u8;
            // Highlight the ORP in red, others in white
            let pixel = if is_orp {
                Rgb([val, (c * 50.0) as u8, (c * 50.0) as u8])
            } else {
                Rgb([val, val, val])
            };
            img.put_pixel(px as u32, py as u32, pixel);
        }
    });
}
