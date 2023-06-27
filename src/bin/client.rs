use wgpu2::{window, State};

// how to start from local program
fn main() {
    window::run(State::init);
}

// how to start from website
#[cfg(target_arch = "wasm32")]
use wasm_bindgen::prelude::wasm_bindgen;
#[cfg_attr(target_arch="wasm32", wasm_bindgen(start))]
async fn wasm_main() {
    window::run_async(State::init);
}
