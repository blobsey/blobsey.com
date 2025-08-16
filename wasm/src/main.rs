use macroquad::prelude::*;
mod blob;
mod constants;

// Handy macro for calling console.log from rust
// macro_rules! console_log {
//     ($($t:tt)*) => {
//         (web_sys::console::log_1(&format!($($t)*).into()))
//     };
// }

#[macroquad::main("blob")]
async fn main() {
    let mut blob = blob::Blob::new(Vec2::new(400.0, 300.0));
    loop {
        clear_background(WHITE);
        blob.update(get_frame_time());
        blob.draw();
        next_frame().await
    }
}
