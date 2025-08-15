
use macroquad::prelude::*;

// Handy macro for calling console.log from rust
// macro_rules! console_log {
//     ($($t:tt)*) => {
//         (web_sys::console::log_1(&format!($($t)*).into()))
//     };
// }

#[macroquad::main("Blobsey")]
async fn main() {
    loop {
        clear_background(WHITE);
        draw_text("Welcome to blobsey.com", 90.0, 90.0, 90.0, BLACK);

        next_frame().await
    }
}
         