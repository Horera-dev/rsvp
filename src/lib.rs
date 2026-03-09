// // 1. Declare your modules so they are part of the library
// pub mod color;
// pub mod config;
// pub mod constant;
// pub mod content_parser;
// pub mod io;
// pub mod processing;
// pub mod renderer;
// pub mod rsvp;
// pub mod scheduler;
// pub mod spiral;

// use ab_glyph::{Font, FontRef};
// use wasm_bindgen::prelude::*;

// use crate::{
//     config::Config,
//     processing::render_instruction,
//     scheduler::{FrameInstruction, compute_padding, compute_schedule},
//     spiral::{SpiralCache, create_spiral_cache},
// };

// #[wasm_bindgen]
// extern "C" {
//     #[wasm_bindgen(js_namespace = console)]
//     fn log(s: &str);
// }

// macro_rules! console_log {
//     ($($t:tt)*) => (log(&format_args!($($t)*).to_string()))
// }

// #[wasm_bindgen]
// pub struct WasmRenderer {
//     width: u32,
//     height: u32,
//     pixels: Vec<u8>,
//     font: FontRef<'static>,
//     spiral_cache: SpiralCache,
//     instructions: Vec<FrameInstruction>,
//     config: Config,
// }

// #[wasm_bindgen]
// impl WasmRenderer {
//     #[wasm_bindgen(constructor)]
//     pub fn new(
//         width: u32,
//         height: u32,
//         font_bytes: Vec<u8>,
//         toml_config: String,
//     ) -> Result<WasmRenderer, JsValue> {
//         // This line sends Rust panics to the F12 Browser Console!
//         console_error_panic_hook::set_once();

//         // 1. Parse Config (assuming you use toml crate)
//         let mut config: Config =
//             toml::from_str(&toml_config).map_err(|e| JsValue::from_str(&e.to_string()))?;

//         config.settings.video.height = height;
//         config.settings.gif.height = height;
//         config.settings.video.width = width;
//         config.settings.gif.width = width;

//         // 2. Load Font (leak the bytes to get 'static lifetime required for FontRef)
//         let font_vec = font_bytes.to_vec();
//         let data = font_vec.into_boxed_slice();
//         let font_data: &'static [u8] = Box::leak(data);
//         let font: FontRef<'static> = FontRef::try_from_slice(font_data).expect("Font load failed");

//         // 3. Precompute Cache and Schedule
//         let spiral_cache = create_spiral_cache(width, height);
//         let mut instructions = compute_schedule(&config);

//         // DEBUG: Try to draw a single character's bounding box
//         let test_glyph = font.glyph_id('B').with_scale(48.0);
//         console_log!("Font Loaded! 'B' glyph id: {:?}", test_glyph);

//         // Add padding instructions
//         let padding = compute_padding(&config, instructions.len() as u32);
//         instructions.extend(padding.instructions);

//         Ok(Self {
//             width,
//             height,
//             pixels: vec![0; (width * height * 4) as usize],
//             font,
//             spiral_cache,
//             instructions,
//             config,
//         })
//     }

//     pub fn render(&mut self, timestamp_secs: f32) -> *const u8 {
//         // Find the correct instruction based on the current time
//         // or just use the current timestamp to drive the spiral
//         let frame_idx = (timestamp_secs * 60.0) as usize; // Assuming 60fps
//         let looped_frame = frame_idx % self.instructions.len();

//         let test_glyph = self.font.glyph_id('A').with_scale(48.0);
//         console_log!("Font Loaded! 'A' glyph id: {:?}", test_glyph);

//         if let Some(instruction) = self.instructions.get(looped_frame).cloned() {
//             // We need a helper to draw into our Vec<u8> (RGBA) buffer
//             self.draw_frame(&instruction);
//         }

//         self.pixels.as_ptr()
//     }

//     fn draw_frame(&mut self, instruction: &FrameInstruction) {
//         // This is where you adapt your old 'render_instruction'
//         // But instead of RgbImage, you are writing to self.pixels (RGBA)
//         let img = render_instruction(instruction, &self.config, &self.spiral_cache, &self.font);
//         let clean_img = img.unwrap();

//         // chunks_exact_mut(4) gives us [R, G, B, A] for every pixel in self.pixels
//         // .pixels() gives us [R, G, B] for every pixel in the RgbImage
//         self.pixels
//             .chunks_exact_mut(4)
//             .zip(clean_img.pixels())
//             .for_each(|(rgba, rgb)| {
//                 rgba[0] = rgb[0]; // R
//                 rgba[1] = rgb[1]; // G
//                 rgba[2] = rgb[2]; // B
//                 rgba[3] = 255; // A (Opaque)
//             });

//         // Inside draw_frame, after you copy the RgbImage
//         // Draw a 50x50 red square in the top left corner manually
//         for y in 0..50 {
//             for x in 0..50 {
//                 let i = (y * self.width + x) as usize * 4;
//                 self.pixels[i] = 255; // R
//                 self.pixels[i + 1] = 0; // G
//                 self.pixels[i + 2] = 0; // B
//                 self.pixels[i + 3] = 255; // A
//             }
//         }
//     }
// }
