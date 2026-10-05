//! Testing tools opened from Options. Writes commands.txt next to nrap.exe.

#![cfg(windows)]

use std::fs;
use std::path::PathBuf;

type HWND = *mut c_void;
use std::ffi::c_void;

const WS_CHILD: u32 = 0x40000000;
const WS_VISIBLE: u32 = 0x10000000;
const WS_OVERLAPPEDWINDOW: u32 = 0x00CF0000;
const WM_COMMAND: u32 = 0x0111;
const WM_DESTROY: u32 = 0x0002;

#[link(name = "user32")]
extern "system" {
    fn RegisterClassW(class: *const WndClass) -> u16;
    fn CreateWindowExW(ex: u32, class: *const u16, title: *const u16, style: u32, x: i32, y: i32, w: i32, h: i32, parent: HWND, menu: isize, inst: *mut c_void, param: *mut c_void) -> HWND;
    fn DestroyWindow(hwnd: HWND) -> i32;
    fn DefWindowProcW(hwnd: HWND, msg: u32, w: usize, l: isize) -> isize;
}

#[repr(C)]
struct WndClass {
    style: u32,
    wnd_proc: Option<unsafe extern "system" fn(HWND, u32, usize, isize) -> isize>,
    cls_extra: i32,
    wnd_extra: i32,
    instance: *mut c_void,
    icon: *mut c_void,
    cursor: *mut c_void,
    background: *mut c_void,
    menu_name: *const u16,
    class_name: *const u16,
}

fn wide(s: &str) -> Vec<u16> {
    s.encode_utf16().chain(std::iter::once(0)).collect()
}

static mut WIN: HWND = std::ptr::null_mut();
static mut DIR: Option<PathBuf> = None;

fn queue(line: &str) {
    let Some(dir) = (unsafe { DIR.as_ref() }) else { return };
    let path = dir.join("commands.txt");
    let mut existing = fs::read_to_string(&path).unwrap_or_default();
    if !existing.is_empty() && !existing.ends_with('\n') { existing.push('\n'); }
    existing.push_str(line);
    existing.push('\n');
    let _ = fs::write(path, existing);
}

unsafe extern "system" fn proc(hwnd: HWND, msg: u32, w: usize, l: isize) -> isize {
    match msg {
        WM_COMMAND => {
            match (w & 0xffff) as isize {
                1 => queue("/testdeath send"),
                2 => queue("/testdeath recv"),
                3 => queue("/testdeath runes"),
                4 => queue("/testdeath half"),
                5 => queue("/testdeath double"),
                _ => {}
            }
            0
        }
        WM_DESTROY => {
            WIN = std::ptr::null_mut();
            0
        }
        _ => DefWindowProcW(hwnd, msg, w, l),
    }
}

pub fn open(dir: PathBuf) {
    unsafe {
        if !WIN.is_null() { return; }
        DIR = Some(dir);
        let class = wide("NRAPTesting");
        let wc = WndClass { style: 0, wnd_proc: Some(proc), cls_extra: 0, wnd_extra: 0, instance: std::ptr::null_mut(), icon: std::ptr::null_mut(), cursor: std::ptr::null_mut(), background: 16 as *mut c_void, menu_name: std::ptr::null(), class_name: class.as_ptr() };
        RegisterClassW(&wc);
        let win = CreateWindowExW(0, class.as_ptr(), wide("NRAP Testing").as_ptr(), WS_OVERLAPPEDWINDOW | WS_VISIBLE, 220, 140, 360, 360, std::ptr::null_mut(), 0, std::ptr::null_mut(), std::ptr::null_mut());
        let button = wide("BUTTON");
        CreateWindowExW(0, button.as_ptr(), wide("Test Death Link Send").as_ptr(), WS_CHILD | WS_VISIBLE, 40, 24, 260, 36, win, 1, std::ptr::null_mut(), std::ptr::null_mut());
        CreateWindowExW(0, button.as_ptr(), wide("Test Death Link Received").as_ptr(), WS_CHILD | WS_VISIBLE, 40, 76, 260, 36, win, 2, std::ptr::null_mut(), std::ptr::null_mut());
        CreateWindowExW(0, button.as_ptr(), wide("Add 1000 runes").as_ptr(), WS_CHILD | WS_VISIBLE, 40, 128, 260, 36, win, 3, std::ptr::null_mut(), std::ptr::null_mut());
        CreateWindowExW(0, button.as_ptr(), wide("Model size 1/2").as_ptr(), WS_CHILD | WS_VISIBLE, 40, 180, 260, 36, win, 4, std::ptr::null_mut(), std::ptr::null_mut());
        CreateWindowExW(0, button.as_ptr(), wide("Model size x2").as_ptr(), WS_CHILD | WS_VISIBLE, 40, 232, 260, 36, win, 5, std::ptr::null_mut(), std::ptr::null_mut());
        WIN = win;
    }
}
