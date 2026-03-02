use crate::rsvp::determine_orp;
use crate::{color::Color, constant};
use ab_glyph::{Font, FontRef, Glyph, OutlinedGlyph, Point, PxScale, PxScaleFont, ScaleFont};
use image::RgbImage;

pub fn draw_word(img: &mut RgbImage, word: &str, scale: f32, font: &FontRef) {
    // --- Prepare font settings ---
    let px_scale = PxScale::from(scale);
    let scaled_font = font.as_scaled(px_scale);
    let chars: Vec<char> = word.chars().collect();
    let orp_index = determine_orp(chars.len());

    // --- Position glyphs relative to (0,0) ---
    let (glyphs, orp_center_x) = layout_word(word, orp_index, px_scale, &scaled_font);

    // --- Calculate where to place the word on the screen---
    let (x_offset, y_offset) = calculate_alignment(img, orp_center_x, &scaled_font);

    // --- Draw the glyphs onto the image buffer ---
    for (i, glyph) in glyphs.into_iter().enumerate() {
        if let Some(outlined) = font.outline_glyph(glyph) {
            let is_orp = i == orp_index;
            draw_outlined_glyph(img, outlined, x_offset, y_offset, is_orp);
        }
    }
}

pub fn draw_word_colored(img: &mut RgbImage, word: &str, scale: f32, font: &FontRef, color: Color) {
    // --- Prepare font settings ---
    let px_scale = PxScale::from(scale);
    let scaled_font = font.as_scaled(px_scale);
    let chars: Vec<char> = word.chars().collect();
    let orp_index = determine_orp(chars.len());

    // --- Position glyphs relative to (0,0) ---
    let (glyphs, orp_center_x) = layout_word(word, orp_index, px_scale, &scaled_font);

    // --- Calculate where to place the word on the screen---
    let (x_offset, y_offset) = calculate_alignment(img, orp_center_x, &scaled_font);

    // --- Draw the glyphs onto the image buffer ---

    for glyph in glyphs.into_iter() {
        if let Some(outlined) = font.outline_glyph(glyph) {
            draw_outlined_glyph_colored(img, outlined, x_offset, y_offset, color);
        }
    }
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

/// Draws a single glyph with anti-aliasing
fn draw_outlined_glyph(
    img: &mut RgbImage,
    outlined: OutlinedGlyph,
    x_offset: f32,
    y_offset: f32,
    is_orp: bool,
) {
    let width = img.width();
    let height = img.height();
    let bounds = outlined.px_bounds();
    outlined.draw(|x, y, coverage| {
        let px = (x as f32 + bounds.min.x + x_offset) as u32;
        let py = (y as f32 + bounds.min.y + y_offset) as u32;

        if px < width && py < height {
            let bg = Color::pixel(img.get_pixel(px, py).0);

            // Highlight the ORP in red, others in white
            let color = match is_orp {
                true => constant::ORP.lerp(bg, coverage),
                false => constant::WHITE.lerp(bg, coverage),
            };

            img.put_pixel(px, py, color.to_rgb());
        }
    });
}

/// Draws a single glyph with anti-aliasing
fn draw_outlined_glyph_colored(
    img: &mut RgbImage,
    outlined: OutlinedGlyph,
    x_offset: f32,
    y_offset: f32,
    color: Color,
) {
    let width = img.width();
    let height = img.height();
    let bounds = outlined.px_bounds();
    outlined.draw(|x, y, coverage| {
        let px = (x as f32 + bounds.min.x + x_offset) as u32;
        let py = (y as f32 + bounds.min.y + y_offset) as u32;

        if px < width && py < height {
            let bg = Color::pixel(img.get_pixel(px, py).0);
            let pixel = color.lerp(bg, coverage);
            img.put_pixel(px, py, pixel.to_rgb());
        }
    });
}

/// Smoothstep easing — feels much more natural than linear lerp for color blending.
pub fn smoothstep(t: f32) -> f32 {
    t * t * (3.0 - 2.0 * t)
}

pub fn wash_to_background(img: &mut RgbImage, bg: Color, amount: f32) {
    if amount <= 0.0 {
        return;
    } // skip if no wash needed
    img.pixels_mut().for_each(|p| {
        p[0] = (p[0] as f32 + (bg.r - p[0] as f32) * amount) as u8;
        p[1] = (p[1] as f32 + (bg.g - p[1] as f32) * amount) as u8;
        p[2] = (p[2] as f32 + (bg.b - p[2] as f32) * amount) as u8;
    });
}
