use libm::{cosf, sinf};
use macroquad::prelude::*;
use raycaster::{State, HALF_FOV, MAP_SIZE, SCREEN_SIZE};

// multiply inner resolution to look nice for development
const DISPLAY_SCALE: i32 = 4;
pub const TILE_SIZE: i32 = SCREEN_SIZE / MAP_SIZE;

// colors
const SKY: Vec4 = Vec4::new(0.243, 0.596, 0.737, 1.0);
const GROUND: Vec4 = Vec4::new(0.6, 0.57, 0.45, 1.0);

fn window_conf() -> Conf {
    Conf {
        window_title: "Raycaster".to_string(),
        window_width: (SCREEN_SIZE * 2) * DISPLAY_SCALE,
        window_height: SCREEN_SIZE * DISPLAY_SCALE,
        window_resizable: false,
        ..Default::default()
    }
}

#[macroquad::main(window_conf)]
async fn main() {
    let mut state = State::default();

    loop {
        state.update_character(
            is_key_down(KeyCode::Left),
            is_key_down(KeyCode::Right),
            is_key_down(KeyCode::Up),
            is_key_down(KeyCode::Down),
        );

        draw_map(&state);

        // println!("{}", state.player_angle);
        // println!(
        //     "current square: {}",
        //     (state.player_y as i32 * MAP_SIZE + state.player_x as i32)
        // );

        next_frame().await;
    }
}

fn draw_map(state: &State) {
    // draw background
    draw_rectangle(
        0.0,
        0.0,
        (SCREEN_SIZE * DISPLAY_SCALE) as f32,
        (SCREEN_SIZE * DISPLAY_SCALE) as f32,
        BLACK,
    );
    draw_rectangle(
        (SCREEN_SIZE * DISPLAY_SCALE) as f32,
        0.0,
        (SCREEN_SIZE * DISPLAY_SCALE) as f32,
        (SCREEN_SIZE * DISPLAY_SCALE) as f32 / 2.0,
        Color::from_vec(SKY),
    );
    draw_rectangle(
        (SCREEN_SIZE * DISPLAY_SCALE) as f32,
        (SCREEN_SIZE * DISPLAY_SCALE) as f32 / 2.0,
        (SCREEN_SIZE * DISPLAY_SCALE) as f32,
        (SCREEN_SIZE * DISPLAY_SCALE) as f32 / 2.0,
        Color::from_vec(GROUND),
    );

    // draw grid
    for row in 0..MAP_SIZE {
        for col in 0..MAP_SIZE {
            let grid_color = if state.map[(row * MAP_SIZE + col) as usize] {
                Color::from_rgba(200, 200, 200, 255)
            } else {
                Color::from_rgba(100, 100, 100, 255)
            };

            draw_rectangle(
                (col * TILE_SIZE * DISPLAY_SCALE) as f32,
                (row * TILE_SIZE * DISPLAY_SCALE) as f32,
                (DISPLAY_SCALE * TILE_SIZE - 2) as f32,
                (DISPLAY_SCALE * TILE_SIZE - 2) as f32,
                grid_color,
            );
        }
    }

    // draw direction indicators
    draw_line(
        state.player_x * (DISPLAY_SCALE * TILE_SIZE) as f32,
        state.player_y * (DISPLAY_SCALE * TILE_SIZE) as f32,
        (state.player_x - sinf(state.player_angle - HALF_FOV)) * (DISPLAY_SCALE * TILE_SIZE) as f32,
        (state.player_y + cosf(state.player_angle - HALF_FOV)) * (DISPLAY_SCALE * TILE_SIZE) as f32,
        3.0,
        GREEN,
    );
    draw_line(
        state.player_x * (DISPLAY_SCALE * TILE_SIZE) as f32,
        state.player_y * (DISPLAY_SCALE * TILE_SIZE) as f32,
        (state.player_x - sinf(state.player_angle + HALF_FOV)) * (DISPLAY_SCALE * TILE_SIZE) as f32,
        (state.player_y + cosf(state.player_angle + HALF_FOV)) * (DISPLAY_SCALE * TILE_SIZE) as f32,
        3.0,
        GREEN,
    );

    // draw map ray from center of player's vision
    let (target_x, target_y) = &state.raycast();
    draw_line(
        state.player_x * (DISPLAY_SCALE * TILE_SIZE) as f32,
        state.player_y * (DISPLAY_SCALE * TILE_SIZE) as f32,
        target_x * (DISPLAY_SCALE * TILE_SIZE) as f32,
        target_y * (DISPLAY_SCALE * TILE_SIZE) as f32,
        4.0,
        YELLOW,
    );
    println!("{:?}", (target_x, target_y));

    // draw player
    draw_circle(
        state.player_x * (DISPLAY_SCALE * TILE_SIZE) as f32,
        state.player_y * (DISPLAY_SCALE * TILE_SIZE) as f32,
        8.0,
        RED,
    );
}
