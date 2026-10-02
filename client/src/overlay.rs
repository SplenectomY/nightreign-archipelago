//! Click-through log panel. Position and colors come from [overlay] in flags.toml.

use std::collections::VecDeque;
use std::ffi::c_void;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Mutex;
use std::thread;

type HWND = *mut c_void;
type HDC = *mut c_void;
type HGDIOBJ = *mut c_void;
type HFONT = *mut c_void;

const WS_POPUP: u32 = 0x80000000;
const WS_EX_TOPMOST: u32 = 0x00000008;
const WS_EX_TOOLWINDOW: u32 = 0x00000080;
const WS_EX_LAYERED: u32 = 0x00080000;
const WS_EX_TRANSPARENT: u32 = 0x00000020;
const WS_EX_NOACTIVATE: u32 = 0x08000000;
const LWA_ALPHA: u32 = 0x2;
const HWND_TOPMOST: isize = -1;
const SWP_SHOWWINDOW: u32 = 0x0040;
const WM_DESTROY: u32 = 0x0002;
const WM_PAINT: u32 = 0x000F;
const WM_TIMER: u32 = 0x0113;
const DT_LEFT: u32 = 0;
const DT_NOPREFIX: u32 = 0x800;
const DT_SINGLELINE: u32 = 0x20;
const CS_HREDRAW: u32 = 0x0002;
const CS_VREDRAW: u32 = 0x0001;

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

#[repr(C)]
struct Paint {
    hdc: HDC,
    erase: i32,
    rc: [i32; 4],
    restore: i32,
    inc_update: i32,
    reserved: [u8; 32],
}

#[repr(C)]
struct Rect {
    left: i32,
    top: i32,
    right: i32,
    bottom: i32,
}

#[repr(C)]
struct Msg {
    hwnd: HWND,
    message: u32,
    wparam: usize,
    lparam: isize,
    time: u32,
    pt: [i32; 2],
}

struct Cfg {
    x: i32,
    y: i32,
    width: i32,
    height: i32,
    font_size: i32,
    lines: usize,
    alpha: u8,
    text: u32,
    back: u32,
    local: u32,
    remote: u32,
    item: u32,
    location: u32,
    slot: String,
}

static LINES: Mutex<VecDeque<String>> = Mutex::new(VecDeque::new());
static HWND_SLOT: AtomicUsize = AtomicUsize::new(0);
static LINE_CAP: AtomicUsize = AtomicUsize::new(8);
static TEXT_COLOR: AtomicUsize = AtomicUsize::new(0x00A4D7E8);
static BACK_COLOR: AtomicUsize = AtomicUsize::new(0x00080E12);
static FONT_SIZE: AtomicUsize = AtomicUsize::new(16);
static COLOR_LOCAL: AtomicUsize = AtomicUsize::new(0x00FF77EE);
static COLOR_REMOTE: AtomicUsize = AtomicUsize::new(0x00FF77EE);
static COLOR_ITEM: AtomicUsize = AtomicUsize::new(0x00C8C85D);
static COLOR_LOCATION: AtomicUsize = AtomicUsize::new(0x006BE36B);
static SLOT: Mutex<String> = Mutex::new(String::new());

#[link(name = "user32")]
extern "system" {
    fn RegisterClassW(class: *const WndClass) -> u16;
    fn CreateWindowExW(ex: u32, class: *const u16, title: *const u16, style: u32, x: i32, y: i32, w: i32, h: i32, parent: HWND, menu: *mut c_void, instance: *mut c_void, param: *mut c_void) -> HWND;
    fn DefWindowProcW(hwnd: HWND, msg: u32, w: usize, l: isize) -> isize;
    fn SetLayeredWindowAttributes(hwnd: HWND, key: u32, alpha: u8, flags: u32) -> i32;
    fn SetWindowPos(hwnd: HWND, after: isize, x: i32, y: i32, w: i32, h: i32, flags: u32) -> i32;
    fn ShowWindow(hwnd: HWND, cmd: i32) -> i32;
    fn InvalidateRect(hwnd: HWND, rect: *const c_void, erase: i32) -> i32;
    fn BeginPaint(hwnd: HWND, paint: *mut Paint) -> HDC;
    fn EndPaint(hwnd: HWND, paint: *const Paint) -> i32;
    fn FillRect(hdc: HDC, rect: *const Rect, brush: *mut c_void) -> i32;
    fn SetTimer(hwnd: HWND, id: usize, ms: u32, proc: *mut c_void) -> usize;
    fn GetMessageW(msg: *mut Msg, hwnd: HWND, min: u32, max: u32) -> i32;
    fn TranslateMessage(msg: *const Msg) -> i32;
    fn DispatchMessageW(msg: *const Msg) -> isize;
    fn GetClientRect(hwnd: HWND, rect: *mut Rect) -> i32;
    fn PostQuitMessage(code: i32);
}

#[link(name = "gdi32")]
extern "system" {
    fn CreateSolidBrush(color: u32) -> *mut c_void;
    fn CreateFontW(height: i32, width: i32, esc: i32, orient: i32, weight: i32, italic: u32, underline: u32, strike: u32, charset: u32, out: u32, clip: u32, quality: u32, pitch: u32, face: *const u16) -> HFONT;
    fn SelectObject(hdc: HDC, obj: HGDIOBJ) -> HGDIOBJ;
    fn SetTextColor(hdc: HDC, color: u32) -> u32;
    fn SetBkMode(hdc: HDC, mode: i32) -> i32;
    fn DrawTextW(hdc: HDC, text: *const u16, len: i32, rect: *mut Rect, format: u32) -> i32;
    fn DeleteObject(obj: HGDIOBJ) -> i32;
}

fn wide(s: &str) -> Vec<u16> {
    s.encode_utf16().chain(std::iter::once(0)).collect()
}

fn color(s: &str, default: u32) -> u32 {
    let s = s.trim().trim_start_matches('#');
    u32::from_str_radix(s, 16).ok().map(|rgb| {
        let r = (rgb >> 16) & 0xff;
        let g = (rgb >> 8) & 0xff;
        let b = rgb & 0xff;
        (b << 16) | (g << 8) | r
    }).unwrap_or(default)
}

fn parse(text: &str) -> Option<Cfg> {
    let mut in_overlay = false;
    let mut enable = true;
    let mut cfg = Cfg { x: 24, y: 48, width: 720, height: 220, font_size: 16, lines: 8, alpha: 210, text: color("E8D7A4", 0x00A4D7E8), back: color("120E08", 0x00080E12), local: color("EE77FF", 0x00FF77EE), remote: color("EE77FF", 0x00FF77EE), item: color("5DC8C8", 0x00C8C85D), location: color("6BE36B", 0x006BE36B), slot: String::new() };
    let mut section = "";
    for line in text.lines() {
        let t = line.trim();
        if t.starts_with('[') {
            section = t;
            in_overlay = t == "[overlay]";
            continue;
        }
        if section == "[ap]" {
            if let Some((k, v)) = t.split_once('=') {
                if k.trim() == "slot" { cfg.slot = v.trim().trim_matches('"').to_string(); }
            }
        }
        if !in_overlay { continue; }
        if t.is_empty() || t.starts_with('#') { continue; }
        let Some((k, v)) = t.split_once('=') else { continue };
        let k = k.trim();
        let v = v.trim().trim_matches('"');
        match k {
            "enable" => enable = v == "true",
            "x" => cfg.x = v.parse().unwrap_or(cfg.x),
            "y" => cfg.y = v.parse().unwrap_or(cfg.y),
            "width" => cfg.width = v.parse().unwrap_or(cfg.width),
            "height" => cfg.height = v.parse().unwrap_or(cfg.height),
            "font_size" => cfg.font_size = v.parse().unwrap_or(cfg.font_size),
            "lines" => cfg.lines = v.parse().unwrap_or(cfg.lines),
            "alpha" => cfg.alpha = v.parse().unwrap_or(cfg.alpha as u32) as u8,
            "text_color" => cfg.text = color(v, cfg.text),
            "back_color" => cfg.back = color(v, cfg.back),
            "color_local" => cfg.local = color(v, cfg.local),
            "color_remote" => cfg.remote = color(v, cfg.remote),
            "color_item" => cfg.item = color(v, cfg.item),
            "color_location" => cfg.location = color(v, cfg.location),
            _ => {}
        }
    }
    enable.then_some(cfg)
}

pub fn start(toml: &str) {
    let Some(cfg) = parse(toml) else { return };
    LINE_CAP.store(cfg.lines.max(1), Ordering::SeqCst);
    TEXT_COLOR.store(cfg.text as usize, Ordering::SeqCst);
    BACK_COLOR.store(cfg.back as usize, Ordering::SeqCst);
    FONT_SIZE.store(cfg.font_size.max(8) as usize, Ordering::SeqCst);
    COLOR_LOCAL.store(cfg.local as usize, Ordering::SeqCst);
    COLOR_REMOTE.store(cfg.remote as usize, Ordering::SeqCst);
    COLOR_ITEM.store(cfg.item as usize, Ordering::SeqCst);
    COLOR_LOCATION.store(cfg.location as usize, Ordering::SeqCst);
    if let Ok(mut slot) = SLOT.lock() { *slot = cfg.slot; }
    thread::spawn(move || run(cfg));
}

pub fn push(line: &str) {
    if let Ok(mut q) = LINES.lock() {
        q.push_back(line.to_string());
        let cap = LINE_CAP.load(Ordering::SeqCst).max(1);
        while q.len() > cap { q.pop_front(); }
    }
    let hwnd = HWND_SLOT.load(Ordering::SeqCst) as HWND;
    if !hwnd.is_null() {
        unsafe { InvalidateRect(hwnd, std::ptr::null(), 1) };
    }
}


fn player_color(name: &str) -> u32 {
    let local = SLOT.lock().map(|s| s.clone()).unwrap_or_default();
    if !local.is_empty() && name == local {
        COLOR_LOCAL.load(Ordering::SeqCst) as u32
    } else {
        COLOR_REMOTE.load(Ordering::SeqCst) as u32
    }
}

fn colorize(line: &str) -> Vec<(String, u32)> {
    let text = TEXT_COLOR.load(Ordering::SeqCst) as u32;
    let item = COLOR_ITEM.load(Ordering::SeqCst) as u32;
    let loc = COLOR_LOCATION.load(Ordering::SeqCst) as u32;
    let body = line.split("NRAP AP | ").nth(1).unwrap_or(line);
    let prefix = line.len() - body.len();
    let mut spans = vec![(line[..prefix].to_string(), text)];
    if let Some(idx) = body.find(" found their ") {
        let (player, rest) = body.split_at(idx);
        let rest = &rest[" found their ".len()..];
        spans.push((player.to_string(), player_color(player.trim())));
        spans.push((" found their ".to_string(), text));
        if let Some(open) = rest.rfind(" (") {
            spans.push((rest[..open].to_string(), item));
            spans.push((" (".to_string(), text));
            let end = rest[open+2..].trim_end_matches(')');
            spans.push((end.to_string(), loc));
            spans.push((")".to_string(), text));
        } else {
            spans.push((rest.to_string(), item));
        }
        return spans;
    }
    if let Some(sent) = body.find(" sent ") {
        let (player, rest) = body.split_at(sent);
        let rest = &rest[" sent ".len()..];
        spans.push((player.to_string(), player_color(player.trim())));
        spans.push((" sent ".to_string(), text));
        if let Some(to) = rest.find(" to ") {
            spans.push((rest[..to].to_string(), item));
            spans.push((" to ".to_string(), text));
            let rest = &rest[to + " to ".len()..];
            if let Some(open) = rest.rfind(" (") {
                spans.push((rest[..open].to_string(), player_color(rest[..open].trim())));
                spans.push((" (".to_string(), text));
                spans.push((rest[open+2..].trim_end_matches(')').to_string(), loc));
                spans.push((")".to_string(), text));
            } else {
                spans.push((rest.to_string(), player_color(rest.trim())));
            }
        } else {
            spans.push((rest.to_string(), text));
        }
        return spans;
    }
    spans.push((body.to_string(), text));
    spans
}

fn paint_wrapped(hdc: HDC, rc: &Rect, lines: &[String]) {
    let width = (rc.right - rc.left - 16).max(40);
    let height = FONT_SIZE.load(Ordering::SeqCst) as i32 + 4;
    let mut y = rc.top + 6;
    for line in lines {
        let spans = colorize(line);
        let mut row: Vec<(String, u32)> = Vec::new();
        let mut used = 0i32;
        for (text, color) in spans {
            for word in text.split_inclusive(' ') {
                let w = wide(word);
                let mut box_rc = Rect { left: 0, top: 0, right: 0, bottom: 0 };
                DrawTextW(hdc, w.as_ptr(), (w.len() as i32) - 1, &mut box_rc, DT_LEFT | DT_NOPREFIX | DT_SINGLELINE | 0x400);
                let word_w = box_rc.right.max(8);
                if used > 0 && used + word_w > width {
                    draw_row(hdc, 8, y, &row);
                    y += height;
                    row.clear();
                    used = 0;
                    if y > rc.bottom { return; }
                }
                row.push((word.to_string(), color));
                used += word_w;
            }
        }
        if !row.is_empty() {
            draw_row(hdc, 8, y, &row);
            y += height;
            if y > rc.bottom { return; }
        }
    }
}

fn draw_row(hdc: HDC, mut x: i32, y: i32, row: &[(String, u32)]) {
    for (text, color) in row {
        SetTextColor(hdc, *color);
        let w = wide(text);
        let mut box_rc = Rect { left: x, top: y, right: x + 2000, bottom: y + 40 };
        DrawTextW(hdc, w.as_ptr(), (w.len() as i32) - 1, &mut box_rc, DT_LEFT | DT_NOPREFIX | DT_SINGLELINE);
        x = box_rc.right;
    }
}

fn joined(lines: &VecDeque<String>) -> String {
    let mut out = String::new();
    for (i, line) in lines.iter().enumerate() {
        if i > 0 {
            out.push(char::from(10));
        }
        out.push_str(line);
    }
    out
}

unsafe extern "system" fn wnd_proc(hwnd: HWND, msg: u32, w: usize, l: isize) -> isize {
    match msg {
        WM_PAINT => {
            let mut paint = std::mem::zeroed::<Paint>();
            let hdc = BeginPaint(hwnd, &mut paint);
            let mut rc = Rect { left: 0, top: 0, right: 0, bottom: 0 };
            GetClientRect(hwnd, &mut rc);
            let brush = CreateSolidBrush(BACK_COLOR.load(Ordering::SeqCst) as u32);
            FillRect(hdc, &rc, brush);
            DeleteObject(brush);
            SetBkMode(hdc, 1);
            SetTextColor(hdc, TEXT_COLOR.load(Ordering::SeqCst) as u32);
            let face = wide("Consolas");
            let font = CreateFontW(-(FONT_SIZE.load(Ordering::SeqCst) as i32), 0, 0, 0, 400, 0, 0, 0, 1, 0, 0, 5, 0, face.as_ptr());
            let old = SelectObject(hdc, font);
            let lines = LINES.lock().map(|q| q.iter().cloned().collect::<Vec<_>>()).unwrap_or_default();
            paint_wrapped(hdc, &rc, &lines);
            SelectObject(hdc, old);
            DeleteObject(font);
            EndPaint(hwnd, &paint);
            0
        }
        WM_TIMER => { InvalidateRect(hwnd, std::ptr::null(), 0); 0 }
        WM_DESTROY => { PostQuitMessage(0); 0 }
        _ => DefWindowProcW(hwnd, msg, w, l),
    }
}

fn run(cfg: Cfg) {
    unsafe {
        let class = wide("NRAPOverlay");
        let title = wide("NRAP");
        let wc = WndClass {
            style: CS_HREDRAW | CS_VREDRAW,
            wnd_proc: Some(wnd_proc),
            cls_extra: 0,
            wnd_extra: 0,
            instance: std::ptr::null_mut(),
            icon: std::ptr::null_mut(),
            cursor: std::ptr::null_mut(),
            background: std::ptr::null_mut(),
            menu_name: std::ptr::null(),
            class_name: class.as_ptr(),
        };
        RegisterClassW(&wc);
        let hwnd = CreateWindowExW(WS_EX_LAYERED | WS_EX_TRANSPARENT | WS_EX_TOPMOST | WS_EX_TOOLWINDOW | WS_EX_NOACTIVATE, class.as_ptr(), title.as_ptr(), WS_POPUP, cfg.x, cfg.y, cfg.width, cfg.height, std::ptr::null_mut(), std::ptr::null_mut(), std::ptr::null_mut(), std::ptr::null_mut());
        if hwnd.is_null() { return; }
        HWND_SLOT.store(hwnd as usize, Ordering::SeqCst);
        SetLayeredWindowAttributes(hwnd, 0, cfg.alpha, LWA_ALPHA);
        SetWindowPos(hwnd, HWND_TOPMOST, cfg.x, cfg.y, cfg.width, cfg.height, SWP_SHOWWINDOW);
        ShowWindow(hwnd, 8);
        SetTimer(hwnd, 1, 500, std::ptr::null_mut());
        let mut msg = std::mem::zeroed::<Msg>();
        while GetMessageW(&mut msg, std::ptr::null_mut(), 0, 0) > 0 {
            TranslateMessage(&msg);
            DispatchMessageW(&msg);
        }
    }
}
