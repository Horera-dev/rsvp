pub struct Config {
    pub width: u32,
    pub height: u32,
    pub fps: f32,
    pub wpm: f32,
    pub phrase: String,
}

impl Config {
    pub fn frames_per_word(&self) -> u32 {
        (60.0 / self.wpm * self.fps) as u32
    }
}
