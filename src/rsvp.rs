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

pub fn get_current_wpm(block: &Block, progress: f32) -> f32 {
    match block.easing.as_str() {
        "linear" => block.wpm_from + (block.wpm_to - block.wpm_from) * progress,
        _ => block.wpm_from, // Default to starting speed
    }
}
