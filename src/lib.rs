#![no_std]

use heapless::Vec;

mod maps;

use maps::DEFAULT_MAP;

// square screen size
pub const SCREEN_SIZE: u16 = 640;

// the maximum x / y size of the map
pub const MAX_MAP_SIZE: usize = 32;

const MOVEMENT_STEP: f32 = 0.25;

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
            map: to_worldmap(DEFAULT_MAP),
            player: Player {
                pos_x: 5.5,
                pos_y: 12.5,
                direction: 0.0,
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
    pub pos_x: f32,
    pub pos_y: f32,
    pub direction: f32,
}

impl Player {
    pub fn move_forward(&mut self) {
        self.pos_y += MOVEMENT_STEP;
    }

    pub fn move_backward(&mut self) {
        self.pos_y -= MOVEMENT_STEP;
    }
}

fn to_worldmap(raw: &[&[bool]]) -> WorldMap {
    raw.iter()
        .map(|line| Vec::from_slice(line).unwrap())
        .collect()
}
