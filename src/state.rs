use core::{
    arch::wasm32,
    f32::consts::{PI, TAU},
};

use heapless::Vec;
use libm::{ceilf, cosf, fabsf, floorf, powf, sinf, sqrtf, tanf};

use crate::{
    wasm4::trace, View, FOV, HALF_FOV, LOOK_STEP, MAP_BUFFER, MAP_SIZE, MOVE_STEP, NUMBER_OF_RAYS,
};

pub struct State {
    pub view: View,
    pub player_x: f32,
    pub player_y: f32,
    pub player_angle: f32,
    pub select_x: u8,
    pub select_y: u8,
    pub map: [bool; MAP_BUFFER],
    pub previous_gamepad: u8,
}

impl State {
    // update the player's movement
    pub fn update_character(&mut self, left: bool, right: bool, forwards: bool, backwards: bool, sprint: bool) {
        if left {
            self.player_angle += LOOK_STEP;
        }

        if right {
            self.player_angle -= LOOK_STEP;
        }

        // store this incase we might want to restore position if we hit a wall
        let previous_position = (self.player_x, self.player_y);
        
        let sprint_factor: f32 = if sprint {
            1.5
        } else {
            1.0
        };

        if forwards {
            self.player_x += cosf(self.player_angle) * MOVE_STEP * sprint_factor;
            self.player_y += -sinf(self.player_angle) * MOVE_STEP * sprint_factor;
        }

        if backwards {
            self.player_x -= cosf(self.player_angle) * MOVE_STEP * sprint_factor;
            self.player_y -= -sinf(self.player_angle) * MOVE_STEP * sprint_factor;
        }

        // move back to our original position if we moved into a wall
        if self.map[(self.player_y as i32 * MAP_SIZE + self.player_x as i32) as usize] {
            self.player_x = previous_position.0;
            self.player_y = previous_position.1;
        }

        // reset our angle if we go below 0 or above τ.
        if self.player_angle > TAU {
            self.player_angle -= TAU;
        } else if self.player_angle < 0.0 {
            self.player_angle += TAU;
        }
    }

    // get all rays
    pub fn get_rays(&self) -> Vec<Ray, NUMBER_OF_RAYS> {
        let mut rays = Vec::new();

        let angle_step = FOV / NUMBER_OF_RAYS as f32;
        let initial_angle = self.player_angle - HALF_FOV;

        for num in 0..NUMBER_OF_RAYS {
            let angle = initial_angle + num as f32 * angle_step;

            if let Err(_err) = rays.push(self.raycast(angle)) {
                trace("too many rays in raycast buffer");
                wasm32::unreachable();
            };
        }

        rays
    }

    // return a single intersection from an angle
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

        let distance = distance(
            self.player_x,
            self.player_y,
            self.player_x + next_x,
            self.player_y + next_y,
        ) * cosf(angle - self.player_angle);

        Ray {
            distance,
            vertical: false,
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

        let distance = distance(
            self.player_x,
            self.player_y,
            self.player_x + next_x,
            self.player_y + next_y,
        ) * cosf(angle - self.player_angle);

        Ray {
            distance,
            vertical: true,
        }
    }
}

fn distance(x1: f32, y1: f32, x2: f32, y2: f32) -> f32 {
    sqrtf(powf(y2 - y1, 2.0) + powf(x2 - x1, 2.0))
}

fn in_bounds(square: i32) -> bool {
    square > 0 && square < MAP_BUFFER as i32
}

pub struct Ray {
    pub distance: f32,
    pub vertical: bool,
}
