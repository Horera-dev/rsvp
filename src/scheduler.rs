use crate::{config::Config, rsvp};

/// A FrameInstruction describes what a single frame should contain,
/// without caring at all about how pixels are produced.
/// This is pure data — no I/O, no image crate types.
pub enum FrameInstruction {
    /// A visible word frame: draw the spiral + this word on top.
    Word {
        time_secs: f32,
        word: String, // owned, because the schedule outlives the block loop
        scale: f32,
    },
    /// A masking frame: draw the spiral + a random mask of this character length.
    Mask {
        time_secs: f32,
        word_len: usize, // we only need the length to generate the mask
        scale: f32,
    },
    /// A padding frame: draw the spiral only, no text.
    Padding { time_secs: f32 },
}

pub fn compute_schedule(config: &Config) -> Vec<FrameInstruction> {
    let active = config.settings.active_format();
    let fps = active.fps;

    let mut instructions = Vec::new();
    let mut frame_count: u32 = 0;

    for block in &config.blocks {
        let words: Vec<&str> = block.text.split_whitespace().collect();
        let scale = block.get_scale(active.scale);
        let easing = block.get_easing(active.easing.clone());
        let mut frac_buffer: f32 = 0.0;

        for (i, word) in words.iter().enumerate() {
            let progress = rsvp::compute_progress(words.len(), i);
            let wpm = rsvp::apply_easing(&easing, block.wpm_from, block.wpm_to, progress);
            let base = (60.0 / wpm) * fps;
            let weighted = base * rsvp::apply_punctuation(word);
            let total_raw = weighted + frac_buffer;
            let frames = total_raw.round();
            frac_buffer = total_raw - frames;
            if frames < 1.0 {
                continue;
            }

            let word_frames = frames as u32;
            let cleaned = rsvp::clean_word(word).to_string();

            // Push one instruction per frame — no rendering here, just description
            for _ in 0..word_frames {
                instructions.push(FrameInstruction::Word {
                    time_secs: frame_count as f32 / fps,
                    word: cleaned.clone(),
                    scale,
                });
                frame_count += 1;
            }

            for _ in 0..config.settings.masking_frames {
                instructions.push(FrameInstruction::Mask {
                    time_secs: frame_count as f32 / fps,
                    word_len: word.len(),
                    scale,
                });
                frame_count += 1;
            }
        }
    }

    let period = ((2.0 * fps) / (config.spiral.speed)).round() as u32;
    let remainder = period - (frame_count % period);

    println!(
        "Frame: {}; Period: {}; Remainder: {}",
        frame_count, period, remainder
    );

    for _ in 0..remainder {
        instructions.push(FrameInstruction::Padding {
            time_secs: frame_count as f32 / fps,
        });
        frame_count += 1;
    }

    instructions
}
