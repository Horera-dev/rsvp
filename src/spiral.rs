use std::f32::consts::PI;

use image::{Rgb, RgbImage};
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
    frame: u32,
    fps: f32,
    cache: &SpiralCache,
) {
    let rotation_offset = -(frame as f32 / fps) * PI * config.speed;

    img.as_flat_samples_mut()
        .samples
        .par_chunks_exact_mut(3)
        .zip(&cache.distances)
        .zip(&cache.angles)
        .for_each(|((pixel, &r), &theta_base)| {
            let theta = theta_base + rotation_offset;
            let color = get_spiral_color(theta, r, config);
            pixel[0] = color;
            pixel[1] = color;
            pixel[2] = color;
        })
}

pub fn draw_spiral_fast(img: &mut RgbImage, config: &SpiralSettings, frame: u32, fps: f32) {
    let width = img.width();
    let height = img.height();
    let center_x = width as f32 / 2.0;
    let center_y = height as f32 / 2.0;
    let rotation_offset = -(frame as f32 / fps) * PI * config.speed;

    // Access the raw byte buffer of the existing image
    // .par_chunks_exact_mut(3) gives us parallel access to [R, G, B] groups
    img.as_flat_samples_mut()
        .samples
        .par_chunks_exact_mut(3) // Process each row on a different core
        .enumerate()
        .for_each(|(i, pixel)| {
            // Calculate x and y from the flat index
            let dx = (i as u32 % width) as f32 - center_x;
            let dy = (i as u32 / width) as f32 - center_y;
            let r = (dx * dx + dy * dy).sqrt();
            let theta = dy.atan2(dx) + rotation_offset;
            let color = get_spiral_color(theta, r, config);
            pixel[0] = color;
            pixel[1] = color;
            pixel[2] = color;
        });
}

pub fn draw_spiral(img: &mut RgbImage, config: &SpiralSettings, frame: u32, fps: f32) {
    let width = img.width();
    let height = img.height();
    let center_x = width as f32 / 2.0;
    let center_y = height as f32 / 2.0;
    let rotation_offset = -(frame as f32 / fps) * PI * config.speed;

    for y in 0..height {
        for x in 0..width {
            let dx = x as f32 - center_x;
            let dy = y as f32 - center_y;
            let r = (dx * dx + dy * dy).sqrt();
            let theta = dy.atan2(dx) + rotation_offset;
            let color = get_spiral_color(theta, r, config);
            let pixel = Rgb([color, color, color]);
            img.put_pixel(x, y, pixel);
        }
    }
}

fn get_spiral_color(theta: f32, r: f32, config: &SpiralSettings) -> u8 {
    // 1. Calculate 'spiral_value' (Pixel in the spiral)
    let spiral_value = (theta * config.thickness + r * config.curvature).sin();

    // 2. Calculate 't' (Normalization)
    // Added parentheses around (smoothness * 2.0) so you don't divide by smoothness then multiply by 2.
    let t = ((spiral_value + config.smoothness) / (config.smoothness * 2.0)).clamp(0.0, 1.0);

    // 3. Calculate 'smooth_val' (The Curve)
    // This is the standard Smoothstep formula: 3t^2 - 2t^3
    let smooth_val = t * t * (3.0 - 2.0 * t);

    // 4. Calculate 'color' (The Lerp)
    // Formula: start + smooth_val * (end - start)
    (config.lighter_color + smooth_val * (config.darker_color - config.lighter_color)) as u8
}
