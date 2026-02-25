use rand::Rng;

use crate::{config::Easing, constant::MASK_CHARSET};

pub fn determine_orp(words_len: usize) -> usize {
    match words_len {
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

pub fn apply_easing(easing: &Easing, from: f32, to: f32, progress: f32) -> f32 {
    match easing {
        Easing::Linear => from + (to - from) * progress,
        _ => to, // Default to starting speed
    }
}

const PUNCTUATION: [char; 5] = ['.', '!', '?', ';', ','];
pub fn apply_punctuation(word: &str) -> f32 {
    match word.ends_with(PUNCTUATION) {
        true => 2.0,
        false => 1.0,
    }
}
pub fn clean_word(word: &str) -> &str {
    word.trim_matches(|c: char| !c.is_alphanumeric())
}

pub fn generate_random_mask(len: usize) -> String {
    let mut rng = rand::thread_rng();

    (0..len)
        .map(|_| MASK_CHARSET[rng.gen_range(0..MASK_CHARSET.len())] as char)
        .collect()
}
