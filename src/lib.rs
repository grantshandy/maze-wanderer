#![no_std]

use core::f32::consts::{PI, TAU};

use libm::{cosf, sinf};

pub const SCREEN_SIZE: i32 = 160;

pub const FOV: f32 = PI / 3.0;
pub const HALF_FOV: f32 = FOV / 2.0;

pub const MAP_SIZE: i32 = 8;
const MAP_BUFFER: usize = MAP_SIZE as usize * MAP_SIZE as usize;

const MOVE_STEP: f32 = 0.05;
const LOOK_STEP: f32 = 0.05;

#[rustfmt::skip]
const DEFAULT_MAP: [bool; MAP_BUFFER] = [
    true, true, true, true, true, true, true, true,
    true, false, false, false, true, true, false, true,
    true, false, false, false, false, false, false, true,
    true, false, false, false, false, true, true, true,
    true, true, false, false, false, false, false, true,
    true, false, false, false, true, false, false, true,
    true, false, false, false, true, false, false, true,
    true, true, true, true, true, true, true, true
];

pub struct State {
    pub player_x: f32,
    pub player_y: f32,
    pub player_angle: f32,
    pub player_forward: bool,
    pub map: [bool; MAP_BUFFER],
}

impl Default for State {
    fn default() -> Self {
        Self {
            player_x: 3.5,
            player_y: 3.5,
            player_angle: PI,
            player_forward: true,
            map: DEFAULT_MAP,
        }
    }
}

impl State {
    // update the player's movement
    pub fn update_character(&mut self, left: bool, right: bool, forwards: bool, backwards: bool) {
        if left {
            self.player_angle += LOOK_STEP;
        }

        if right {
            self.player_angle -= LOOK_STEP;
        }

        if forwards {
            self.player_forward = true;
            self.player_x += cosf(self.player_angle) * MOVE_STEP;
            self.player_y += -sinf(self.player_angle) * MOVE_STEP;
        }

        if backwards {
            self.player_forward = false;
            self.player_x -= cosf(self.player_angle) * MOVE_STEP;
            self.player_y -= -sinf(self.player_angle) * MOVE_STEP;
        }

        if self.map[(self.player_y as i32 * MAP_SIZE + self.player_x as i32) as usize] {
            if self.player_forward {
                self.player_x -= cosf(self.player_angle) * MOVE_STEP;
                self.player_y -= -sinf(self.player_angle) * MOVE_STEP;
            } else {
                self.player_x += cosf(self.player_angle) * MOVE_STEP;
                self.player_y += -sinf(self.player_angle) * MOVE_STEP;
            }
        }

        if self.player_angle > TAU {
            self.player_angle = 0.0;
        } else if self.player_angle < 0.0 {
            self.player_angle = TAU;
        }
    }

    pub fn raycast(&self) -> (f32, f32) {
        let start_angle = self.player_angle;

        for depth in 0..MAP_SIZE {
            let target_x = self.player_x + cosf(start_angle) * depth as f32;
            let target_y = self.player_y - sinf(start_angle) * depth as f32;

            let col: i32 = target_x as i32;
            let row: i32 = target_y as i32;

            let square: i32 = (row * MAP_SIZE) + col;

            if self.map[square as usize] {
                return (target_x, target_y);
            }
        }

        (0.0, 0.0)
    }
}
