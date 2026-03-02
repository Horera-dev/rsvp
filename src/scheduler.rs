use std::{error::Error, io::Write};

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
        wpm: f32,
    },
    /// A masking frame: draw the spiral + a random mask of this character length.
    Mask {
        time_secs: f32,
        word_len: usize, // we only need the length to generate the mask
        scale: f32,
        wpm: f32,
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
                    wpm,
                });
                frame_count += 1;
            }

            for _ in 0..config.settings.masking_frames {
                instructions.push(FrameInstruction::Mask {
                    time_secs: frame_count as f32 / fps,
                    word_len: word.len(),
                    scale,
                    wpm,
                });
                frame_count += 1;
            }
        }
    }

    instructions
}

pub struct PaddingInfo {
    pub instructions: Vec<FrameInstruction>,
    pub period: u32,
    pub remainder: u32,
}

pub fn compute_padding(config: &Config, frame_count: u32) -> PaddingInfo {
    // (TAU * speed * T) / branches = TAU / branches
    // TAU * speed * T = TAU
    // T = 1 / speed     seconds
    let fps = config.settings.active_format().fps;
    let period = (fps / config.spiral.speed).round() as u32;
    let remainder = period - (frame_count % period);
    let mut instructions = Vec::new();

    for i in 0..remainder {
        instructions.push(FrameInstruction::Padding {
            time_secs: (frame_count + i) as f32 / fps,
        });
    }

    PaddingInfo {
        instructions,
        period,
        remainder,
    }
}

pub fn dump_schedule(
    instructions: &[FrameInstruction],
    path: &str,
    period: u32,
    remainder: u32,
    elapsed: std::time::Duration,
) -> Result<(), Box<dyn Error>> {
    let mut file = std::fs::File::create(path)?;

    for (i, instruction) in instructions.iter().enumerate() {
        let line = match instruction {
            FrameInstruction::Word {
                time_secs,
                word,
                scale,
                wpm,
            } => format!(
                "{:>6} | {:.4}s | WORD  | {:.2}x | {:.1}wpm | {:?} \n",
                i, time_secs, scale, wpm, word
            ),
            FrameInstruction::Mask {
                time_secs,
                word_len,
                scale,
                wpm,
            } => format!(
                "{:>6} | {:.4}s | MASK  | {:.2}x | {:.1}wpm | len={} \n",
                i, time_secs, scale, wpm, word_len
            ),
            FrameInstruction::Padding { time_secs } => {
                format!("{:>6} | {:.4}s | PAD   |\n", i, time_secs)
            }
        };
        file.write_all(line.as_bytes())?;
    }

    // Summary at the end
    let words = instructions
        .iter()
        .filter(|i| matches!(i, FrameInstruction::Word { .. }))
        .count();
    let masks = instructions
        .iter()
        .filter(|i| matches!(i, FrameInstruction::Mask { .. }))
        .count();
    let pads = instructions
        .iter()
        .filter(|i| matches!(i, FrameInstruction::Padding { .. }))
        .count();

    let n = instructions.len() as u32;
    let avg = elapsed / n;

    write!(
        file,
        "
--- Schedule ---
Total frames : {}  |  {} word  |  {} mask  |  {} padding
Period       : {} frames
Remainder    : {} frames
Last time_secs should be a multiple of period: {}

--- Performance ---
Total   : {:?}
Avg/frame: {:?}
Eff. FPS : {:.2}
",
        n,
        words,
        masks,
        pads,
        period,
        remainder,
        if n.is_multiple_of(period) {
            "✓ OK"
        } else {
            "✗ NOT a multiple — loop will jump!"
        },
        elapsed,
        avg,
        1.0 / avg.as_secs_f32(),
    )?;

    Ok(())
}
