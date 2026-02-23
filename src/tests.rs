#[cfg(test)]
use crate::{io::load_config, spiral};
#[cfg(test)]
use image::RgbImage;
#[cfg(test)]
use std::error::Error;

#[test]
fn benchmark_spiral() -> Result<(), Box<dyn Error>> {
    let mut img = RgbImage::new(1920, 1280);
    let config = load_config("configuration.toml")?;
    let start = std::time::Instant::now();
    let frames = 100;
    for i in 0..frames {
        spiral::draw_spiral(&mut img, &config.spiral, i, 50.0);
    }
    println!("{} frames took: {:?}", frames, start.elapsed());
    println!("1 frame took: {:?}", start.elapsed() / frames);
    Ok(())
}

#[test]
fn benchmark_spiral_fast() -> Result<(), Box<dyn Error>> {
    let mut img = RgbImage::new(1920, 1280);
    let config = load_config("configuration.toml")?;
    let start = std::time::Instant::now();
    let frames = 100;
    for i in 0..100 {
        spiral::draw_spiral_fast(&mut img, &config.spiral, i, 50.0);
    }
    println!("{} frames took: {:?}", frames, start.elapsed());
    println!("1 frame took: {:?}", start.elapsed() / frames);
    Ok(())
}
