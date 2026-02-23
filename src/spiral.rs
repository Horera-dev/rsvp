use std::f32::consts::PI;

use image::{Rgb, RgbImage};
use rayon::prelude::*;

use crate::config::SpiralSettings;

struct SpiralCache {
    distances: Vec<f32>,
    angles: Vec<f32>,
}

// 1. Run this ONCE at the start of the program
fn create_cache(width: u32, height: u32) -> SpiralCache {
    let mut distances = Vec::with_capacity((width * height) as usize);
    let mut angles = Vec::with_capacity((width * height) as usize);
    let (cx, cy) = (width as f32 / 2.0, height as f32 / 2.0);

    for y in 0..height {
        for x in 0..width {
            let dx = x as f32 - cx;
            let dy = y as f32 - cy;
            distances.push((dx * dx + dy * dy).sqrt());
            angles.push(dy.atan2(dx));
        }
    }
    SpiralCache { distances, angles }
}

pub fn draw_spiral_fast(img: &mut RgbImage, config: &SpiralSettings, frame: u32, fps: f32) {
    let width = img.width();
    let height = img.height();
    let center_x = width as f32 / 2.0;
    let center_y = height as f32 / 2.0;
    let speed = config.speed;
    let rotation_offset = -(frame as f32 / fps) * PI * speed;

    // Access the raw byte buffer of the existing image
    // .par_chunks_exact_mut(3) gives us parallel access to [R, G, B] groups
    img.as_flat_samples_mut()
        .samples
        .par_chunks_exact_mut(3) // Process each row on a different core
        .enumerate()
        .for_each(|(i, pixel)| {
            // Calculate x and y from the flat index
            let x = (i as u32 % width) as f32;
            let y = (i as u32 / width) as f32;

            let dx = x - center_x;
            let dy = y - center_y;

            // Convert Cartesian (x,y) to Polar (r, theta)
            let r = (dx * dx + dy * dy).sqrt();
            let theta = dy.atan2(dx) + rotation_offset;

            // The spiral logic: creates "arms" using a sine wave
            let spiral_value = (theta * config.thickness + r * config.curvature).sin();

            // Smoothstep: Creates a soft transition between -0.1 and 0.1
            // This removes the "staircase" jagged edges.
            let t = ((spiral_value + config.smoothness) / config.smoothness * 2.0).clamp(0.0, 1.0);
            let smooth_val = t * t * (3.0 - t * 2.0);
            let color = (config.lighter_color + smooth_val * config.darker_color) as u8;

            pixel[0] = color;
            pixel[1] = color;
            pixel[2] = color;
        });
}

/**
A spiral is defined by the relationship between the angle (θ) and the radius (r).

For a simple Archimedean spiral:

`r=a+bθ`

To rotate it, we simply add an offset to θ based on the current frame number.
*/
pub fn draw_spiral(img: &mut RgbImage, config: &SpiralSettings, frame: u32, fps: f32) {
    let width = img.width();
    let height = img.height();
    let center_x = width as f32 / 2.0;
    let center_y = height as f32 / 2.0;
    let thickness = config.thickness;
    let curvature = config.curvature;
    let smoothness = config.smoothness;
    let lighter_color = config.lighter_color;
    let darker_color = config.darker_color;
    let speed = config.speed;
    let rotation_offset = -(frame as f32 / fps) * PI * speed;

    for y in 0..height {
        for x in 0..width {
            let dx = x as f32 - center_x;
            let dy = y as f32 - center_y;

            // Convert Cartesian (x,y) to Polar (r, theta)
            let r = (dx * dx + dy * dy).sqrt();
            let theta = dy.atan2(dx) + rotation_offset;

            // The spiral logic: creates "arms" using a sine wave
            let spiral_value = (theta * thickness + r * curvature).sin();

            // Smoothstep: Creates a soft transition between -0.1 and 0.1
            // This removes the "staircase" jagged edges.
            let t = ((spiral_value + smoothness) / smoothness * 2.0).clamp(0.0, 1.0);
            let smooth_val = t * t * (3.0 - t * 2.0);
            let color = (lighter_color + smooth_val * darker_color) as u8;
            let pixel = Rgb([color, color, color]);
            img.put_pixel(x, y, pixel);
        }
    }
}
