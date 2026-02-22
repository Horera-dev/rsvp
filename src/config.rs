use serde::Deserialize;

#[derive(Deserialize)]
pub struct Config {
    pub settings: GlobalSettings,
    pub blocks: Vec<Block>,
}

#[derive(Deserialize)]
pub struct GlobalSettings {
    pub width: u32,
    pub height: u32,
    pub fps: f32,
    pub font_path: String,
}

#[derive(Deserialize)]
pub struct Block {
    pub text: String,
    pub wpm_from: f32,
    pub wpm_to: f32,
    pub duration_ms: u32, // Duration of the whole block in milliseconds
    pub easing: String,   // "linear" or "instant"
}
