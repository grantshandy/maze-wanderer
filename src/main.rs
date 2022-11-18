use macroquad::prelude::*;

use raycaster::{self, State, MAX_MAP_SIZE, SCREEN_SIZE};

fn window_conf() -> Conf {
    Conf {
        window_title: "Raycaster Demo".to_owned(),
        // window is twice the width for the demo
        window_width: (SCREEN_SIZE * 2) as i32,
        window_height: SCREEN_SIZE as i32,
        window_resizable: false,
        ..Default::default()
    }
}

#[macroquad::main(window_conf)]
async fn main() {
    let mut state = State::default();
    let cell_size: usize = SCREEN_SIZE as usize / MAX_MAP_SIZE;

    loop {
        // draw map
        for (y, values) in state.map().iter().enumerate() {
            let y_adj: f32 = (y * cell_size) as f32;

            for (x, value) in values.iter().enumerate() {
                let x_adj: f32 = (x * cell_size) as f32;

                let color = if *value { DARKBLUE } else { GRAY };

                draw_rectangle(x_adj, y_adj, cell_size as f32, cell_size as f32, color);
            }
        }

        // draw grid lines
        for x in 0..(MAX_MAP_SIZE + 1) {
            let x: f32 = (x * cell_size) as f32;
            draw_line(x, 0.0, x, SCREEN_SIZE as f32, 1.0, DARKGRAY);
            draw_line(0.0, x, SCREEN_SIZE as f32, x, 1.0, DARKGRAY);
        }

        // draw player
        let player = state.player();
        draw_circle(
            player.pos_x * cell_size as f32,
            player.pos_y * cell_size as f32,
            4.3,
            RED,
        );

        // move player
        if is_key_down(KeyCode::W) {
            state.player_mut().move_forward();
        }

        if is_key_down(KeyCode::S) {
            state.player_mut().move_backward();
        }

        if is_key_down(KeyCode::A) {
        }

        if is_key_down(KeyCode::D) {
        }

        next_frame().await;
    }
}
