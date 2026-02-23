use serde::Deserialize;

#[derive(Deserialize)]
pub struct Config {
    pub settings: GlobalSettings,
    pub blocks: Vec<Block>,
}

#[derive(Deserialize, PartialEq, Debug)]
#[serde(rename_all = "lowercase")]
pub enum RenderMode {
    Gif,
    Video,
}

#[derive(Deserialize)]
pub struct GlobalSettings {
    pub renderer: RenderMode,
    pub font_path: String,
    pub video: FormatSettings, // [settings.video]
    pub gif: FormatSettings,   // [settings.gif]
}

#[derive(Deserialize)]
pub struct FormatSettings {
    pub width: u32,
    pub height: u32,
    pub fps: f32,
    pub scale: f32,
}

#[derive(Deserialize)]
pub struct Block {
    pub text: String,
    pub wpm_from: f32,
    pub wpm_to: f32,
    pub easing: String, // "linear" or "instant"
    pub scale: Option<f32>,
}

impl Block {
    /// Returns the block's scale if defined, otherwise returns the fallback.
    pub fn get_scale(&self, fallback: f32) -> f32 {
        self.scale.unwrap_or(fallback)
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
