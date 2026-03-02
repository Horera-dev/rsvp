// theta, r  →  [spiral math]  →  intensity: f32 (0.0–1.0)
//                                      ↓
// wpm       →  [wpm_to_tint]  →  tint: [f32; 3] (0.0–1.0 per channel)
//                                      ↓
//                              [blend_color]  →  [f32; 3]
//                                      ↓
//                              [to_pixel]     →  [u8; 3]

use std::f32::consts::TAU;

use image::RgbImage;
use rayon::prelude::*;

use crate::{color::Color, config::SpiralSettings, renderer};

pub struct SpiralCache {
    pub distances: Vec<f32>,
    pub angles: Vec<f32>,
}

pub fn create_spiral_cache(width: u32, height: u32) -> SpiralCache {
    let mut distances = Vec::with_capacity((width * height) as usize);
    let mut angles = Vec::with_capacity((width * height) as usize);
    let center_x = width as f32 / 2.0;
    let center_y = height as f32 / 2.0;

    for y in 0..height {
        for x in 0..width {
            let dx = x as f32 - center_x;
            let dy = y as f32 - center_y;
            distances.push((dx * dx + dy * dy).sqrt());
            angles.push(dy.atan2(dx));
        }
    }
    SpiralCache { distances, angles }
}

pub fn draw_spiral_fast_with_cache(
    img: &mut RgbImage,
    config: &SpiralSettings,
    time_secs: f32,
    cache: &SpiralCache,
    tint: Color,
) {
    let clockwise_value = if config.clockwise { -1.0 } else { 1.0 };
    //`TAU = 2π` is literally "one full turn", so now:
    // - `speed = 1.0` → one full rotation per second per branch
    // - `speed = 0.5` → half a rotation per second
    // - the unit of `speed` is now **rotations/second**, which is legible
    let rotation_offset = (clockwise_value * time_secs * TAU * config.speed) / config.branches;
    let dist_to_edge = img.height().min(img.width()) as f32 / config.shrink_height;

    img.as_flat_samples_mut()
        .samples
        .par_chunks_exact_mut(3)
        .zip(&cache.distances)
        .zip(&cache.angles)
        .for_each(|((pixel, &r), &theta_base)| {
            let theta = theta_base + rotation_offset;
            let intensity = spiral_intensity(theta, r, dist_to_edge, config);
            let base = spiral_base_color(intensity, config);
            let color = blend_tint(base, tint, config.tint_strength);
            pixel[0] = color.r as u8;
            pixel[1] = color.g as u8;
            pixel[2] = color.b as u8;
        })
}

// Everything internal works in 0.0–1.0 normalized floats.
// Only `to_pixel` touches u8.
fn spiral_intensity(theta: f32, r: f32, dist_to_edge: f32, config: &SpiralSettings) -> f32 {
    let spiral_value = (config.curvature * r + theta * config.branches).cos();
    let fade = 1.0 - (r / dist_to_edge).clamp(0.0, 1.0);
    let t = ((spiral_value + config.smoothness) / (config.smoothness * 2.0)).clamp(0.0, 1.0);
    renderer::smoothstep(t) * fade // 0.0–1.0
}

fn spiral_base_color(intensity: f32, config: &SpiralSettings) -> Color {
    config.darker_color.lerp(config.lighter_color, intensity)
}

pub fn wpm_to_tint(wpm: f32, config: &SpiralSettings) -> Color {
    let t = ((wpm - config.wpm_min) / (config.wpm_max - config.wpm_min)).clamp(0.0, 1.0);
    let smooth_t = renderer::smoothstep(t);
    config.color_fast.lerp(config.color_slow, smooth_t)
}

fn blend_tint(base: Color, tint: Color, strength: f32) -> Color {
    // Determine how "bright" the base pixel is to weight the tint
    let luma = (base.r + base.g + base.b) / (3.0 * 255.0);
    let effective_strength = strength * luma;
    base.lerp(tint, effective_strength)
}
