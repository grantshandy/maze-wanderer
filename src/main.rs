use macroquad::prelude::*;

use raycaster::{self, State};

fn window_conf() -> Conf {
    Conf {
        window_title: "Raycaster".to_owned(),
        window_width: raycaster::SCREEN_WIDTH as i32,
        window_height: raycaster::SCREEN_HEIGHT as i32,
        window_resizable: false,
        ..Default::default()
    }
}

#[macroquad::main(window_conf)]
async fn main() {
    let state = State::default();

    println!("{:?}", state.world_map());

    loop {
        next_frame().await;
    }
}
