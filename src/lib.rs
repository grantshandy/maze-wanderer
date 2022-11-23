#![no_std]

use core::f32::consts::PI;

use heapless::Vec;
use libm::{ceilf, cosf, floorf, powf, sinf, sqrtf, tanf, fabsf};

pub const SCREEN_SIZE: i32 = 160;

pub const FOV: f32 = PI / 2.0;
pub const HALF_FOV: f32 = FOV / 2.0;
const NUMBER_OF_RAYS: usize = SCREEN_SIZE as usize;

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
    // player state
    pub player_x: f32,
    pub player_y: f32,
    pub player_angle: f32,
    
    pub map: [bool; MAP_BUFFER],
}

impl Default for State {
    fn default() -> Self {
        Self {
            player_x: 3.5,
            player_y: 3.5,
            player_angle: PI,
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

        // store this incase we might want to restore position if we hit a wall
        let previous_position = (self.player_x, self.player_y);

        if forwards {
            self.player_x += cosf(self.player_angle) * MOVE_STEP;
            self.player_y += -sinf(self.player_angle) * MOVE_STEP;
        }

        if backwards {
            self.player_x -= cosf(self.player_angle) * MOVE_STEP;
            self.player_y -= -sinf(self.player_angle) * MOVE_STEP;
        }

        // move back to our original position if we moved into a wall
        if self.map[(self.player_y as i32 * MAP_SIZE + self.player_x as i32) as usize] {
            self.player_x = previous_position.0;
            self.player_y = previous_position.1;
        }

        // reset our angle if we go below 0 or above τ.
        // if self.player_angle > TAU {
        //     self.player_angle = 0.0;
        // } else if self.player_angle < 0.0 {
        //     self.player_angle = TAU;
        // }
    }

    // get all rays
    pub fn get_rays(&self) -> Vec<Ray, NUMBER_OF_RAYS> {
        let mut rays = Vec::new();

        let angle_step = FOV / NUMBER_OF_RAYS as f32;
        let initial_angle = self.player_angle - HALF_FOV;

        for num in 0..NUMBER_OF_RAYS {
            let angle = initial_angle + num as f32 * angle_step;

            rays.push(self.raycast(angle)).unwrap();
        }

        // rays.push(self.raycast(self.player_angle)).unwrap();

        rays
    }

    /// return the location of the first intersection
    fn raycast(&self, angle: f32) -> Ray {
        let vert = self.get_vert_intersection(angle);
        let horiz = self.get_horiz_intersection(angle);

        if vert.distance < horiz.distance {
            vert
        } else {
            horiz
        }
    }

    // get a vector containing the first horizontal intersection with the walls
    fn get_horiz_intersection(&self, angle: f32) -> Ray {
        let up = fabsf(floorf(angle / PI) % 2.0) == 0.0;

        // where the ray first intersects with the grid
        let first_y = if up {
            floorf(self.player_y) - self.player_y
        } else {
            ceilf(self.player_y) - self.player_y
        };
        let first_x = first_y / -tanf(angle);

        // individual steps
        let ya = if up { -1.0 } else { 1.0 };
        let xa = ya / -tanf(angle);

        // our final vectors
        let mut next_x = first_x;
        let mut next_y = first_y;

        for _ in 0..MAP_SIZE {
            // check the cell the ray is currently in
            let cell_x = (next_x + self.player_x) as i32;
            let cell_y = if up {
                (next_y + self.player_y) as i32 - 1
            } else {
                (next_y + self.player_y) as i32
            };
            let square = (cell_y * MAP_SIZE) + cell_x;

            if in_bounds(square) && self.map[square as usize] {
                break;
            }

            next_x += xa;
            next_y += ya;
        }

        Ray {
            x: next_x + self.player_x,
            y: next_y + self.player_y,
            distance: distance(
                self.player_x,
                self.player_y,
                self.player_x + next_x,
                self.player_y + next_y,
            ),
            angle,
        }
    }

    // get a vector containing the first vertical intersection with the walls
    fn get_vert_intersection(&self, angle: f32) -> Ray {
        let right = fabsf(floorf((angle - (PI / 2.0)) / PI) % 2.0) != 0.0;

        // where the ray first intersects with the grid
        let first_x = if right {
            ceilf(self.player_x) - self.player_x
        } else {
            floorf(self.player_x) - self.player_x
        };
        let first_y = -tanf(angle) * first_x;

        // individual steps
        let xa = if right { 1.0 } else { -1.0 };
        let ya = xa * -tanf(angle);

        // our final_vectors
        let mut next_x = first_x;
        let mut next_y = first_y;

        for _ in 0..MAP_SIZE {
            // check the cell the ray is currently in
            let cell_x = if right {
                (next_x + self.player_x) as i32
            } else {
                (next_x + self.player_x) as i32 - 1
            };
            let cell_y = (next_y + self.player_y) as i32;
            let square = (cell_y * MAP_SIZE) + cell_x;

            if in_bounds(square) && self.map[square as usize] {
                break;
            }

            next_x += xa;
            next_y += ya;
        }

        Ray {
            x: next_x + self.player_x,
            y: next_y + self.player_y,
            distance: distance(
                self.player_x,
                self.player_y,
                self.player_x + next_x,
                self.player_y + next_y,
            ),
            angle,
        }
    }
}

fn distance(x1: f32, y1: f32, x2: f32, y2: f32) -> f32 {
    sqrtf(powf(y2 - y1, 2.0) + powf(x2 - x1, 2.0))
}

fn in_bounds(square: i32) -> bool {
    square > 0 && square < MAP_BUFFER as i32
}

#[derive(Clone, Copy, Debug)]
pub struct Ray {
    pub x: f32,
    pub y: f32,
    pub angle: f32,
    pub distance: f32,
}
