#![no_std]

use core::{arch::wasm32, f32::consts::PI, panic::PanicInfo};

mod state;
mod wasm4;

use state::*;
use wasm4::*;

// player perspective
const FOV: f32 = PI / 2.7;
const HALF_FOV: f32 = FOV / 2.0;
const NUMBER_OF_RAYS: usize = SCREEN_SIZE as usize;

// wall constants (don't ask)
const WALL_CONSTANT: f32 = 20.0;
const WALL_FACTOR: f32 = 5.0;

const EDITOR_TILE_SIZE: i32 = SCREEN_SIZE as i32 / MAP_SIZE;

// player movement
const MOVE_STEP: f32 = 0.04;
const LOOK_STEP: f32 = 0.04;

// map data
const MAP_SIZE: i32 = 15;
const MAP_BUFFER: usize = MAP_SIZE as usize * MAP_SIZE as usize;

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

// runtime state is stored in a single location to minimize unsafe usage.
static mut STATE: State = State {
    view: View::StartMenu,
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
    match &STATE.view {
        View::StartMenu => draw_start_menu(),
        View::FirstPerson => draw_first_person(),
        View::MapEditor => draw_map_editor(),
    }

    if x_pressed() {
        set_view(View::FirstPerson);
    }

    if z_pressed() {
        set_view(View::MapEditor);
    }
}

fn draw_start_menu() {
    set_draw_colors(0x32);
    rect(0, 0, SCREEN_SIZE, SCREEN_SIZE);

    set_draw_colors(0x24);
    text("Maze Wanderer", 30, 20);

    set_draw_colors(0x31);
    rect(14, 47, 134, 14);
    set_draw_colors(0x14);
    text("press x to start", 18, 50);

    set_draw_colors(0x31);
    rect(14, 67, 134, 27);
    set_draw_colors(0x14);
    text("press z for the", 22, 70);
    text("map editor", 41, 83);

    let mouse_x = mouse_x();
    let mouse_y = mouse_y();

    if left_clicked() {
        if mouse_x >= 14 && mouse_x <= 148 && mouse_y >= 47 && mouse_y <= 61 {
            set_view(View::FirstPerson);
        }

        if mouse_x >= 14 && mouse_x <= 148 && mouse_y >= 67 && mouse_y <= 97 {
            set_view(View::MapEditor);
        }
    }
}

fn draw_first_person() {
    unsafe {
        STATE.update_character(
            left_pressed(),
            right_pressed(),
            up_pressed(),
            down_pressed(),
        );
    }

    // draw the ground and sky
    set_draw_colors(0x33);
    rect(0, 0, SCREEN_SIZE, SCREEN_SIZE / 2);
    set_draw_colors(0x44);
    rect(0, (SCREEN_SIZE / 2) as i32, SCREEN_SIZE, SCREEN_SIZE / 2);

    // draw the walls
    let rays = unsafe { STATE.get_rays() };

    for (idx, ray) in (0_u8..).zip(rays.into_iter()) {
        let wall_height = (WALL_CONSTANT / ray.distance) * WALL_FACTOR;

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

fn draw_map_editor() {
    set_draw_colors(0x22);
    rect(0, 0, SCREEN_SIZE, SCREEN_SIZE);

    let map_data = unsafe { STATE.map };

    // draw cells
    for y in 0..MAP_SIZE {
        for x in 0..MAP_SIZE {
            if map_data[((y * MAP_SIZE) + x) as usize] {
                set_draw_colors(0x11);
            } else {
                set_draw_colors(0x44);
            }

            rect(
                x * EDITOR_TILE_SIZE + (EDITOR_TILE_SIZE / 2),
                y * EDITOR_TILE_SIZE + (EDITOR_TILE_SIZE / 2),
                EDITOR_TILE_SIZE as u32,
                EDITOR_TILE_SIZE as u32,
            );
        }
    }

    // draw player
    unsafe {
        set_draw_colors(0x33);
        oval(
            (STATE.player_x as i32 * EDITOR_TILE_SIZE) + ((EDITOR_TILE_SIZE / 4) * 3),
            (STATE.player_y as i32 * EDITOR_TILE_SIZE) + ((EDITOR_TILE_SIZE / 4) * 3),
            5,
            5,
        );
    }

    // edit map
    let col = (mouse_x() as i32 - (EDITOR_TILE_SIZE / 2)) / EDITOR_TILE_SIZE;
    let row = (mouse_y() as i32 - (EDITOR_TILE_SIZE / 2)) / EDITOR_TILE_SIZE;

    unsafe {
        if (row, col) != (STATE.player_y as i32, STATE.player_x as i32)
            && col != 0
            && col != MAP_SIZE - 1
            && row != 0
            && row != MAP_SIZE - 1
        {
            let square = ((row * MAP_SIZE) + col) as usize;

            if left_clicked() {
                STATE.map[square] = true;
            }

            if right_clicked() {
                STATE.map[square] = false;
            }
        }
    }
}

pub enum View {
    StartMenu,
    FirstPerson,
    MapEditor,
}

// this should be stripped in the wasm-snip process
#[panic_handler]
fn phandler(_: &PanicInfo<'_>) -> ! {
    trace("panic!");
    wasm32::unreachable()
}
