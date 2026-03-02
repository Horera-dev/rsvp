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
    tint: [u8; 3],
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
            let intensity = get_fading_spiral_color(theta, r, dist_to_edge, config) as f32;
            let rgb = apply_tint_to_color(intensity, tint, config.tint_strength);

            pixel[0] = rgb[0];
            pixel[1] = rgb[1];
            pixel[2] = rgb[2];
        })
}

fn get_fading_spiral_color(theta: f32, r: f32, dist_to_edge: f32, config: &SpiralSettings) -> u8 {
    // 1. The Spiral Math (cos based)
    // Shader uses: cos(0.25 * dist + angle + rotation)
    let spiral_value = (config.curvature * r + theta * config.branches).cos();

    // 2. The Fade Math
    // percentDistToEdge = clamp(dist / distToEdge, 0.0, 1.0)
    let percent_dist_to_edge = (r / dist_to_edge).clamp(0.0, 1.0);

    // col = mix(0.0, col, 1.0 - percentDistToEdge)
    // This multiplier is 1.0 at center, 0.0 at distToEdge
    let fade_factor = 1.0 - percent_dist_to_edge;

    // 3. Apply Smoothstep and Fade to the color range
    let t = ((spiral_value + config.smoothness) / (config.smoothness * 2.0)).clamp(0.0, 1.0);
    let smooth_val = smoothstep(t);

    // Base intensity blended with the fade factor
    let intensity = smooth_val * fade_factor;

    let color = config.lighter_color + intensity * (config.darker_color - config.lighter_color);

    color as u8
}

/// Smoothstep easing — feels much more natural than linear lerp for color blending.
fn smoothstep(t: f32) -> f32 {
    t * t * (3.0 - 2.0 * t)
}

pub fn wpm_to_tint(wpm: f32, config: &SpiralSettings) -> [u8; 3] {
    let t = ((wpm - config.wpm_min) / (config.wpm_max - config.wpm_min)).clamp(0.0, 1.0);
    let t = smoothstep(t); // ease the transition

    let lerp = |a: u8, b: u8| -> u8 { (a as f32 + (b as f32 - a as f32) * t) as u8 };

    [
        lerp(config.color_slow[0], config.color_fast[0]),
        lerp(config.color_slow[1], config.color_fast[1]),
        lerp(config.color_slow[2], config.color_fast[2]),
    ]
}

fn apply_tint_to_color(intensity: f32, tint: [u8; 3], strength: f32) -> [u8; 3] {
    let i = intensity / 255.0;

    // Start from pure greyscale, then blend toward tint
    let grey = intensity;
    let r_out = (grey + (tint[0] as f32 - grey) * strength * i) as u8;
    let g_out = (grey + (tint[1] as f32 - grey) * strength * i) as u8;
    let b_out = (grey + (tint[2] as f32 - grey) * strength * i) as u8;

    ([r_out, g_out, b_out]) as [u8; 3]
}
