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

pub fn compute_progress(words_len: usize, i: usize) -> f32 {
    match words_len {
        0..=1 => 0.0,
        _ => i as f32 / (words_len - 1) as f32,
    }
}

pub fn apply_easing_wpm(block: &Block, progress: f32) -> f32 {
    match block.easing.as_str() {
        "linear" => block.wpm_from + (block.wpm_to - block.wpm_from) * progress,
        _ => block.wpm_from, // Default to starting speed
    }
}

const PUNCTUATION: [char; 5] = ['.', '!', '?', ';', ','];
pub fn apply_punctuation(word: &str) -> f32 {
    match word.ends_with(PUNCTUATION) {
        true => 1.5,
        false => 1.0,
    }
}
pub fn clean_word(word: &str) -> &str {
    word.trim_matches(|c: char| !c.is_alphanumeric())
}
