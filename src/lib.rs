#[cfg(target_arch="wasm32")]
use wasm_bindgen::prelude::*;

pub mod win;
pub mod client;
pub mod simple;
pub mod square;
pub mod mat;
pub mod rrs;
pub mod textured;
pub mod texture;
pub mod util;
