
use macroquad::prelude::*;

// Handy macro for calling console.log from rust
// macro_rules! console_log {
//     ($($t:tt)*) => {
//         (web_sys::console::log_1(&format!($($t)*).into()))
//     };
// }

#[macroquad::main("MyGame")]
async fn main() {
    loop {
        clear_background(RED);

        draw_line(40.0, 40.0, 100.0, 200.0, 15.0, BLUE);
        draw_rectangle(screen_width() / 2.0 - 60.0, 100.0, 120.0, 60.0, GREEN);

        draw_text("Hello, Macroquad!", 20.0, 20.0, 30.0, DARKGRAY);

        next_frame().await
    }
}
         