use crate::config::Config;
use anyhow::{Context, Result};
use std::fs;
use std::path::Path;

pub fn load_config<P: AsRef<Path>>(path: P) -> Result<Config> {
    let absolute_path = std::env::current_dir()?.join(path.as_ref());

    let content = fs::read_to_string(&absolute_path)
        .with_context(|| format!("Failed to read config file at: {:?}", absolute_path))?;

    let config = toml::from_str(&content)
        .with_context(|| "Failed to parse TOML configuration. Check your syntax.")?;

    Ok(config)
}

/// Reads the font file into an owned byte vector.
pub fn load_font_data(config: &Config) -> Result<Vec<u8>> {
    let font_path = &config.settings.font_path;

    let font_data = fs::read(font_path)
        .with_context(|| format!("Font file missing or inaccessible: {}", font_path))?;

    Ok(font_data)
}
