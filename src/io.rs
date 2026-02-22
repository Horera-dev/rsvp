use std::fs;

use crate::config::Config;

pub fn load_config(path: &str) -> Config {
    let content = fs::read_to_string(path).expect(
        "Failed to load configuration.toml. Please ensure the file exists in the parent directory.",
    );
    toml::from_str(&content).expect(
        "Failed to load configuration.toml. Please ensure the file exists in the parent directory.",
    )
}

pub fn load_font_data(config: &Config) -> Vec<u8> {
    fs::read(config.settings.font_path.clone()).unwrap_or_else(|_| {
        panic!(
            "Could not find the font file at the specified path: {}",
            config.settings.font_path
        )
    })
}
