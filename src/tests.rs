#[cfg(test)]
use crate::renderer;
#[cfg(test)]
use crate::spiral::create_spiral_cache;
#[cfg(test)]
use crate::{io::load_config, spiral};
#[cfg(test)]
use ab_glyph::FontRef;
#[cfg(test)]
use image::RgbImage;
#[cfg(test)]
use std::error::Error;

#[test]
fn draw_basic() -> Result<(), Box<dyn Error>> {
    // Load Font (Include bytes at compile time for simplicity)
    let font_data = include_bytes!("../assets/Roboto-Black.ttf");
    let font = FontRef::try_from_slice(font_data).expect("Error loading font");
    let config = load_config("configuration.toml")?;

    let mut img = RgbImage::new(1920, 1280);
    renderer::draw_word(&mut img, "Rust", 100.0, &font);
    img.save("test_frame.png").unwrap();

    let mut img = RgbImage::new(1920, 1280);
    spiral::draw_spiral(&mut img, &config.spiral, 533, 1.0);
    img.save("spiral.png").unwrap();

    let mut img = RgbImage::new(1920, 1280);
    spiral::draw_spiral_fast(&mut img, &config.spiral, 533, 1.0);
    img.save("spiral_fast.png").unwrap();

    let mut img = RgbImage::new(1920, 1280);
    let spiral_cache = create_spiral_cache(img.width(), img.height());
    spiral::draw_spiral_fast_with_cache(&mut img, &config.spiral, 533, 1.0, &spiral_cache);
    img.save("spiral_fast_with_cache.png").unwrap();

    Ok(())
}

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
    let frames = 1000;
    for i in 0..frames {
        spiral::draw_spiral_fast(&mut img, &config.spiral, i, 50.0);
    }
    println!("{} frames took: {:?}", frames, start.elapsed());
    println!("1 frame took: {:?}", start.elapsed() / frames);
    Ok(())
}

#[test]
fn benchmark_spiral_fast_with_cache() -> Result<(), Box<dyn Error>> {
    let mut img = RgbImage::new(1920, 1280);
    let config = load_config("configuration.toml")?;
    let start = std::time::Instant::now();
    let spiral_cache = create_spiral_cache(img.width(), img.height());
    let frames = 1000;
    for i in 0..frames {
        spiral::draw_spiral_fast_with_cache(&mut img, &config.spiral, i, 50.0, &spiral_cache);
    }
    println!("{} frames took: {:?}", frames, start.elapsed());
    println!("1 frame took: {:?}", start.elapsed() / frames);
    Ok(())
}
