#![no_std]

use core::f32::consts::PI;

use heapless::Vec;
use libm::{cosf, sinf};

mod maps;

use maps::DEFAULT_MAP;

// env constants
pub const SCREEN_SIZE: u16 = 640;
pub const MAX_MAP_SIZE: usize = 32;
const TWO_PI: f32 = PI * 2.0;

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
    pub fn run(&mut self) {}

    pub fn player(&self) -> &Player {
        &self.player
    }

    pub fn player_mut(&mut self) -> &mut Player {
        &mut self.player
    }

    pub fn map(&self) -> &WorldMap {
        &self.map
    }

    pub fn is_in_wall(&self, x: f32, y: f32) -> bool {
        *self
            .map
            .get(y.round() as usize)
            .map(|line| line.get(x.round() as usize).unwrap_or(&false))
            .unwrap_or(&false)
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
            self.angle = TWO_PI;
        } else if self.angle > TWO_PI {
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
