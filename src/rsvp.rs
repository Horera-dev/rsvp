use crate::config::Block;

pub fn determine_orp(word: &[char]) -> usize {
    match word.len() {
        0..=1 => 0,
        2..=5 => 1,
        6..=9 => 2,
        10..13 => 3,
        _ => 4,
    }
}

pub fn determine_frame_duration(word: &str, frames_per_word: u32) -> u32 {
    match word.len() {
        len if len <= 3 => frames_per_word.saturating_sub(2),
        len if len >= 10 => frames_per_word.saturating_sub(3),
        _ => frames_per_word,
    }
}

pub fn get_current_wpm(block: &Block, elapsed_ms: f32) -> f32 {
    let progress = (elapsed_ms / block.duration_ms as f32).min(1.0);
    match block.easing.as_str() {
        "linear" => block.wpm_from + (block.wpm_to - block.wpm_from) * progress,
        _ => block.wpm_from, // Default to starting speed
    }
}
