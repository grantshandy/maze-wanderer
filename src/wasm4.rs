use crate::{View, STATE};

pub const SCREEN_SIZE: u32 = 160;

pub static mut PALETTE: *mut [u32; 4] = 0x04 as *mut [u32; 4];
pub const DRAW_COLORS: *mut u16 = 0x14 as *mut u16;
pub const GAMEPAD1: *const u8 = 0x16 as *const u8;

pub const BUTTON_LEFT: u8 = 16;
pub const BUTTON_RIGHT: u8 = 32;
pub const BUTTON_UP: u8 = 64;
pub const BUTTON_DOWN: u8 = 128;
pub const BUTTON_1: u8 = 1;
pub const BUTTON_2: u8 = 2;

pub const MOUSE_X: *const i16 = 0x1a as *const i16;
pub const MOUSE_Y: *const i16 = 0x1c as *const i16;
pub const MOUSE_BUTTONS: *const u8 = 0x1e as *const u8;
pub const MOUSE_LEFT: u8 = 1;
pub const MOUSE_RIGHT: u8 = 2;

pub fn set_draw_colors(colors: u16) {
    unsafe {
        *DRAW_COLORS = colors;
    }
}

pub fn set_view(view: View) {
    unsafe {
        STATE.view = view;
    }
}

pub fn vline(x: i32, y: i32, len: u32) {
    unsafe {
        extern_vline(x, y, len);
    }
}

pub fn oval(x: i32, y: i32, width: u32, height: u32) {
    unsafe {
        extern_oval(x, y, width, height);
    }
}

pub fn trace<T: AsRef<str>>(text: T) {
    let text_ref = text.as_ref();
    unsafe { extern_trace(text_ref.as_ptr(), text_ref.len()) }
}

pub fn rect(x: i32, y: i32, width: u32, height: u32) {
    unsafe { extern_rect(x, y, width, height) }
}

pub fn text(text: &str, x: i32, y: i32) {
    unsafe { extern_text(text.as_ptr(), text.len(), x, y) }
}

pub fn x_pressed() -> bool {
    unsafe { *GAMEPAD1 & BUTTON_1 != 0 }
}

pub fn z_pressed() -> bool {
    unsafe { *GAMEPAD1 & BUTTON_2 != 0 }
}

pub fn up_pressed() -> bool {
    unsafe { *GAMEPAD1 & BUTTON_UP != 0 }
}

pub fn down_pressed() -> bool {
    unsafe { *GAMEPAD1 & BUTTON_DOWN != 0 }
}

pub fn left_pressed() -> bool {
    unsafe { *GAMEPAD1 & BUTTON_LEFT != 0 }
}

pub fn right_pressed() -> bool {
    unsafe { *GAMEPAD1 & BUTTON_RIGHT != 0 }
}

pub fn left_clicked() -> bool {
    unsafe { *MOUSE_BUTTONS & MOUSE_LEFT != 0 }
}

pub fn right_clicked() -> bool {
    unsafe { *MOUSE_BUTTONS & MOUSE_RIGHT != 0 }
}

pub fn mouse_x() -> i16 {
    unsafe { *MOUSE_X }
}

pub fn mouse_y() -> i16 {
    unsafe { *MOUSE_Y }
}

// extern functions linking to the wasm runtime
extern "C" {
    #[link_name = "vline"]
    fn extern_vline(x: i32, y: i32, len: u32);
    #[link_name = "oval"]
    fn extern_oval(x: i32, y: i32, width: u32, height: u32);
    #[link_name = "traceUtf8"]
    fn extern_trace(trace: *const u8, length: usize);
    #[link_name = "rect"]
    fn extern_rect(x: i32, y: i32, width: u32, height: u32);
    #[link_name = "textUtf8"]
    fn extern_text(text: *const u8, length: usize, x: i32, y: i32);
}
