use image::Rgb;
use serde::{Deserialize, Deserializer};

#[derive(Debug, Clone, Copy)]
pub struct Color {
    pub r: f32,
    pub g: f32,
    pub b: f32,
}

impl Color {
    pub const fn rgb(r: f32, g: f32, b: f32) -> Self {
        Self { r, g, b }
    }

    pub const fn hex(value: u32) -> Self {
        Self {
            r: ((value >> 16) & 0xFF) as f32,
            g: ((value >> 8) & 0xFF) as f32,
            b: (value & 0xFF) as f32,
        }
    }

    pub const fn pixel(value: [u8; 3]) -> Self {
        Self {
            r: value[0] as f32,
            g: value[1] as f32,
            b: value[2] as f32,
        }
    }

    pub const fn str(s: &[u8]) -> Self {
        const fn hex_digit(c: u8) -> u8 {
            match c {
                b'0'..=b'9' => c - b'0',
                b'a'..=b'f' => c - b'a' + 10,
                b'A'..=b'F' => c - b'A' + 10,
                _ => panic!("Invalid hex digit in color string"),
            }
        }

        let offset = if s[0] == b'#' { 1 } else { 0 };
        let len = s.len() - offset;

        let (r, g, b) = if len == 3 {
            // Shorthand: #fcc -> #ffcccc
            let r_digit = hex_digit(s[offset]);
            let g_digit = hex_digit(s[offset + 1]);
            let b_digit = hex_digit(s[offset + 2]);
            (
                r_digit * 16 + r_digit,
                g_digit * 16 + g_digit,
                b_digit * 16 + b_digit,
            )
        } else if len == 6 {
            // Standard: #ffcccc
            (
                hex_digit(s[offset]) * 16 + hex_digit(s[offset + 1]),
                hex_digit(s[offset + 2]) * 16 + hex_digit(s[offset + 3]),
                hex_digit(s[offset + 4]) * 16 + hex_digit(s[offset + 5]),
            )
        } else {
            panic!("Hex color must be 3 or 6 digits");
        };

        Color {
            r: r as f32,
            g: g as f32,
            b: b as f32,
        }
    }

    /// Creates a Color from a normalized [f32; 3] array
    pub fn from_normalized(channels: [f32; 3]) -> Self {
        Self {
            r: channels[0] * 255.0,
            g: channels[1] * 255.0,
            b: channels[2] * 255.0,
        }
    }

    pub fn lerp(self, other: Color, t: f32) -> Color {
        Color {
            r: self.r + (other.r - self.r) * t,
            g: self.g + (other.g - self.g) * t,
            b: self.b + (other.b - self.b) * t,
        }
    }

    pub fn scale(self, factor: f32) -> Color {
        Color {
            r: self.r * factor,
            g: self.g * factor,
            b: self.b * factor,
        }
    }

    /// Returns the color with values scaled to 0.0 - 1.0
    pub fn normalized(&self) -> [f32; 3] {
        [self.r / 255.0, self.g / 255.0, self.b / 255.0]
    }

    pub fn to_pixel(self) -> [u8; 3] {
        [
            self.r.clamp(0.0, 255.0) as u8,
            self.g.clamp(0.0, 255.0) as u8,
            self.b.clamp(0.0, 255.0) as u8,
        ]
    }

    pub fn to_array(self) -> [f32; 3] {
        [self.r, self.g, self.b]
    }

    pub fn to_rgb(self) -> Rgb<u8> {
        Rgb(self.to_pixel())
    }
}

// In TOML: color_slow = 0x000000  or  color_slow = [0.0, 0.0, 0.0]
// Simplest: just use hex integers in TOML

impl<'de> Deserialize<'de> for Color {
    fn deserialize<D: Deserializer<'de>>(d: D) -> Result<Self, D::Error> {
        // Accept either a hex integer or an [r, g, b] array
        #[derive(Deserialize)]
        #[serde(untagged)]
        enum Raw {
            Hex(u32),
            Array([f32; 3]),
            String(String),
        }
        match Raw::deserialize(d)? {
            Raw::Hex(v) => Ok(Color::hex(v)),
            Raw::Array([r, g, b]) => Ok(Color::rgb(r, g, b)),
            Raw::String(s) => Ok(Color::str(s.as_bytes())),
        }
    }
}
