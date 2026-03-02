
fn draw_spiral_fast(img: &mut RgbImage, config: &SpiralSettings, time_secs: f32) {
    let clockwise_value = if config.clockwise { -1.0 } else { 1.0 };
    let width = img.width();
    let height = img.height();
    let center_x = width as f32 / 2.0;
    let center_y = height as f32 / 2.0;
    //`TAU = 2π` is literally "one full turn", so now:
    // - `speed = 1.0` → one full rotation per second per branch
    // - `speed = 0.5` → half a rotation per second
    // - the unit of `speed` is now **rotations/second**, which is legible
    let rotation_offset = (clockwise_value * time_secs * TAU * config.speed) / config.branches;

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

fn draw_spiral(img: &mut RgbImage, config: &SpiralSettings, time_secs: f32) {
    let clockwise_value = if config.clockwise { -1.0 } else { 1.0 };
    let width = img.width();
    let height = img.height();
    let center_x = width as f32 / 2.0;
    let center_y = height as f32 / 2.0;
    //`TAU = 2π` is literally "one full turn", so now:
    // - `speed = 1.0` → one full rotation per second per branch
    // - `speed = 0.5` → half a rotation per second
    // - the unit of `speed` is now **rotations/second**, which is legible
    let rotation_offset = (clockwise_value * time_secs * TAU * config.speed) / config.branches;

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
    let spiral_value = (theta * config.branches + r * config.curvature).sin();

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
