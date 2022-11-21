use std::f32::consts::PI;

use libm::{cosf, sinf};
use macroquad::prelude::*;

const SCREEN_HEIGHT: i32 = 480;
const SCREEN_WIDTH: i32 = SCREEN_HEIGHT * 2;
const MAP_SIZE: i32 = 8;
const TILE_SIZE: i32 = (SCREEN_WIDTH / 2) / MAP_SIZE;
const MAX_DEPTH: i32 = MAP_SIZE * TILE_SIZE;
const FOV: f32 = PI / 3.0;
const HALF_FOV: f32 = FOV / 2.0;
const CASTED_RAYS: i32 = 120;
const STEP_ANGLE: f32 = FOV / CASTED_RAYS as f32;
const SCALE: i32 = (SCREEN_WIDTH / 2) / CASTED_RAYS;

const MAP: &str = "#########   ## ##      ##    #####     ##   #  ##   #  #########";

fn window_conf() -> Conf {
    Conf {
        window_title: "Raycaster Demo".to_owned(),
        window_width: SCREEN_WIDTH,
        window_height: SCREEN_HEIGHT,
        window_resizable: false,
        ..Default::default()
    }
}

#[macroquad::main(window_conf)]
async fn main() {
    let mut player_x: f32 = ((SCREEN_WIDTH / 2) / 2) as f32;
    let mut player_y: f32 = ((SCREEN_WIDTH / 2) / 2) as f32;
    let mut player_angle: f32 = PI;
    let mut forward = true;

    loop {
        let col: i32 = player_x as i32 / TILE_SIZE;
        let row: i32 = player_y as i32 / TILE_SIZE;

        let square: i32 = (row * MAP_SIZE) + col;
        println!("{square}");

        if MAP.chars().nth(square as usize) == Some('#') {
            if forward {
                player_x -= -sinf(player_angle) * 5.0;
                player_y -= cosf(player_angle) * 5.0;
            } else {
                player_x += -sinf(player_angle) * 5.0;
                player_y += cosf(player_angle) * 5.0;
            }
        }

        // draw background colors
        draw_rectangle(
            0.0,
            0.0,
            SCREEN_HEIGHT as f32,
            SCREEN_HEIGHT as f32,
            Color::from_rgba(0, 0, 0, 255),
        );
        draw_rectangle(
            480.0,
            (SCREEN_HEIGHT / 2) as f32,
            SCREEN_HEIGHT as f32,
            SCREEN_HEIGHT as f32,
            Color::from_rgba(100, 0, 0, 255),
        );
        draw_rectangle(
            480.0,
            (-SCREEN_HEIGHT / 2) as f32,
            SCREEN_HEIGHT as f32,
            SCREEN_HEIGHT as f32,
            Color::from_rgba(200, 0, 0, 255),
        );

        draw_map(player_x, player_y, player_angle);
        cast_rays(player_x, player_y, player_angle);

        if is_key_down(KeyCode::Left) {
            player_angle -= 0.1;
        }

        if is_key_down(KeyCode::Right) {
            player_angle += 0.1;
        }

        if is_key_down(KeyCode::Up) {
            forward = true;
            player_x += -sinf(player_angle) * 5.0;
            player_y += cosf(player_angle) * 5.0;
        }

        if is_key_down(KeyCode::Down) {
            forward = false;
            player_x -= -sinf(player_angle) * 5.0;
            player_y -= cosf(player_angle) * 5.0;
        }

        draw_text(
            &format!("{}", get_fps()),
            1.0,
            18.0,
            30.0,
            Color::from_rgba(255, 255, 255, 255),
        );

        next_frame().await;
    }
}

fn draw_map(player_x: f32, player_y: f32, player_angle: f32) {
    for row in 0..MAP_SIZE {
        for col in 0..MAP_SIZE {
            let square = row * MAP_SIZE + col;

            if MAP.chars().nth(square as usize) == Some('#') {
                draw_rectangle(
                    (col * TILE_SIZE) as f32,
                    (row * TILE_SIZE) as f32,
                    (TILE_SIZE - 2) as f32,
                    (TILE_SIZE - 2) as f32,
                    Color::from_rgba(200, 200, 200, 255),
                );
            } else {
                draw_rectangle(
                    (col * TILE_SIZE) as f32,
                    (row * TILE_SIZE) as f32,
                    (TILE_SIZE - 2) as f32,
                    (TILE_SIZE - 2) as f32,
                    Color::from_rgba(100, 100, 100, 255),
                );
            }
        }
    }

    draw_circle(player_x, player_y, 8.0, Color::from_rgba(255, 0, 0, 255));
    draw_line(
        player_x,
        player_y,
        player_x - sinf(player_angle) * 50.0,
        player_y + cosf(player_angle) * 50.0,
        3.0,
        Color::from_rgba(0, 255, 0, 255),
    );
    draw_line(
        player_x,
        player_y,
        player_x - sinf(player_angle - HALF_FOV) * 50.0,
        player_y + cosf(player_angle - HALF_FOV) * 50.0,
        3.0,
        Color::from_rgba(0, 255, 0, 255),
    );
    draw_line(
        player_x,
        player_y,
        player_x - sinf(player_angle + HALF_FOV) * 50.0,
        player_y + cosf(player_angle + HALF_FOV) * 50.0,
        3.0,
        Color::from_rgba(0, 255, 0, 255),
    );
}

fn cast_rays(player_x: f32, player_y: f32, player_angle: f32) {
    let mut start_angle: f32 = player_angle - HALF_FOV;

    for ray in 0..CASTED_RAYS {
        for depth in 0..MAX_DEPTH {
            let target_x = player_x - sinf(start_angle) * depth as f32;
            let target_y = player_y + cosf(start_angle) * depth as f32;

            let col: i32 = target_x as i32 / TILE_SIZE;
            let row: i32 = target_y as i32 / TILE_SIZE;

            let square: i32 = (row * MAP_SIZE) + col;

            if MAP.chars().nth(square as usize) == Some('#') {
                draw_rectangle(
                    (col * TILE_SIZE) as f32,
                    (row * TILE_SIZE) as f32,
                    (TILE_SIZE - 2) as f32,
                    (TILE_SIZE - 2) as f32,
                    Color::from_rgba(0, 255, 0, 255),
                );
                draw_line(
                    player_x,
                    player_y,
                    target_x,
                    target_y,
                    1.0,
                    Color::from_rgba(255, 255, 0, 255),
                );

                let depth_alt = depth as f32 * cosf(player_angle - start_angle);
                let mut wall_height = 21000.0 / (depth_alt + 0.0001);

                if wall_height > SCREEN_HEIGHT as f32 {
                    wall_height = SCREEN_HEIGHT as f32;
                }

                draw_rectangle(
                    (SCREEN_HEIGHT + ray * SCALE) as f32,
                    (SCREEN_HEIGHT / 2) as f32 - wall_height / 2.0,
                    SCALE as f32,
                    wall_height,
                    Color::from_rgba(30, 30, 30, 255),
                );

                break;
            }
        }

        start_angle += STEP_ANGLE;
    }
}
