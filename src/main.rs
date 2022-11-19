use macroquad::prelude::*;

use raycaster::{self, State, MAX_MAP_SIZE, SCREEN_SIZE};

// the factor for the size of the top down cells on screen
const TOP_DOWN_CELL_SIZE: usize = SCREEN_SIZE as usize / MAX_MAP_SIZE;

// length factor to visualize where the player is looking from dx/dy
const VIEW_LINE_LENGTH_FACTOR: f32 = 300.0;


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

    loop {
        draw_top_down(&state);
        draw_pov(&state);

        // move player around a bit
        if is_key_down(KeyCode::W) {
            state.player_mut().move_forward();
        }

        if is_key_down(KeyCode::S) {
            state.player_mut().move_backward();
        }

        if is_key_down(KeyCode::A) {
            state.player_mut().look_left();
        }

        if is_key_down(KeyCode::D) {
            state.player_mut().look_right();
        }

        next_frame().await;
    }
}

// pov is drawn on the right so everything needs to be shifted
const SCREEN_OFFSET: f32 = SCREEN_SIZE as f32;

fn draw_pov(state: &State) {
}

fn draw_top_down(state: &State) {
    // draw map cells
    for (y, values) in state.map().iter().enumerate() {
        let y_adj: f32 = (y * TOP_DOWN_CELL_SIZE) as f32;

        for (x, value) in values.iter().enumerate() {
            let x_adj: f32 = (x * TOP_DOWN_CELL_SIZE) as f32;

            let color = if *value { DARKBLUE } else { GRAY };

            draw_rectangle(
                x_adj,
                y_adj,
                TOP_DOWN_CELL_SIZE as f32,
                TOP_DOWN_CELL_SIZE as f32,
                color,
            );
        }
    }

    // draw grid lines
    for x in 0..(MAX_MAP_SIZE + 1) {
        let x: f32 = (x * TOP_DOWN_CELL_SIZE) as f32;
        draw_line(x, 0.0, x, SCREEN_SIZE as f32, 1.0, DARKGRAY);
        draw_line(0.0, x, SCREEN_SIZE as f32, x, 1.0, DARKGRAY);
    }

    // draw player
    let player = state.player();
    let (adj_player_x, adj_player_y) = (
        player.x * TOP_DOWN_CELL_SIZE as f32,
        player.y * TOP_DOWN_CELL_SIZE as f32,
    );

    // draw view line
    draw_line(
        adj_player_x,
        adj_player_y,
        adj_player_x + (player.dx * VIEW_LINE_LENGTH_FACTOR),
        adj_player_y + (player.dy * VIEW_LINE_LENGTH_FACTOR),
        2.0,
        YELLOW,
    );

    // draw player circle
    draw_circle(adj_player_x, adj_player_y, 4.3, RED);
}
