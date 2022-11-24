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
    player_x: 1.5,
    player_y: 1.5,
    player_angle: 0.0,
    map: MAP,
    select_x: 4,
    select_y: 4,
    previous_gamepad: 0,
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

    // toggle game view
    let just_pressed = *GAMEPAD1 & (*GAMEPAD1 ^ STATE.previous_gamepad);

    if just_pressed & BUTTON_2 != 0 {
        match &STATE.view {
            View::FirstPerson => set_view(View::MapEditor),
            View::MapEditor => set_view(View::FirstPerson),
            _ => (),
        }
    }

    STATE.previous_gamepad = *GAMEPAD1;
}

unsafe fn draw_start_menu() {
    set_draw_colors(0x32);
    rect(0, 0, SCREEN_SIZE, SCREEN_SIZE);

    // draw map preview
    for y in 0..MAP_SIZE {
        for x in 0..MAP_SIZE {
            if MAP[((y * MAP_SIZE) + x) as usize] {
                set_draw_colors(0x11);
            } else {
                set_draw_colors(0x44);
            }

            rect((x * 3) + 57, (y * 3) + 105, 3, 3);
        }
    }

    set_draw_colors(0x24);
    text("Maze Wanderer", 31, 20);

    set_draw_colors(0x04);
    text("press x to start", 18, 50);

    if *GAMEPAD1 & BUTTON_1 != 0 {
        set_view(View::FirstPerson);
    }

    set_draw_colors(0x04);
    text("press z to toggle", 13, 74);
    text("view", 64, 87);
}

fn draw_first_person() {
    unsafe {
        STATE.update_character(
            *GAMEPAD1 & BUTTON_LEFT != 0,
            *GAMEPAD1 & BUTTON_RIGHT != 0,
            *GAMEPAD1 & BUTTON_UP != 0,
            *GAMEPAD1 & BUTTON_DOWN != 0,
            *GAMEPAD1 & BUTTON_1 != 0,
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

unsafe fn draw_map_editor() {
    set_draw_colors(0x22);
    rect(0, 0, SCREEN_SIZE, SCREEN_SIZE);

    // draw cells
    for y in 0..MAP_SIZE {
        for x in 0..MAP_SIZE {
            if STATE.map[((y * MAP_SIZE) + x) as usize] {
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

    // player coords on the integer grid
    let player_x = STATE.player_x as i32;
    let player_y = STATE.player_y as i32;

    // draw player
    set_draw_colors(0x33);
    oval(
        (player_x * EDITOR_TILE_SIZE) + ((EDITOR_TILE_SIZE / 4) * 3),
        (player_y * EDITOR_TILE_SIZE) + ((EDITOR_TILE_SIZE / 4) * 3),
        5,
        5,
    );

    // draw select border
    let select_x = STATE.select_x as i32;
    let select_y = STATE.select_y as i32;

    let just_pressed = *GAMEPAD1 & (*GAMEPAD1 ^ STATE.previous_gamepad);

    if just_pressed & BUTTON_UP != 0 && STATE.select_y - 1 != 0 {
        STATE.select_y -= 1;
    }

    if just_pressed & BUTTON_DOWN != 0 && STATE.select_y + 1 != MAP_SIZE as u8 {
        STATE.select_y += 1;
    }

    if just_pressed & BUTTON_RIGHT != 0 && STATE.select_x + 1 != MAP_SIZE as u8 {
        STATE.select_x += 1;
    }

    if just_pressed & BUTTON_LEFT != 0 && STATE.select_x - 1 != 0 {
        STATE.select_x -= 1;
    }

    set_draw_colors(0x30);
    rect(
        select_x * EDITOR_TILE_SIZE + (EDITOR_TILE_SIZE / 2),
        select_y * EDITOR_TILE_SIZE + (EDITOR_TILE_SIZE / 2),
        EDITOR_TILE_SIZE as u32,
        EDITOR_TILE_SIZE as u32,
    );

    // edit map
    if (select_x, select_y) != (player_x, player_y) && just_pressed & BUTTON_1 != 0 {
        let square = ((select_y * MAP_SIZE) + select_x) as usize;

        if STATE.map[square] {
            STATE.map[square] = false;
        } else {
            STATE.map[square] = true;
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
