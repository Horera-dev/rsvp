use std::f32::consts::TAU;

use crate::scheduler::AudioInstruction;
use crate::utils;
use std::error::Error;
use std::io::Write;
use std::path::Path;

#[derive(Debug, Clone, serde::Deserialize)]
pub struct BinauralSettings {
    pub carrier_hz: f32, // base frequency, e.g. 200.0
    pub beat_hz: f32,    // binaural beat, e.g. 7.0 (theta)
    pub drone_hz: f32,   // ambient drone, e.g. 180.0
    pub volume: f32,     // 0.0–1.0
}

impl BinauralSettings {
    /// Relaxed focus preset — theta wave, good for reading
    pub fn theta() -> Self {
        Self {
            carrier_hz: 200.0,
            beat_hz: 7.0,
            drone_hz: 180.0,
            volume: 0.4,
        }
    }

    /// Calm alertness preset — alpha wave
    #[allow(dead_code)]
    pub fn alpha() -> Self {
        Self {
            carrier_hz: 220.0,
            beat_hz: 10.0,
            drone_hz: 196.0,
            volume: 0.4,
        }
    }
}

#[derive(Clone)]
pub struct BinauralGen {
    pub sample_rate: u32,
    phase_left: f32, // continuous phase to avoid clicks between frames
    phase_right: f32,
    phase_drone: f32,
}

impl BinauralGen {
    pub fn new(sample_rate: u32) -> Self {
        Self {
            sample_rate,
            phase_left: 0.0,
            phase_right: 0.0,
            phase_drone: 0.0,
        }
    }

    /// Generate one frame of stereo PCM samples.
    /// Returns interleaved [L, R, L, R, ...] f32 in -1.0..1.0
    pub fn generate_frame(&mut self, settings: &BinauralSettings, fps: f32) -> Vec<f32> {
        let samples_per_frame = (self.sample_rate as f32 / fps).round() as usize;
        let mut output = Vec::with_capacity(samples_per_frame * 2);
        let dt = 1.0 / self.sample_rate as f32;

        for _ in 0..samples_per_frame {
            let binaural_l = self.phase_left.sin();
            let binaural_r = self.phase_right.sin();
            let drone = self.phase_drone.sin() * 0.3; // drone quieter than tones

            // Mix binaural tone + drone, then scale by volume
            // * 0.5 keeps headroom when summing two signals
            output.push((binaural_l + drone) * settings.volume * 0.5);
            output.push((binaural_r + drone) * settings.volume * 0.5);

            // Advance phases — modulo TAU keeps values small and avoids float drift
            self.phase_left = (self.phase_left + TAU * settings.carrier_hz * dt) % TAU;
            self.phase_right =
                (self.phase_right + TAU * (settings.carrier_hz + settings.beat_hz) * dt) % TAU;
            self.phase_drone = (self.phase_drone + TAU * settings.drone_hz * dt) % TAU;
        }

        output
    }
}

/// Write interleaved f32 stereo samples as a valid WAV file.
pub fn write_wav(path: &Path, samples: &[f32], sample_rate: u32) -> Result<(), Box<dyn Error>> {
    let mut file = std::fs::File::create(path)?;

    let num_channels = 2u16;
    let bits_per_sample = 32u16;
    let byte_rate = sample_rate * num_channels as u32 * (bits_per_sample as u32 / 8);
    let block_align = num_channels * (bits_per_sample / 8);
    let data_size = (samples.len() * 4) as u32; // 4 bytes per f32
    let chunk_size = 36 + data_size;

    // RIFF header
    file.write_all(b"RIFF")?;
    file.write_all(&chunk_size.to_le_bytes())?;
    file.write_all(b"WAVE")?;

    // fmt chunk — IEEE float PCM (format 3)
    file.write_all(b"fmt ")?;
    file.write_all(&16u32.to_le_bytes())?; // chunk size
    file.write_all(&3u16.to_le_bytes())?; // format: IEEE float
    file.write_all(&num_channels.to_le_bytes())?;
    file.write_all(&sample_rate.to_le_bytes())?;
    file.write_all(&byte_rate.to_le_bytes())?;
    file.write_all(&block_align.to_le_bytes())?;
    file.write_all(&bits_per_sample.to_le_bytes())?;

    // data chunk
    file.write_all(b"data")?;
    file.write_all(&data_size.to_le_bytes())?;
    for sample in samples {
        file.write_all(&sample.to_le_bytes())?;
    }

    Ok(())
}

pub fn generate_audio(instructions: &[AudioInstruction], fps: f32, sample_rate: u32) -> Vec<f32> {
    let mut generator = BinauralGen::new(sample_rate);
    let mut samples = Vec::new();

    for instruction in instructions {
        let frame_samples = match instruction {
            AudioInstruction::Silence => {
                let n = (sample_rate as f32 / fps).round() as usize;
                vec![0.0f32; n * 2]
            }
            AudioInstruction::Binaural(settings) => generator.generate_frame(settings, fps),
            AudioInstruction::CrossFade { from, to, t } => {
                let blended = BinauralSettings {
                    carrier_hz: utils::lerp(from.carrier_hz, to.carrier_hz, *t),
                    beat_hz: utils::lerp(from.beat_hz, to.beat_hz, *t),
                    drone_hz: utils::lerp(from.drone_hz, to.drone_hz, *t),
                    volume: utils::lerp(from.volume, to.volume, *t),
                };
                generator.generate_frame(&blended, fps)
            }
        };

        samples.extend(frame_samples);
    }

    samples
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn frame_has_correct_sample_count() {
        let mut generator = BinauralGen::new(44100);
        let settings = BinauralSettings::theta();
        let frame = generator.generate_frame(&settings, 50.0);

        // 44100 / 50fps = 882 samples per channel, * 2 for stereo
        assert_eq!(frame.len(), 882 * 2);
    }

    #[test]
    fn samples_are_in_valid_range() {
        let mut generator = BinauralGen::new(44100);
        let settings = BinauralSettings::theta();
        let frame = generator.generate_frame(&settings, 50.0);

        for s in &frame {
            assert!(*s >= -1.0 && *s <= 1.0, "Sample out of range: {}", s);
        }
    }

    #[test]
    fn phase_is_continuous_across_frames() {
        let mut generator = BinauralGen::new(44100);
        let settings = BinauralSettings::theta();

        let frame1 = generator.generate_frame(&settings, 50.0);
        let frame2 = generator.generate_frame(&settings, 50.0);

        // Last sample of frame1 and first sample of frame2 should be close
        // (no discontinuity = no click)
        let last = frame1[frame1.len() - 2]; // last L sample
        let first = frame2[0]; // first L sample
        let delta = (last - first).abs();
        assert!(
            delta < 0.1,
            "Phase discontinuity detected: delta = {}",
            delta
        );
    }
}
