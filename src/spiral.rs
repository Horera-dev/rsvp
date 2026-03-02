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

use crate::config::SpiralSettings;

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
    tint: [f32; 3],
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
            let rgb = to_pixel(color);

            pixel[0] = rgb[0];
            pixel[1] = rgb[1];
            pixel[2] = rgb[2];
        })
}

/// Smoothstep easing — feels much more natural than linear lerp for color blending.
fn smoothstep(t: f32) -> f32 {
    t * t * (3.0 - 2.0 * t)
}

fn lerp(a: f32, b: f32, lerp_max: f32) -> f32 {
    a / lerp_max + (b / lerp_max - a / lerp_max)
}

// Everything internal works in 0.0–1.0 normalized floats.
// Only `to_pixel` touches u8.
fn spiral_intensity(theta: f32, r: f32, dist_to_edge: f32, config: &SpiralSettings) -> f32 {
    let spiral_value = (config.curvature * r + theta * config.branches).cos();
    let fade = 1.0 - (r / dist_to_edge).clamp(0.0, 1.0);
    let t = ((spiral_value + config.smoothness) / (config.smoothness * 2.0)).clamp(0.0, 1.0);
    smoothstep(t) * fade // 0.0–1.0
}

fn spiral_base_color(intensity: f32, config: &SpiralSettings) -> f32 {
    // Map intensity into the configured light/dark range, normalized
    let lighter = config.lighter_color / 255.0;
    let darker = config.darker_color / 255.0;
    lighter + intensity * (darker - lighter) // 0.0–1.0
}

pub fn wpm_to_tint(wpm: f32, config: &SpiralSettings) -> [f32; 3] {
    let t =
        smoothstep(((wpm - config.wpm_min) / (config.wpm_max - config.wpm_min)).clamp(0.0, 1.0));
    [
        lerp(config.color_slow[0], config.color_fast[0], 255.0) * t,
        lerp(config.color_slow[1], config.color_fast[1], 255.0) * t,
        lerp(config.color_slow[2], config.color_fast[2], 255.0) * t,
    ]
}

fn blend_tint(base: f32, tint: [f32; 3], strength: f32) -> [f32; 3] {
    // intensity-weighted strength: no tint where spiral is dark
    let effective_strength = strength * base;
    // Blend from greyscale toward tint color, scaled by strength
    [
        base + (tint[0] - base) * effective_strength,
        base + (tint[1] - base) * effective_strength,
        base + (tint[2] - base) * effective_strength,
    ]
}

fn to_pixel(color: [f32; 3]) -> [u8; 3] {
    color.map(|c| (c.clamp(0.0, 1.0) * 255.0) as u8)
}
