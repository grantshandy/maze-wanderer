// #![no_std]

use core::f32::consts::{FRAC_PI_2, PI, TAU};

use heapless::Vec;
use libm::{ceilf, cosf, floorf, fminf, powf, sinf, sqrtf, tanf};

mod maps;

use maps::DEFAULT_MAP;

// env constants
pub const SCREEN_SIZE: u16 = 640;
pub const MAX_MAP_SIZE: usize = 32;

// gameplay constants
const MOVEMENT_STEP: f32 = 0.07;
const LOOK_STEP: f32 = 0.07;
const FOV: usize = 60;

// a simplified type that represents the in-memory map.
type WorldMap = Vec<Vec<bool, MAX_MAP_SIZE>, MAX_MAP_SIZE>;

// backing state for the entire application
pub struct State {
    map: WorldMap,
    player: Player,
    pub projection_plane_dist: f32,
}

impl Default for State {
    fn default() -> Self {
        Self {
            map: const_to_worldmap(DEFAULT_MAP),
            player: Player {
                x: 5.5,
                y: 12.5,
                dx: -MOVEMENT_STEP,
                dy: 0.0,
                angle: PI,
            },
            projection_plane_dist: (SCREEN_SIZE / 2) as f32 / tanf(FOV as f32 / 2.0),
        }
    }
}

const INTERSECTIONS_BUFFER: usize = 300;

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

    // FOLLOW https://www.permadi.com/tutorial/raycast/rayc7.html
    pub fn raycast_from_player(
        &self,
        angle: f32,
    ) -> ((f32, f32), Vec<(f32, f32), INTERSECTIONS_BUFFER>) {
        let ray_angle = self.player.angle + angle;
        let mut intersections = Vec::new();

        // where the raycast will (hopefully) hit
        let (mut fin_x, mut fin_y) = (0.0, 0.0);

        // add closest point on the grid cast from the player
        let (closest_x, closest_y) =
            self.first_intersection(ray_angle, self.player.x, self.player.y);
        fin_x += closest_x;
        fin_y += closest_y;

        intersections.push((fin_x, fin_y)).unwrap();

        // the vectors for the next intersections with the wall (supposedly)
        let (hx, hy) = if ray_angle < PI && ray_angle > 0.0 {
            (1.0 / tanf(ray_angle), 1.0)
        } else if ray_angle > PI  && ray_angle < TAU {
            (-1.0 / tanf(ray_angle), -1.0)
        } else if ray_angle == TAU  || ray_angle == 0.0 {
            (1.0, 0.0)
        } else {
            (-1.0, 0.0)
        };
        
        fin_x += hx;
        fin_y += hy;

        // add to intersections for visual debugging
        intersections.push((fin_x, fin_y)).expect("intersections overflow");

        // let (_vx, _vy) = if (ray_angle > FRAC_PI_2 && ray_angle < PI + FRAC_PI_2) {
        //     (-1.0, -tanf(ray_angle))
        // } else {
        //     (1.0, tanf(ray_angle))
        // };

        // TODO: this sort of thing
        // if self.map[fin_y as usize][fin_x as usize] {
        //     break;
        // }

        ((fin_x, fin_y), intersections)
    }

    fn first_intersection(&self, ray_angle: f32, origin_x: f32, origin_y: f32) -> (f32, f32) {
        // closest point on the grid that intersects with the x axis
        let side_dist_x: (f32, f32) = {
            let x = if !(FRAC_PI_2..PI + FRAC_PI_2).contains(&ray_angle) {
                // facing right
                ceilf(origin_x) - origin_x
            } else {
                // facing left
                floorf(origin_x) - origin_x
            };

            let y = tanf(ray_angle) * x;

            (x, y)
        };

        // length of side dist x vector
        let side_dist_x_len: f32 = sqrtf(powf(side_dist_x.0, 2.0) + powf(side_dist_x.1, 2.0));

        // closest point on the grid that intersects the y axis
        let side_dist_y: (f32, f32) = {
            let y = if ray_angle < PI {
                // facing up
                ceilf(origin_y) - origin_y
            } else {
                // facing down
                floorf(origin_y) - origin_y
            };

            let x = y / tanf(ray_angle);

            (x, y)
        };

        // length of side dist y vector
        let side_dist_y_len: f32 = sqrtf(powf(side_dist_y.0, 2.0) + powf(side_dist_y.1, 2.0));

        // add the smallest vector to the final vector
        if side_dist_x_len == fminf(side_dist_x_len, side_dist_y_len) {
            side_dist_x
        } else {
            side_dist_y
        }
    }

    fn next_intersection(&self, ray_angle: f32, origin_x: f32, origin_y: f32) -> (f32, f32) {
        let side_dist_x: (f32, f32) = {
            let x = if !(FRAC_PI_2..PI + FRAC_PI_2).contains(&ray_angle) {
                // facing right
                ceilf(origin_x) - origin_x
            } else {
                // facing left
                floorf(origin_x) - origin_x
            };

            let y = tanf(ray_angle) * x;

            (x, y)
        };

        // length of side dist x vector
        let side_dist_x_len: f32 = sqrtf(powf(side_dist_x.0, 2.0) + powf(side_dist_x.1, 2.0));

        // closest point on the grid that intersects the y axis
        let side_dist_y: (f32, f32) = {
            let y = if ray_angle < PI {
                // facing up
                ceilf(origin_y) - origin_y
            } else {
                // facing down
                floorf(origin_y) - origin_y
            };

            let x = y / tanf(ray_angle);

            (x, y)
        };

        // length of side dist y vector
        let side_dist_y_len: f32 = sqrtf(powf(side_dist_y.0, 2.0) + powf(side_dist_y.1, 2.0));

        // add the smallest vector to the final vector
        if side_dist_x_len == fminf(side_dist_x_len, side_dist_y_len) {
            side_dist_x
        } else {
            side_dist_y
        }
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
        }

        if self.angle > TAU {
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
