#![no_std]

use core::f32::consts::{FRAC_PI_2, PI, TAU};

use heapless::Vec;
use libm::{ceilf, cosf, floorf, fminf, sinf, sqrtf, tanf};

mod maps;

use maps::DEFAULT_MAP;

// env constants
pub const SCREEN_SIZE: u16 = 640;
pub const MAX_MAP_SIZE: usize = 32;

// gameplay constants
const MOVEMENT_STEP: f32 = 0.07;
const LOOK_STEP: f32 = 0.07;

// a simplified type that represents the in-memory map.
type WorldMap = Vec<Vec<bool, MAX_MAP_SIZE>, MAX_MAP_SIZE>;

// backing state for the entire application
pub struct State {
    map: WorldMap,
    player: Player,
}

impl Default for State {
    fn default() -> Self {
        Self {
            map: const_to_worldmap(DEFAULT_MAP),
            player: Player {
                x: 5.5,
                y: 12.5,
                dx: MOVEMENT_STEP,
                dy: 0.0,
                angle: 0.0,
            },
        }
    }
}

impl State {
    pub fn player(&self) -> &Player {
        &self.player
    }

    pub fn player_mut(&mut self) -> &mut Player {
        &mut self.player
    }

    pub fn map(&self) -> &WorldMap {
        &self.map
    }

    pub fn raycast(&self, angle: f32) -> (f32, f32) {
        let ray_angle = self.player.angle + angle;

        // This is the closest point on the grid that the ray intersects
        //
        // SEE https://lodev.org/cgtutor/images/raycastdelta.gif
        // TODO: clean this up
        let (mut fin_x, mut fin_y): (f32, f32) = {
            let side_dist_x: ((f32, f32), f32) = {
                let x = if ray_angle < FRAC_PI_2 || ray_angle > PI + FRAC_PI_2 {
                    ceilf(self.player.x) - self.player.x
                } else {
                    floorf(self.player.x) - self.player.x
                };
                let y = tanf(ray_angle) * x;

                ((x, y), sqrtf(x * x + y * y))
            };

            let side_dist_y: ((f32, f32), f32) = {
                let y = if ray_angle < PI {
                    ceilf(self.player.y) - self.player.y
                } else {
                    floorf(self.player.y) - self.player.y
                };

                let x = y / tanf(ray_angle);

                ((x, y), sqrtf(x * x + y * y))
            };

            if side_dist_x.1 == fminf(side_dist_x.1, side_dist_y.1) {
                side_dist_x.0
            } else {
                side_dist_y.0
            }
        };

        // let mut depth: usize = 0;

        // impossible extend past the max map size, so lets set it as that.
        // while depth < MAX_MAP_SIZE {
        //     if ray_angle < FRAC_PI_2 {
        //         fin_y += 1.0;
        //         fin_x += 1.0;
        //     } else if ray_angle < PI {
        //         fin_y += 1.0;
        //         fin_x -= 1.0;
        //     } else if ray_angle < PI + FRAC_PI_2 {
        //         fin_y -= 1.0;
        //         fin_x -= 1.0;
        //     } else {
        //         fin_y -= 1.0;
        //         fin_x += 1.0;
        //     }
            
        //     depth += 1;
        // }

        return (fin_x, fin_y);
    }
}

#[derive(Clone, Copy)]
pub struct Player {
    // position
    pub x: f32,
    pub y: f32,
    // next place
    pub dx: f32,
    pub dy: f32,
    // angle
    pub angle: f32,
}

impl Player {
    pub fn move_forward(&mut self) {
        self.new_loc(self.x + self.dx, self.y + self.dy);
    }

    pub fn move_backward(&mut self) {
        self.new_loc(self.x - self.dx, self.y - self.dy);
    }

    // created to reduce number of instructions generated
    fn new_loc(&mut self, new_x: f32, new_y: f32) {
        if (new_x < MAX_MAP_SIZE as f32 && new_x > 0.0)
            && (new_y < MAX_MAP_SIZE as f32 && new_y > 0.0)
        {
            self.x = new_x;
            self.y = new_y;
        }
    }

    pub fn look_left(&mut self) {
        self.angle -= LOOK_STEP;

        self.calc_new_angle();
    }

    pub fn look_right(&mut self) {
        self.angle += LOOK_STEP;

        self.calc_new_angle();
    }

    // also created to reduce number of wasm instructions
    fn calc_new_angle(&mut self) {
        if self.angle < 0.0 {
            self.angle = TAU;
        } else if self.angle > TAU {
            self.angle = 0.0;
        }

        self.dx = cosf(self.angle) * MOVEMENT_STEP;
        self.dy = sinf(self.angle) * MOVEMENT_STEP;
    }
}

fn const_to_worldmap(raw: &[&[bool]]) -> WorldMap {
    raw.iter()
        .map(|line| Vec::from_slice(line).unwrap())
        .collect()
}
