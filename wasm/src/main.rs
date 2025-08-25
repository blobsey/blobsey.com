use macroquad::prelude::*;
mod blob;
mod constants;

#[macroquad::main("blob")]
async fn main() {
    let initial_screen_width = screen_width();
    let initial_screen_height = screen_height();

    let mut blob = blob::Blob::new(Vec2 {
        x: initial_screen_width / 2.0,
        y: initial_screen_height / 2.0,
    });
    loop {
        clear_background(WHITE);

        blob.update(get_frame_time().min(1.0 / 120.0));
        blob.draw();
        next_frame().await
    }
}
