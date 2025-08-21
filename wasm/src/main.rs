use macroquad::prelude::*;
mod blob;
mod constants;

#[macroquad::main("blob")]
async fn main() {
    // console_error_panic_hook::set_once();

    let mut blob = blob::Blob::new(Vec2::new(400.0, 300.0));
    loop {
        clear_background(WHITE);
        blob.update(get_frame_time());
        blob.draw();
        next_frame().await
    }
}
