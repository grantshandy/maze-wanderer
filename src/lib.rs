#![no_std]

use core::{
    arch::wasm32,
    f32::consts::{PI, TAU},
    panic::PanicInfo,
};

use heapless::Vec;
use libm::{ceilf, cosf, fabsf, floorf, powf, sinf, sqrtf, tanf};

// External WASM-4 Constants
const SCREEN_SIZE: u32 = 160;

static mut PALETTE: *mut [u32; 4] = 0x04 as *mut [u32; 4];
const DRAW_COLORS: *mut u16 = 0x14 as *mut u16;
const GAMEPAD1: *const u8 = 0x16 as *const u8;

const BUTTON_LEFT: u8 = 16;
const BUTTON_RIGHT: u8 = 32;
const BUTTON_UP: u8 = 64;
const BUTTON_DOWN: u8 = 128;
const BUTTON_1: u8 = 1;
const BUTTON_2: u8 = 2;

// WASM-4 helper and wrapper functions
fn set_draw_colors(colors: u16) {
    unsafe {
        *DRAW_COLORS = colors;
    }
}

fn vline(x: i32, y: i32, len: u32) {
    unsafe {
        extern_vline(x, y, len);
    }
}

fn oval(x: i32, y: i32, width: u32, height: u32) {
    unsafe { extern_oval(x, y, width, height) }
}

fn rect(x: i32, y: i32, width: u32, height: u32) {
    unsafe { extern_rect(x, y, width, height) }
}

fn text(text: &str, x: i32, y: i32) {
    unsafe { extern_text(text.as_ptr(), text.len(), x, y) }
}

// extern functions linking to the wasm runtime
extern "C" {
    #[link_name = "vline"]
    fn extern_vline(x: i32, y: i32, len: u32);
    #[link_name = "oval"]
    fn extern_oval(x: i32, y: i32, width: u32, height: u32);
    #[link_name = "rect"]
    fn extern_rect(x: i32, y: i32, width: u32, height: u32);
    #[link_name = "textUtf8"]
    fn extern_text(text: *const u8, length: usize, x: i32, y: i32);
}

// player perspective
const FOV: f32 = PI / 2.7;
const HALF_FOV: f32 = FOV / 2.0;
const NUMBER_OF_RAYS: usize = SCREEN_SIZE as usize;

// wall constants (don't ask)
const WALL_CONSTANT: f32 = 20.0;
const WALL_FACTOR: f32 = 5.0;

const EDITOR_TILE_SIZE: i32 = SCREEN_SIZE as i32 / MAP_SIZE;

// player movement
const MOVE_STEP: f32 = 0.035;
const LOOK_STEP: f32 = 0.035;

// map data
const MAP_SIZE: i32 = 15;
const MAP_BUFFER: usize = MAP_SIZE as usize * MAP_SIZE as usize;

#[rustfmt::skip]
const MAP: [bool; MAP_BUFFER] = [
    true, true,  true,  true,  true,  true,  true,  true,  true,  true,  true,  true,  true,  true,  true,
    true, false, false, false, false, false, false, false, false, false, false, false, false, false, true,
    true, false, true,  false, true,  false, true,  true,  true,  false, true,  false, true,  false, true,
    true, false, false, false, true,  false, false, false, false, false, true,  false, false, false, true,
    true, false, true,  true,  true,  false, true,  false, true,  false, true,  true,  true,  false, true,
    true, false, false, false, false, false, false, false, false, false, false, false, false, false, true,
    true, false, true,  false, true,  false, true,  true,  true,  false, true,  false, true,  false, true,
    true, false, true,  false, false, false, true,  false, true,  false, false, false, true,  false, true,
    true, false, true,  false, true,  false, true,  true,  true,  false, true,  false, true,  false, true,
    true, false, false, false, false, false, false, false, false, false, false, false, false, false, true,
    true, false, true,  true,  true,  false, true,  false, true,  false, true,  true,  true,  false, true,
    true, false, false, false, true,  false, false, false, false, false, true,  false, false, false, true,
    true, false, true,  false, true,  false, true,  true,  true,  false, true,  false, true,  false, true,
    true, false, false, false, false, false, false, false, false, false, false, false, false, false, true,
    true, true,  true,  true,  true,  true,  true,  true,  true,  true,  true,  true,  true,  true,  true,
];

// runtime state is stored in a single location to minimize unsafe usage.
static mut STATE: State = State {
    view: View::StartMenu,
    player_x: 1.6,
    player_y: 1.5,
    player_angle: 0.0,
    map: MAP,
    select_x: 4,
    select_y: 4,
    previous_gamepad: 0,
};

// runs on startup
#[no_mangle]
fn start() {
    unsafe {
        *PALETTE = [0x2B2D24, 0x606751, 0x949C81, 0x3E74BC];
    }
}

// runs every frame
#[no_mangle]
fn update() {
    unsafe {
        STATE.update();
    }
}

struct Ray {
    angle_diff: f32,
    pub distance: f32,
    pub vertical: bool,
}

impl Ray {
    pub fn wall_height(&self) -> f32 {
        (WALL_CONSTANT / (self.distance * cosf(self.angle_diff))) * WALL_FACTOR
    }
}

enum View {
    StartMenu,
    FirstPerson,
    MapEditor,
}

struct State {
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
    pub fn update(&mut self) {
        match self.view {
            View::StartMenu => {
                set_draw_colors(0x11);
                rect(0, 0, SCREEN_SIZE, SCREEN_SIZE);

                set_draw_colors(0x44);
                rect(56, 104, 47, 47);

                // draw map preview
                for y in 0..MAP_SIZE {
                    for x in 0..MAP_SIZE {
                        if MAP[((y * MAP_SIZE) + x) as usize] {
                            set_draw_colors(0x22);
                        } else {
                            set_draw_colors(0x33);
                        }

                        rect((x * 3) + 57, (y * 3) + 105, 3, 3);
                    }
                }

                set_draw_colors(0x44);
                rect(78, 126, 3, 3);

                set_draw_colors(0x44);
                rect(5, 5, 150, 83);

                set_draw_colors(0x23);
                rect(6, 6, 148, 81);
                rect(7, 7, 146, 79);

                set_draw_colors(0x31);
                text("Maze Wanderer", 31, 13);

                set_draw_colors(0x31);
                text("press x to start", 18, 37);

                unsafe {
                    if *GAMEPAD1 & BUTTON_1 != 0 {
                        self.view = View::FirstPerson;
                    }
                }

                set_draw_colors(0x31);
                text("press z to toggle", 13, 58);
                text("view", 64, 71);
            }
            View::MapEditor => {
                set_draw_colors(0x11);
                rect(0, 0, SCREEN_SIZE, SCREEN_SIZE);

                // draw cells
                for y in 0..MAP_SIZE {
                    for x in 0..MAP_SIZE {
                        if self.map[((y * MAP_SIZE) + x) as usize] {
                            set_draw_colors(0x22);
                        } else {
                            set_draw_colors(0x33);
                        }

                        rect(
                            x * EDITOR_TILE_SIZE + (EDITOR_TILE_SIZE / 2),
                            y * EDITOR_TILE_SIZE + (EDITOR_TILE_SIZE / 2),
                            EDITOR_TILE_SIZE as u32,
                            EDITOR_TILE_SIZE as u32,
                        );
                    }
                }

                // player coords on the integer grid
                let player_x = self.player_x as i32;
                let player_y = self.player_y as i32;

                // draw player
                set_draw_colors(0x44);
                oval(
                    (player_x * EDITOR_TILE_SIZE) + ((EDITOR_TILE_SIZE / 4) * 3),
                    (player_y * EDITOR_TILE_SIZE) + ((EDITOR_TILE_SIZE / 4) * 3),
                    5,
                    5,
                );

                // draw select border
                let just_pressed = unsafe { *GAMEPAD1 & (*GAMEPAD1 ^ self.previous_gamepad) };

                if just_pressed & BUTTON_UP != 0 && self.select_y - 1 != 0 {
                    self.select_y -= 1;
                }

                if just_pressed & BUTTON_DOWN != 0 && self.select_y + 1 != (MAP_SIZE - 1) as u8 {
                    self.select_y += 1;
                }

                if just_pressed & BUTTON_LEFT != 0 && self.select_x - 1 != 0 {
                    self.select_x -= 1;
                }
                
                if just_pressed & BUTTON_RIGHT != 0 && self.select_x + 1 != (MAP_SIZE - 1) as u8 {
                    self.select_x += 1;
                }

                let select_x = self.select_x as i32;
                let select_y = self.select_y as i32;

                set_draw_colors(0x40);
                rect(
                    select_x * EDITOR_TILE_SIZE + (EDITOR_TILE_SIZE / 2),
                    select_y * EDITOR_TILE_SIZE + (EDITOR_TILE_SIZE / 2),
                    EDITOR_TILE_SIZE as u32,
                    EDITOR_TILE_SIZE as u32,
                );

                // edit map
                if (select_x, select_y) != (player_x, player_y) && just_pressed & BUTTON_1 != 0 {
                    let square = ((select_y * MAP_SIZE) + select_x) as usize;

                    if self.map[square] {
                        self.map[square] = false;
                    } else {
                        self.map[square] = true;
                    }
                }
            }
            View::FirstPerson => {
                self.update_character();

                // draw the ground and sky
                set_draw_colors(0x44);
                rect(0, 0, SCREEN_SIZE, SCREEN_SIZE / 2);
                set_draw_colors(0x33);
                rect(0, (SCREEN_SIZE / 2) as i32, SCREEN_SIZE, SCREEN_SIZE / 2);

                // draw the walls
                let rays = self.get_rays();

                for (idx, ray) in (0_u8..).zip(rays.into_iter()) {
                    let wall_height = ray.wall_height();

                    if ray.vertical {
                        set_draw_colors(0x22);
                    } else {
                        set_draw_colors(0x11);
                    }

                    vline(
                        SCREEN_SIZE as i32 - idx as i32 - 1,
                        (SCREEN_SIZE / 2) as i32 - (wall_height / 2.0) as i32,
                        wall_height as u32,
                    );
                }
            }
        }

        // toggle game view
        unsafe {
            if (*GAMEPAD1 & (*GAMEPAD1 ^ self.previous_gamepad)) & BUTTON_2 != 0 {
                self.view = match &self.view {
                    View::FirstPerson => View::MapEditor,
                    View::MapEditor => View::FirstPerson,
                    View::StartMenu => View::StartMenu,
                }
            }

            self.previous_gamepad = *GAMEPAD1;
        }
    }

    // update the player's movement
    pub fn update_character(&mut self) {
        let (left, right, forwards, backwards, sprint) = unsafe {
            (
                *GAMEPAD1 & BUTTON_LEFT != 0,
                *GAMEPAD1 & BUTTON_RIGHT != 0,
                *GAMEPAD1 & BUTTON_UP != 0,
                *GAMEPAD1 & BUTTON_DOWN != 0,
                *GAMEPAD1 & BUTTON_1 != 0,
            )
        };

        if left {
            self.player_angle += LOOK_STEP;
        }

        if right {
            self.player_angle -= LOOK_STEP;
        }

        // store this incase we might want to restore position if we hit a wall
        let previous_position = (self.player_x, self.player_y);

        let sprint_factor: f32 = if sprint { 1.5 } else { 1.0 };

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

    // get all for the screen rays
    pub fn get_rays(&self) -> Vec<Ray, NUMBER_OF_RAYS> {
        let mut rays = Vec::new();

        let angle_step = FOV / NUMBER_OF_RAYS as f32;
        let initial_angle = self.player_angle - HALF_FOV;

        for num in 0..NUMBER_OF_RAYS {
            let angle = initial_angle + num as f32 * angle_step;

            if let Err(_err) = rays.push(self.raycast(angle)) {
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
        );

        Ray {
            angle_diff: angle - self.player_angle,
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
        );

        Ray {
            angle_diff: angle - self.player_angle,
            distance,
            vertical: true,
        }
    }
}

// returns
fn distance(x1: f32, y1: f32, x2: f32, y2: f32) -> f32 {
    sqrtf(powf(y2 - y1, 2.0) + powf(x2 - x1, 2.0))
}

// returns true if the map index is in the map
fn in_bounds(square: i32) -> bool {
    square > 0 && square < MAP_BUFFER as i32
}

// this should be stripped in the wasm-snip process
#[panic_handler]
fn phandler(_: &PanicInfo<'_>) -> ! {
    wasm32::unreachable()
}
