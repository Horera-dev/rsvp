use crate::constant;
use crate::rsvp::determine_orp;
use ab_glyph::{Font, FontRef, Glyph, OutlinedGlyph, Point, PxScale, PxScaleFont, ScaleFont};
use image::{Rgb, RgbImage};

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
    render_glyphs_to_image(img, glyphs, orp_index, x_offset, y_offset, font);
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
    for (i, glyph) in glyphs.into_iter().enumerate() {
        if let Some(outlined) = font.outline_glyph(glyph) {
            let is_orp = i == orp_index;
            draw_outlined_glyph(img, outlined, x_offset, y_offset, is_orp);
        }
    }
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
            let background_rgb = img.get_pixel(px, py).0;

            // Highlight the ORP in red, others in white
            let pixel = match is_orp {
                // true => alpha_blend([255.0, 50.0, 50.0], background_rgb, coverage),
                true => alpha_blend(constant::ORP, background_rgb, coverage),
                false => alpha_blend(constant::WHITE, background_rgb, coverage),
            };

            img.put_pixel(px, py, pixel);
        }
    });
}

fn alpha_blend(color: [f32; 3], background: [u8; 3], coverage: f32) -> Rgb<u8> {
    // Alpha Blending Formula: Result = (Front * Alpha) + (Background * (1 - Alpha))
    let r = (color[0] * coverage + background[0] as f32 * (1.0 - coverage)) as u8;
    let g = (color[1] * coverage + background[1] as f32 * (1.0 - coverage)) as u8;
    let b = (color[2] * coverage + background[2] as f32 * (1.0 - coverage)) as u8;
    Rgb([r, g, b])
}
