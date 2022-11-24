#![no_std]

use core::{
    arch::wasm32,
    f32::consts::{PI, TAU},
    panic::PanicInfo,
};

use heapless::Vec;
use libm::{ceilf, cosf, fabsf, floorf, powf, sinf, sqrtf, tanf};

// player perspective
const FOV: f32 = PI / 2.9;
const HALF_FOV: f32 = FOV / 2.0;
const NUMBER_OF_RAYS: usize = SCREEN_SIZE as usize;

// map data
const MAP_SIZE: i32 = 15;
const MAP_BUFFER: usize = MAP_SIZE as usize * MAP_SIZE as usize;

// player movement
const MOVE_STEP: f32 = 0.05;
const LOOK_STEP: f32 = 0.05;

#[rustfmt::skip]
const DEFAULT_MAP: [bool; MAP_BUFFER] = [
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

// ---- wasm4 game engine pointers and constants ----
const SCREEN_SIZE: u32 = 160;

static mut PALETTE: *mut [u32; 4] = 0x04 as *mut [u32; 4];
const DRAW_COLORS: *mut u16 = 0x14 as *mut u16;
const GAMEPAD1: *const u8 = 0x16 as *const u8;

const BUTTON_LEFT: u8 = 16;
const BUTTON_RIGHT: u8 = 32;
const BUTTON_UP: u8 = 64;
const BUTTON_DOWN: u8 = 128;

const BUTTON_1: u8 = 1;

// runtime state is stored in a single location to minimize unsafe usage.
static mut STATE: State = State {
    menu: true,
    player_x: 1.5,
    player_y: 1.5,
    player_angle: 0.0,
    map: DEFAULT_MAP,
};

// runs on startup
#[no_mangle]
unsafe fn start() {
    *PALETTE = [0x36392D, 0x4B503F, 0x4E74BC, 0xDEF2C8];
}

// runs every frame
#[no_mangle]
unsafe fn update() {
    if STATE.menu {
        *DRAW_COLORS = 0x32;
        rect(0, 0, SCREEN_SIZE, SCREEN_SIZE);

        *DRAW_COLORS = 0x24;
        text("Walking Simulator!", 10, 20);

        *DRAW_COLORS = 0x34;
        text("Press X To Start", 16, 90);

        if *GAMEPAD1 & BUTTON_1 != 0 {
            STATE.menu = false;
        }
    } else {
        // move the character from the gamepad
        STATE.update_character(
            *GAMEPAD1 & BUTTON_RIGHT != 0,
            *GAMEPAD1 & BUTTON_LEFT != 0,
            *GAMEPAD1 & BUTTON_UP != 0,
            *GAMEPAD1 & BUTTON_DOWN != 0,
        );

        // draw the ground and sky
        *DRAW_COLORS = 0x33;
        rect(0, 0, SCREEN_SIZE, SCREEN_SIZE / 2);
        *DRAW_COLORS = 0x44;
        rect(0, (SCREEN_SIZE / 2) as i32, SCREEN_SIZE, SCREEN_SIZE / 2);

        // draw the walls
        for (idx, ray) in (0_u8..).zip(STATE.get_rays().into_iter()) {
            let wall_height = (10.0 / ray.perp_distance) * 15.0;

            if ray.vertical {
                *DRAW_COLORS = 0x22;
            } else {
                *DRAW_COLORS = 0x11;
            }

            vline(
                idx as i32,
                (SCREEN_SIZE / 2) as i32 - (wall_height / 2.0) as i32,
                wall_height as u32,
            );
        }
    }
}

struct State {
    pub menu: bool,
    pub player_x: f32,
    pub player_y: f32,
    pub player_angle: f32,
    pub map: [bool; MAP_BUFFER],
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
                trace("too many rays for raycast buffer");
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

        let perp_distance = distance * cosf(angle - self.player_angle);

        Ray {
            distance,
            perp_distance,
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

        let perp_distance = distance * cosf(angle - self.player_angle);

        Ray {
            distance,
            perp_distance,
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

struct Ray {
    pub distance: f32,
    pub perp_distance: f32,
    pub vertical: bool,
}

// draw a vertical line (used for lines)
fn vline(x: i32, y: i32, len: u32) {
    unsafe {
        extern_vline(x, y, len);
    }
}

// write to the console (for errors)
fn trace<T: AsRef<str>>(text: T) {
    let text_ref = text.as_ref();
    unsafe { extern_trace(text_ref.as_ptr(), text_ref.len()) }
}

// create a rectangle (for background)
fn rect(x: i32, y: i32, width: u32, height: u32) {
    unsafe { extern_rect(x, y, width, height) }
}

// draw text on the screen
fn text(text: &str, x: i32, y: i32) {
    unsafe { extern_text(text.as_ptr(), text.len(), x, y) }
}

// extern functions linking to the wasm runtime
extern "C" {
    #[link_name = "vline"]
    fn extern_vline(x: i32, y: i32, len: u32);
    #[link_name = "traceUtf8"]
    fn extern_trace(trace: *const u8, length: usize);
    #[link_name = "rect"]
    fn extern_rect(x: i32, y: i32, width: u32, height: u32);
    #[link_name = "textUtf8"]
    fn extern_text(text: *const u8, length: usize, x: i32, y: i32);
}

// this should be stripped in the wasm-snip process
#[panic_handler]
fn phandler(_: &PanicInfo<'_>) -> ! {
    wasm32::unreachable()
}
