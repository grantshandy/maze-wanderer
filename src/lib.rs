#![no_std]

use heapless::{String, Vec};

// the max x / y size of the map
const MAX_MAP_SIZE: usize = 30;

// the max size of the textual representation
// 1 is added to each line for the newline (\n) character
const MAX_MAP_BUFFER_SIZE: usize = MAX_MAP_SIZE * (MAX_MAP_SIZE + 1);

// a simplified type that represents the in-memory
// map.
type Map = Vec<Vec<u8, MAX_MAP_SIZE>, MAX_MAP_SIZE>;

pub const SCREEN_WIDTH: u16 = 640;
pub const SCREEN_HEIGHT: u16 = 480;

const DEFAULT_MAP: &str = include_str!("one.gmap");

pub struct State {
    world_map: Map,
    player: Player,
}

impl Default for State {
    fn default() -> Self {
        Self {
            player: Player::default(),
            world_map: generate_map(DEFAULT_MAP),
        }
    }
}

impl State {
    pub fn world_map(&self) -> &Vec<Vec<u8, MAX_MAP_SIZE>, MAX_MAP_SIZE> {
        &self.world_map
    }
}

struct Player {
    x: u32,
    y: u32,
}

impl Default for Player {
    fn default() -> Self {
        Player { x: 22, y: 12 }
    }
}

pub fn generate_map(input: &str) -> Vec<Vec<u8, MAX_MAP_SIZE>, MAX_MAP_SIZE> {
    // overall buffer from the input string
    let input: String<MAX_MAP_BUFFER_SIZE> = String::from(input);

    // soy face ascii art
    input
        .lines()
        .map(|line: &str| {
            let line_text: String<MAX_MAP_SIZE> = String::from(line);
            let mut line_nums: Vec<u8, MAX_MAP_SIZE> = Vec::new();

            for c in line_text.chars() {
                line_nums.push(match c {
                    '0' => 0,
                    '1' => 1,
                    '2' => 2,
                    '3' => 3,
                    '4' => 4,
                    '5' => 5,
                    '6' => 6,
                    '7' => 7,
                    '8' => 8,
                    '9' => 9,
                    _ => 0,
                }).expect("line too large");
            }

            line_nums
        })
        .collect()
}
