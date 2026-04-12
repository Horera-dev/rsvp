// script.rs

use crate::{
    audio::BinauralSettings,
    color::Color,
    config::{Block, Easing, FlashSettings},
};

#[derive(Debug)]
pub enum ScriptError {
    UnknownDirective(usize, String), // line number, content
    InvalidWpm(usize, String),
    InvalidEasing(usize, String),
    InvalidScale(usize, String),
    NoWpmDefined(usize), // a block has no wpm at all
    InvalidFlash(usize, String),
    ParseError(usize, String), // ← add this
}

impl std::fmt::Display for ScriptError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            ScriptError::UnknownDirective(l, s) => {
                write!(f, "Line {}: unknown directive '{}'", l, s)
            }
            ScriptError::InvalidWpm(l, s) => write!(f, "Line {}: invalid @wpm value '{}'", l, s),
            ScriptError::InvalidEasing(l, s) => {
                write!(f, "Line {}: invalid @easing value '{}'", l, s)
            }
            ScriptError::InvalidScale(l, s) => {
                write!(f, "Line {}: invalid @scale value '{}'", l, s)
            }
            ScriptError::NoWpmDefined(l) => write!(f, "Line {}: text block has no @wpm defined", l),
            ScriptError::InvalidFlash(l, s) => {
                write!(f, "Line {}: invalid @flash defined: '{}'", l, s)
            }
            ScriptError::ParseError(l, s) => write!(f, "Line {}: {}", l, s),
        }
    }
}

impl std::error::Error for ScriptError {}

/// The current "pen state" — settings that carry forward until overridden.
#[derive(Clone)]
struct State {
    wpm_from: Option<f32>,
    wpm_to: Option<f32>,
    easing: Option<Easing>,
    scale: Option<f32>,
    flash: Option<FlashSettings>,
    binaural: Option<BinauralSettings>, // ← new, persists until @binaural off
}

impl State {
    fn new() -> Self {
        Self {
            wpm_from: None,
            wpm_to: None,
            easing: None,
            scale: None,
            flash: None,
            binaural: None,
        }
    }
}

pub fn parse_script(source: &str) -> Result<Vec<Block>, ScriptError> {
    let mut blocks = Vec::new();
    let mut state = State::new();
    let mut text_buf = String::new();
    let mut buf_start_line = 0usize;

    // Helper: flush the current text buffer into a Block
    let flush = |text_buf: &mut String,
                 state: &State,
                 blocks: &mut Vec<Block>,
                 line_no: usize|
     -> Result<(), ScriptError> {
        let text = text_buf.trim().to_string();
        if text.is_empty() {
            text_buf.clear();
            return Ok(());
        }

        let wpm_from = state.wpm_from.ok_or(ScriptError::NoWpmDefined(line_no))?;
        let wpm_to = state.wpm_to.ok_or(ScriptError::NoWpmDefined(line_no))?;

        blocks.push(Block {
            text,
            wpm_from,
            wpm_to,
            easing: state.easing.clone(),
            scale: state.scale,
            flash: state.flash,
            binaural: state.binaural.clone(), // persists
        });

        text_buf.clear();
        Ok(())
    };

    for (line_no, line) in source.lines().enumerate() {
        let line = line.trim();

        // Comments and empty lines
        if line.starts_with('#') {
            continue;
        }

        if line.is_empty() {
            flush(&mut text_buf, &state, &mut blocks, buf_start_line)?;
            continue;
        }

        // Directives
        if let Some(end) = line.strip_prefix("@") {
            let (directive, rest) = end
                .split_once(' ')
                .map(|(d, r)| (d, r.trim()))
                .unwrap_or((&line[1..], ""));

            match directive {
                "wpm" => {
                    if let Some((from, to)) = rest.split_once("->") {
                        state.wpm_from = Some(
                            from.trim()
                                .parse()
                                .map_err(|_| ScriptError::InvalidWpm(line_no, rest.to_string()))?,
                        );
                        state.wpm_to = Some(
                            to.trim()
                                .parse()
                                .map_err(|_| ScriptError::InvalidWpm(line_no, rest.to_string()))?,
                        );
                    } else {
                        let wpm: f32 = rest
                            .parse()
                            .map_err(|_| ScriptError::InvalidWpm(line_no, rest.to_string()))?;
                        state.wpm_from = Some(wpm);
                        state.wpm_to = Some(wpm);
                    }
                }
                "easing" => {
                    state.easing = Some(match rest {
                        "linear" => Easing::Linear,
                        "instant" => Easing::Instant,
                        _ => return Err(ScriptError::InvalidEasing(line_no, rest.to_string())),
                    });
                }
                "scale" => {
                    state.scale = Some(
                        rest.parse()
                            .map_err(|_| ScriptError::InvalidScale(line_no, rest.to_string()))?,
                    );
                }
                "flash" => {
                    let mut accent_color = Color::rgb(255.0, 80.0, 80.0);
                    let mut bg_color = Color::rgb(255.0, 255.0, 255.0);

                    for raw in rest.split_whitespace() {
                        if let Some(val) = raw.strip_prefix("color=") {
                            accent_color = parse_color(val)
                                .map_err(|e| ScriptError::InvalidFlash(line_no, e))?;
                        } else if let Some(val) = raw.strip_prefix("bgColor=") {
                            bg_color = parse_color(val)
                                .map_err(|e| ScriptError::InvalidFlash(line_no, e))?;
                        }
                    }

                    state.flash = Some(FlashSettings {
                        accent_color,
                        bg_color,
                    });
                }
                "binaural" => {
                    // @binaural off
                    if rest == "off" {
                        state.binaural = None;
                        continue;
                    }

                    // @binaural carrier=200 beat=7 drone=180 volume=0.4
                    // All fields are optional — fall back to theta preset defaults
                    let mut settings = BinauralSettings::theta();

                    for token in rest.split_whitespace() {
                        if let Some(val) = token.strip_prefix("carrier=") {
                            settings.carrier_hz = parse_float(val)
                                .map_err(|e| ScriptError::ParseError(line_no, e))?;
                        } else if let Some(val) = token.strip_prefix("beat=") {
                            settings.beat_hz = parse_float(val)
                                .map_err(|e| ScriptError::ParseError(line_no, e))?;
                        } else if let Some(val) = token.strip_prefix("drone=") {
                            settings.drone_hz = parse_float(val)
                                .map_err(|e| ScriptError::ParseError(line_no, e))?;
                        } else if let Some(val) = token.strip_prefix("volume=") {
                            settings.volume = parse_float(val)
                                .map_err(|e| ScriptError::ParseError(line_no, e))?;
                        } else {
                            return Err(ScriptError::ParseError(
                                line_no,
                                format!("unknown @binaural parameter '{}'", token),
                            ));
                        }
                    }

                    state.binaural = Some(settings);
                }
                _ => return Err(ScriptError::UnknownDirective(line_no, line.to_string())),
            }

            continue;
        }

        // Regular text — accumulate into buffer
        if text_buf.is_empty() {
            buf_start_line = line_no;
        }
        if !text_buf.is_empty() {
            text_buf.push(' '); // join continuation lines with a space
        }
        text_buf.push_str(line);
    }

    // Flush any remaining text
    flush(&mut text_buf, &state, &mut blocks, buf_start_line)?;

    Ok(blocks)
}

fn parse_color(val: &str) -> Result<Color, String> {
    let parts: Vec<&str> = val.split(',').collect();
    if parts.len() != 3 {
        return Err(format!("expected 3 components, got {}", parts.len()));
    }
    let pixel: [f32; 3] = [
        parts[0]
            .trim()
            .parse()
            .map_err(|_| format!("invalid red component '{}'", parts[0].trim()))?,
        parts[1]
            .trim()
            .parse()
            .map_err(|_| format!("invalid green component '{}'", parts[1].trim()))?,
        parts[2]
            .trim()
            .parse()
            .map_err(|_| format!("invalid blue component '{}'", parts[2].trim()))?,
    ];

    Ok(Color::rgb(pixel[0], pixel[1], pixel[2]))
}

fn parse_float(val: &str) -> Result<f32, String> {
    val.trim()
        .parse::<f32>()
        .map_err(|_| format!("'{}' is not a valid number", val.trim()))
}
