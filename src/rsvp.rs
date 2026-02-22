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
