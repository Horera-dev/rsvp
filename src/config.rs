use serde::Deserialize;

use crate::color::Color;

#[derive(Deserialize)]
pub struct Config {
    pub settings: GlobalSettings,
    #[serde(default)] // ← treats missing field as Vec::new()
    pub blocks: Vec<Block>,
    pub spiral: SpiralSettings,
}

#[derive(Deserialize, PartialEq, Debug)]
#[serde(rename_all = "lowercase")]
pub enum RenderMode {
    Gif,
    Video,
}

#[derive(Deserialize, PartialEq, Debug, Clone)]
#[serde(rename_all = "lowercase")]
pub enum Easing {
    Linear,
    Instant,
}

#[derive(Deserialize)]
pub struct GlobalSettings {
    pub renderer: RenderMode,
    pub font_path: String,
    pub masking_frames: u32,
    pub content_path: Option<String>, // path to a .rsvp script file
    pub video: FormatSettings,        // [settings.video]
    pub gif: FormatSettings,          // [settings.gif]
}

#[derive(Deserialize)]
pub struct FormatSettings {
    pub width: u32,
    pub height: u32,
    pub fps: f32,
    pub scale: f32,
    pub easing: Easing,
}

#[derive(Deserialize)]
pub struct SpiralSettings {
    pub branches: f32,
    pub curvature: f32,
    pub smoothness: f32,
    pub lighter_color: Color,
    pub darker_color: Color,
    pub speed: f32,
    pub shrink_height: f32,
    pub clockwise: bool,
    pub color_slow: Color,
    pub color_fast: Color,
    pub wpm_min: f32,
    pub wpm_max: f32,
    pub tint_strength: f32,
}

#[derive(Copy, Clone, Deserialize)]
pub struct FlashSettings {
    pub accent_color: Color,
    pub bg_color: Color,
}

#[derive(Deserialize)]
pub struct Block {
    pub text: String,

    #[serde(default = "default_float")]
    pub wpm_from: f32,

    #[serde(default = "default_float")]
    pub wpm_to: f32,
    pub easing: Option<Easing>,
    pub scale: Option<f32>,
    pub flash: Option<FlashSettings>,
}

fn default_float() -> f32 {
    0.0
}

impl Block {
    /// Returns the block's scale if defined, otherwise returns the fallback.
    pub fn get_scale(&self, fallback: f32) -> f32 {
        self.scale.unwrap_or(fallback)
    }
    /// Returns the block's scale if defined, otherwise returns the fallback.
    pub fn get_easing(&self, fallback: Easing) -> Easing {
        self.easing.clone().unwrap_or(fallback)
    }
}

impl GlobalSettings {
    pub fn active_format(&self) -> &FormatSettings {
        match self.renderer {
            RenderMode::Gif => &self.gif,
            RenderMode::Video => &self.video,
        }
    }
}
