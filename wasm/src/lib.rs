use wasm_bindgen::prelude::*;

// Handy macro for calling console.log from rust
macro_rules! console_log {
    ($($t:tt)*) => {
        (web_sys::console::log_1(&format!($($t)*).into()))
    };
}

#[wasm_bindgen]
pub fn hello_world() {
    console_log!("Hello from WASM!");
}