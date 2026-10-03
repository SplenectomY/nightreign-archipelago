#![cfg(windows)]
//! External NRAP window. Tails nrap.log, writes commands.txt, edits flags.toml, launches me3.

use std::ffi::c_void;
use std::fs;
use std::path::PathBuf;
use std::process::Command;

type HWND = *mut c_void;
type HINSTANCE = *mut c_void;

const WS_OVERLAPPEDWINDOW: u32 = 0x00CF0000;
const WS_VISIBLE: u32 = 0x10000000;
const WS_CHILD: u32 = 0x40000000;
const WS_VSCROLL: u32 = 0x00200000;
const ES_MULTILINE: u32 = 0x0004;
const ES_AUTOVSCROLL: u32 = 0x0040;
const ES_READONLY: u32 = 0x0800;
const WS_DISABLED: u32 = 0x08000000;
const ES_PASSWORD: u32 = 0x0020;
const BS_AUTOCHECKBOX: u32 = 0x0003;
const BS_OWNERDRAW: u32 = 0x000B;
const WM_DRAWITEM: u32 = 0x002B;
const WM_COMMAND: u32 = 0x0111;
const WM_CTLCOLOREDIT: u32 = 0x0133;
const WM_CTLCOLORSTATIC: u32 = 0x0138;
const WS_EX_CLIENTEDGE: u32 = 0x00000200;
const EM_SCROLLCARET: u32 = 0x00B7;
const WM_TIMER: u32 = 0x0113;
const WM_DESTROY: u32 = 0x0002;
const BM_GETCHECK: u32 = 0x00F0;
const BM_SETCHECK: u32 = 0x00F1;
const EM_SETSEL: u32 = 0x00B1;
const EM_REPLACESEL: u32 = 0x00C2;
const EM_SETCHARFORMAT: u32 = 0x0444;
const EM_SETBKGNDCOLOR: u32 = 0x0443;
const EM_SETCUEBANNER: u32 = 0x1501;
const SS_ETCHEDHORZ: u32 = 0x10;
const SCF_SELECTION: usize = 1;
const CFM_COLOR: u32 = 0x40000000;

const LOG: isize = 100;
const CMD: isize = 101;
const SEND: isize = 102;
const HOST: isize = 103;
const SLOT: isize = 104;
const PASS: isize = 105;
const LAUNCH: isize = 107;
const OPTIONS: isize = 113;
const RECONNECT: isize = 114;
const BROWSE_NRSC: isize = 111;
const SEAMLESS: isize = 112;
const BROWSE_ME3: isize = 115;
const ME3: isize = 116;
const EN_KILLFOCUS: u16 = 0x0200;
const SS_ICON: u32 = 0x0003;
const ES_AUTOHSCROLL: u32 = 0x0080;
const SW_HIDE: i32 = 0;
const SW_SHOW: i32 = 5;

static mut APP: *mut App = std::ptr::null_mut();
static mut FIELD_BRUSH: isize = 0;
static mut LABEL_BRUSH: isize = 0;
static mut WARN_BRUSH: isize = 0;
static mut OPT: *mut OptWin = std::ptr::null_mut();
static mut CUST: [u32; 16] = [0; 16];
static mut SCROLLED: bool = false;

#[repr(C)]
struct SystemTime { year: u16, month: u16, dow: u16, day: u16, hour: u16, minute: u16, second: u16, ms: u16 }

struct App {
    log: HWND,
    cmd: HWND,
    send: HWND,
    host: HWND,
    slot: HWND,
    pass: HWND,
    nrsc: HWND,
    me3: HWND,
    reconnect: HWND,
    dir: PathBuf,
    log_off: u64,
    mod_off: u64,
    connected: bool,
}

struct OptWin {
    x: HWND,
    y: HWND,
    width: HWND,
    height: HWND,
    fade: HWND,
    font: HWND,
    local: HWND,
    remote: HWND,
    item: HWND,
    location: HWND,
    text: HWND,
    host_text: HWND,
    host_local: HWND,
    host_remote: HWND,
    host_item: HWND,
    host_location: HWND,
    debug: HWND,
}

#[link(name = "user32")]
extern "system" {
    fn RegisterClassW(class: *const WndClass) -> u16;
    fn CreateWindowExW(ex: u32, class: *const u16, title: *const u16, style: u32, x: i32, y: i32, w: i32, h: i32, parent: HWND, menu: isize, instance: HINSTANCE, param: *mut c_void) -> HWND;
    fn DefWindowProcW(hwnd: HWND, msg: u32, w: usize, l: isize) -> isize;
    fn GetMessageW(msg: *mut [usize; 7], hwnd: HWND, min: u32, max: u32) -> i32;
    fn TranslateMessage(msg: *const [usize; 7]) -> i32;
    fn DispatchMessageW(msg: *const [usize; 7]) -> isize;
    fn PostQuitMessage(code: i32);
    fn SetTimer(hwnd: HWND, id: usize, ms: u32, proc: *mut c_void) -> usize;
    fn SendMessageW(hwnd: HWND, msg: u32, w: usize, l: isize) -> isize;
    fn SetWindowTextW(hwnd: HWND, text: *const u16) -> i32;
    fn GetWindowTextW(hwnd: HWND, buf: *mut u16, max: i32) -> i32;
    fn LoadLibraryW(name: *const u16) -> *mut c_void;
    fn ShowWindow(hwnd: HWND, cmd: i32) -> i32;
    fn DestroyWindow(hwnd: HWND) -> i32;
    fn InvalidateRect(hwnd: HWND, rect: *const c_void, erase: i32) -> i32;
    fn FillRect(hdc: *mut c_void, rect: *const [i32; 4], brush: *mut c_void) -> i32;
    fn LoadIconW(instance: HINSTANCE, name: *const u16) -> *mut c_void;
    fn GetParent(hwnd: HWND) -> HWND;
    fn GetFocus() -> HWND;
    fn GetLocalTime(out: *mut SystemTime);
    fn EnableWindow(hwnd: HWND, enable: i32) -> i32;
    fn CreateMutexW(attrs: *mut c_void, owner: i32, name: *const u16) -> *mut c_void;
    fn GetLastError() -> u32;
    fn FindWindowW(class: *const u16, title: *const u16) -> HWND;
    fn SetForegroundWindow(hwnd: HWND) -> i32;
    fn LockFileEx(file: *mut c_void, flags: u32, reserved: u32, low: u32, high: u32, overlapped: *mut c_void) -> i32;
}
#[link(name = "comdlg32")]
extern "system" {
    fn GetOpenFileNameW(ofn: *mut OpenFile) -> i32;
    fn ChooseColorW(cc: *mut ChooseColor) -> i32;
}
#[link(name = "shell32")]
extern "system" {
    fn SHBrowseForFolderW(info: *mut BrowseInfo) -> *mut c_void;
    fn SHGetPathFromIDListW(pidl: *mut c_void, path: *mut u16) -> i32;
}
#[repr(C)]
struct OpenFile {
    size: u32,
    owner: HWND,
    instance: HINSTANCE,
    filter: *const u16,
    custom: *mut u16,
    max_custom: u32,
    filter_index: u32,
    file: *mut u16,
    max_file: u32,
    title_buf: *mut u16,
    max_title: u32,
    initial: *const u16,
    title: *const u16,
    flags: u32,
    file_offset: u16,
    ext_offset: u16,
    def_ext: *const u16,
    cust: usize,
    hook: usize,
    template: *const u16,
    reserved: *mut c_void,
    reserved_n: u32,
    flags_ex: u32,
}
#[repr(C)]
#[repr(C)]
struct ChooseColor {
    size: u32,
    owner: HWND,
    instance: HWND,
    rgb: u32,
    custom: *mut u32,
    flags: u32,
    data: usize,
    hook: usize,
    template: *const u16,
}
struct BrowseInfo {
    owner: HWND,
    root: *mut c_void,
    display: *mut u16,
    title: *const u16,
    flags: u32,
    callback: usize,
    param: isize,
    image: i32,
}
#[link(name = "gdi32")]
extern "system" {
    fn CreateSolidBrush(color: u32) -> *mut c_void;
    fn SetBkColor(hdc: *mut std::ffi::c_void, color: u32) -> u32;
    fn SetTextColor(hdc: *mut std::ffi::c_void, color: u32) -> u32;
}
#[repr(C)]
struct CharFormat {
    cb_size: u32,
    mask: u32,
    effects: u32,
    height: i32,
    offset: i32,
    color: u32,
    charset: u8,
    pitch: u8,
    pad: [u8; 2],
    face: [u16; 32],
}

#[repr(C)]
struct WndClass {
    style: u32,
    wnd_proc: Option<unsafe extern "system" fn(HWND, u32, usize, isize) -> isize>,
    cls_extra: i32,
    wnd_extra: i32,
    instance: HINSTANCE,
    icon: *mut c_void,
    cursor: *mut c_void,
    background: *mut c_void,
    menu_name: *const u16,
    class_name: *const u16,
}

fn wide(s: &str) -> Vec<u16> {
    s.encode_utf16().chain(std::iter::once(0)).collect()
}

fn text_of(hwnd: HWND) -> String {
    let mut buf = [0u16; 512];
    let n = unsafe { GetWindowTextW(hwnd, buf.as_mut_ptr(), buf.len() as i32) };
    String::from_utf16_lossy(&buf[..n.max(0) as usize]).trim().to_string()
}

fn set_text(hwnd: HWND, s: &str) {
    let w = wide(s);
    unsafe { SetWindowTextW(hwnd, w.as_ptr()) };
}

fn mod_dir() -> PathBuf {
    PathBuf::from(r"C:/Mods/nightreign-ap")
}

fn me3_exe() -> PathBuf {
    let local = std::env::var("LOCALAPPDATA").unwrap_or_default();
    PathBuf::from(local).join(r"Programs\garyttierney\me3\bin\me3.exe")
}

fn replace_section_key(text: &str, section: &str, key: &str, value: &str) -> String {
    let mut out = Vec::new();
    let mut current = "";
    let mut hit = false;
    for line in text.lines() {
        let trimmed = line.trim();
        if trimmed.starts_with('[') {
            if current == section && !hit {
                out.push(format!("{key} = {value}"));
                hit = true;
            }
            current = trimmed;
            out.push(line.to_string());
            continue;
        }
        if current == section && trimmed.starts_with(key) && trimmed[key.len()..].trim_start().starts_with('=') {
            out.push(format!("{key} = {value}"));
            hit = true;
        } else {
            out.push(line.to_string());
        }
    }
    if !hit {
        out.push(format!("{key} = {value}"));
    }
    out.join("\n") + "\n"
}

fn replace_key(text: &str, key: &str, value: &str) -> String {
    let mut out = Vec::new();
    let mut hit = false;
    for line in text.lines() {
        let trimmed = line.trim();
        if trimmed.starts_with(key) && trimmed[key.len()..].trim_start().starts_with('=') {
            out.push(format!("{key} = {value}"));
            hit = true;
        } else {
            out.push(line.to_string());
        }
    }
    if !hit {
        out.push(format!("{key} = {value}"));
    }
    out.join("\n") + "\n"
}

fn toml_get(text: &str, section: &str, key: &str) -> Option<String> {
    let mut current = "";
    for line in text.lines() {
        let t = line.trim();
        if t.starts_with('[') { current = t; continue; }
        if current == section {
            if let Some((k, v)) = t.split_once('=') {
                if k.trim() == key { return Some(v.trim().trim_matches('"').to_string()); }
            }
        }
    }
    None
}

fn quoted(value: &str) -> String {
    if value.parse::<i32>().is_ok() { value.to_string() } else { format!("\"{value}\"") }
}

fn load_settings(app: &App) {
    let text = fs::read_to_string(app.dir.join("flags.toml")).unwrap_or_default();
    set_text(app.host, &toml_get(&text, "[ap]", "host").unwrap_or_else(|| "127.0.0.1:38281".into()));
    set_text(app.slot, &toml_get(&text, "[ap]", "slot").unwrap_or_else(|| "Player1".into()));
    set_text(app.pass, &toml_get(&text, "[ap]", "password").unwrap_or_default());
    if let Some(path) = toml_get(&text, "[paths]", "me3") {
        if !path.is_empty() { set_text(app.me3, &path); }
    }
}

fn save_connection(app: &App) {
    let path = app.dir.join("flags.toml");
    let mut text = fs::read_to_string(&path).unwrap_or_default();
    text = replace_key(&text, "host", &format!("\"{}\"", text_of(app.host)));
    text = replace_key(&text, "slot", &format!("\"{}\"", text_of(app.slot)));
    text = replace_key(&text, "password", &format!("\"{}\"", text_of(app.pass)));
    text = replace_section_key(&text, "[paths]", "me3", &format!("\"{}\"", text_of(app.me3).replace('\\', "/")));
    if let Err(e) = fs::write(&path, text) {
        append_log(app, &format!("Save failed: {e}"));
    }
    let _ = write_profile_paths(app);
}

fn hex_to_bgr(hex: &str) -> u32 {
    let rgb = u32::from_str_radix(hex.trim().trim_start_matches('#'), 16).unwrap_or(0);
    ((rgb & 0xff) << 16) | (rgb & 0xff00) | ((rgb >> 16) & 0xff)
}

fn bgr_to_hex(bgr: u32) -> String {
    format!("{:02X}{:02X}{:02X}", bgr & 0xff, (bgr >> 8) & 0xff, (bgr >> 16) & 0xff)
}

fn pick_color(owner: HWND, current: &str) -> Option<String> {
    let mut cc = unsafe { std::mem::zeroed::<ChooseColor>() };
    cc.size = std::mem::size_of::<ChooseColor>() as u32;
    cc.owner = owner;
    cc.rgb = hex_to_bgr(current);
    cc.custom = unsafe { CUST.as_mut_ptr() };
    cc.flags = 0x3;
    if unsafe { ChooseColorW(&mut cc) } == 0 { return None; }
    Some(bgr_to_hex(cc.rgb))
}

fn open_options(app: &App) {
    unsafe {
        if !OPT.is_null() { return; }
        let class = wide("NRAPOptions");
        let wc = WndClass { style: 0, wnd_proc: Some(opt_proc), cls_extra: 0, wnd_extra: 0, instance: std::ptr::null_mut(), icon: std::ptr::null_mut(), cursor: std::ptr::null_mut(), background: std::ptr::null_mut(), menu_name: std::ptr::null(), class_name: class.as_ptr() };
        RegisterClassW(&wc);
        let win = CreateWindowExW(0, class.as_ptr(), wide("NRAP Options").as_ptr(), WS_OVERLAPPEDWINDOW | WS_VISIBLE, 140, 80, 420, 790, std::ptr::null_mut(), 0, std::ptr::null_mut(), std::ptr::null_mut());
        let edit = wide("EDIT");
        let button = wide("BUTTON");
        let text = fs::read_to_string(app.dir.join("flags.toml")).unwrap_or_default();
        CreateWindowExW(0, wide("STATIC").as_ptr(), wide("Overlay").as_ptr(), WS_CHILD | WS_VISIBLE, 16, 12, 200, 20, win, 0, std::ptr::null_mut(), std::ptr::null_mut());
        let row = |label: &str, y: i32, id: isize, value: &str, color: bool| {
            CreateWindowExW(0, wide("STATIC").as_ptr(), wide(label).as_ptr(), WS_CHILD | WS_VISIBLE, 28, y + 4, 130, 20, win, 0, std::ptr::null_mut(), std::ptr::null_mut());
            if color {
                CreateWindowExW(0, button.as_ptr(), wide(value).as_ptr(), WS_CHILD | WS_VISIBLE | BS_OWNERDRAW, 168, y, 28, 28, win, id, std::ptr::null_mut(), std::ptr::null_mut())
            } else {
                let h = CreateWindowExW(WS_EX_CLIENTEDGE, edit.as_ptr(), wide("").as_ptr(), WS_CHILD | WS_VISIBLE, 168, y, 140, 24, win, id, std::ptr::null_mut(), std::ptr::null_mut());
                set_text(h, value);
                h
            }
        };
        let overlay = |key: &str, fallback: &str| toml_get(&text, "[overlay]", key).unwrap_or_else(|| fallback.into());
        let host_color = |key: &str, fallback: &str| toml_get(&text, "[host]", key).unwrap_or_else(|| fallback.into());
        let x = row("X", 40, 201, &overlay("x", "24"), false);
        let y = row("Y", 72, 202, &overlay("y", "center"), false);
        let width = row("Width", 104, 203, &overlay("width", "720"), false);
        let height = row("Height", 136, 204, &overlay("height", "220"), false);
        let fade = row("Fade seconds", 168, 205, &overlay("hold_seconds", "5"), false);
        let font = row("Font size", 200, 206, &overlay("font_size", "16"), false);
        let local = row("Local player", 240, 211, &overlay("color_local", "EE77FF"), true);
        let remote = row("Remote player", 276, 212, &overlay("color_remote", "EE77FF"), true);
        let item = row("Item", 312, 213, &overlay("color_item", "5DC8C8"), true);
        let location = row("Location", 348, 214, &overlay("color_location", "6BE36B"), true);
        let text_color = row("Text", 384, 215, &overlay("text_color", "E8D7A4"), true);
        CreateWindowExW(0, wide("STATIC").as_ptr(), wide("Host console").as_ptr(), WS_CHILD | WS_VISIBLE, 16, 432, 200, 20, win, 0, std::ptr::null_mut(), std::ptr::null_mut());
        let host_text = row("Text", 456, 221, &host_color("text_color", "000000"), true);
        let host_local = row("Local player", 492, 222, &host_color("color_local", "EE77FF"), true);
        let host_remote = row("Remote player", 528, 223, &host_color("color_remote", "EE77FF"), true);
        let host_item = row("Item", 564, 224, &host_color("color_item", "5DC8C8"), true);
        let host_location = row("Location", 600, 225, &host_color("color_location", "6BE36B"), true);
        CreateWindowExW(0, wide("STATIC").as_ptr(), wide("Debug").as_ptr(), WS_CHILD | WS_VISIBLE, 16, 640, 200, 20, win, 0, std::ptr::null_mut(), std::ptr::null_mut());
        let debug = CreateWindowExW(0, button.as_ptr(), wide("Debug log").as_ptr(), WS_CHILD | WS_VISIBLE | BS_AUTOCHECKBOX, 28, 668, 160, 24, win, 216, std::ptr::null_mut(), std::ptr::null_mut());
        let on = text.lines().any(|l| l.trim() == "debug = true");
        SendMessageW(debug, BM_SETCHECK, if on { 1 } else { 0 }, 0);
        CreateWindowExW(0, button.as_ptr(), wide("Close").as_ptr(), WS_CHILD | WS_VISIBLE, 110, 708, 200, 32, win, 220, std::ptr::null_mut(), std::ptr::null_mut());
        let boxed = Box::new(OptWin { x, y, width, height, fade, font, local, remote, item, location, text: text_color, host_text, host_local, host_remote, host_item, host_location, debug });
        OPT = Box::into_raw(boxed);
    }
}

fn save_options(app: &App) {
    let opt = unsafe {
        if OPT.is_null() { return; }
        &*OPT
    };
    let path = app.dir.join("flags.toml");
    let mut text = fs::read_to_string(&path).unwrap_or_default();
    let font = text_of(opt.font);
    text = replace_section_key(&text, "[overlay]", "x", &quoted(&text_of(opt.x)));
    text = replace_section_key(&text, "[overlay]", "y", &quoted(&text_of(opt.y)));
    text = replace_section_key(&text, "[overlay]", "width", &quoted(&text_of(opt.width)));
    text = replace_section_key(&text, "[overlay]", "height", &quoted(&text_of(opt.height)));
    text = replace_section_key(&text, "[overlay]", "hold_seconds", &quoted(&text_of(opt.fade)));
    text = replace_section_key(&text, "[overlay]", "font_size", &quoted(&font));
    text = replace_section_key(&text, "[overlay]", "color_local", &format!("\"{}\"", text_of(opt.local)));
    text = replace_section_key(&text, "[overlay]", "color_remote", &format!("\"{}\"", text_of(opt.remote)));
    text = replace_section_key(&text, "[overlay]", "color_item", &format!("\"{}\"", text_of(opt.item)));
    text = replace_section_key(&text, "[overlay]", "color_location", &format!("\"{}\"", text_of(opt.location)));
    text = replace_section_key(&text, "[overlay]", "text_color", &format!("\"{}\"", text_of(opt.text)));
    text = replace_section_key(&text, "[host]", "text_color", &format!("\"{}\"", text_of(opt.host_text)));
    text = replace_section_key(&text, "[host]", "color_local", &format!("\"{}\"", text_of(opt.host_local)));
    text = replace_section_key(&text, "[host]", "color_remote", &format!("\"{}\"", text_of(opt.host_remote)));
    text = replace_section_key(&text, "[host]", "color_item", &format!("\"{}\"", text_of(opt.host_item)));
    text = replace_section_key(&text, "[host]", "color_location", &format!("\"{}\"", text_of(opt.host_location)));
    let debug = unsafe { SendMessageW(opt.debug, BM_GETCHECK, 0, 0) } == 1;
    text = replace_key(&text, "debug", if debug { "true" } else { "false" });
    if let Err(e) = fs::write(&path, text) {
        append_log(app, &format!("Options save failed: {e}"));
    } else {
        append_log(app, &format!("Saved overlay options, font size {font}"));
        queue_command(app, "/reload");
    }
}

fn bgr(hex: &str) -> u32 {
    let hex = hex.trim().trim_start_matches('#');
    u32::from_str_radix(hex, 16).ok().map(|rgb| {
        let r = (rgb >> 16) & 0xff;
        let g = (rgb >> 8) & 0xff;
        let b = rgb & 0xff;
        (b << 16) | (g << 8) | r
    }).unwrap_or(0x00A4D7E8)
}

fn colors(app: &App) -> (u32, u32, u32, u32, u32, String) {
    let text = std::fs::read_to_string(app.dir.join("flags.toml")).unwrap_or_default();
    let mut local = bgr("EE77FF");
    let mut remote = bgr("EE77FF");
    let mut item = bgr("5DC8C8");
    let mut loc = bgr("6BE36B");
    let mut plain = bgr("000000");
    let mut slot = text_of(app.slot);
    let mut section = "";
    for line in text.lines() {
        let t = line.trim();
        if t.starts_with('[') { section = t; continue; }
        let Some((k, v)) = t.split_once('=') else { continue };
        let v = v.trim().trim_matches('"');
        if section == "[host]" {
            match k.trim() {
                "color_local" => local = bgr(v),
                "color_remote" => remote = bgr(v),
                "color_item" => item = bgr(v),
                "color_location" => loc = bgr(v),
                "text_color" => plain = bgr(v),
                _ => {}
            }
        }
        if section == "[ap]" && k.trim() == "slot" && slot.is_empty() { slot = v.to_string(); }
    }
    (plain, local, remote, item, loc, slot)
}

fn paint(app: &App, text: &str, color: u32) {
    let w = wide(text);
    unsafe {
        SendMessageW(app.log, EM_SETSEL, usize::MAX, -1);
        let mut fmt = std::mem::zeroed::<CharFormat>();
        fmt.cb_size = 92;
        fmt.mask = CFM_COLOR;
        fmt.color = color;
        SendMessageW(app.log, EM_SETCHARFORMAT, SCF_SELECTION, &fmt as *const _ as isize);
        SendMessageW(app.log, EM_REPLACESEL, 0, w.as_ptr() as isize);
        SendMessageW(app.log, EM_SCROLLCARET, 0, 0);
    }
}

fn append_colored(app: &App, line: &str) {
    let (plain, local, remote, item, loc, slot) = colors(app);
    let player = |name: &str| if !slot.is_empty() && name.trim() == slot { local } else { remote };
    let body = line.split("NRAP AP | ").nth(1).unwrap_or(line);
    let prefix = &line[..line.len() - body.len()];
    paint(app, prefix, plain);
    if let Some(idx) = body.find(" found their ") {
        let (who, rest) = body.split_at(idx);
        let rest = &rest[" found their ".len()..];
        paint(app, who, player(who));
        paint(app, " found their ", plain);
        if let Some(open) = rest.rfind(" (") {
            paint(app, &rest[..open], item);
            paint(app, " (", plain);
            paint(app, rest[open+2..].trim_end_matches(')'), loc);
            paint(app, ")", plain);
        } else { paint(app, rest, item); }
    } else if let Some(sent) = body.find(" sent ") {
        let (who, rest) = body.split_at(sent);
        let rest = &rest[" sent ".len()..];
        paint(app, who, player(who));
        paint(app, " sent ", plain);
        if let Some(to) = rest.find(" to ") {
            paint(app, &rest[..to], item);
            paint(app, " to ", plain);
            let rest = &rest[to + 4..];
            if let Some(open) = rest.rfind(" (") {
                paint(app, &rest[..open], player(&rest[..open]));
                paint(app, " (", plain);
                paint(app, rest[open+2..].trim_end_matches(')'), loc);
                paint(app, ")", plain);
            } else { paint(app, rest, player(rest)); }
        } else { paint(app, rest, plain); }
    } else {
        paint_names(app, body, plain, local, remote, &slot);
    }
    paint(app, "\r\n", plain);
}

fn paint_names(app: &App, body: &str, plain: u32, local: u32, remote: u32, slot: &str) {
    if slot.is_empty() {
        paint(app, body, plain);
        return;
    }
    let mut rest = body;
    while let Some(idx) = rest.find(slot) {
        paint(app, &rest[..idx], plain);
        paint(app, slot, local);
        rest = &rest[idx + slot.len()..];
    }
    paint(app, rest, plain);
    let _ = remote;
}

fn scroll_bottom(log: HWND) {
    unsafe {
        let lines = SendMessageW(log, 0x00BA, 0, 0); // EM_GETLINECOUNT
        SendMessageW(log, EM_SETSEL, usize::MAX, -1);
        SendMessageW(log, 0x00B6, 0, lines); // EM_LINESCROLL
        SendMessageW(log, EM_SCROLLCARET, 0, 0);
        SendMessageW(log, 0x0115, 7, 0); // WM_VSCROLL SB_BOTTOM
    }
}

fn append_log(app: &App, line: &str) {
    append_colored(app, line);
    scroll_bottom(app.log);
}

fn read_new(path: &PathBuf, off: u64) -> Option<(u64, String)> {
    let meta = fs::metadata(path).ok()?;
    let len = meta.len();
    let off = if len < off { 0 } else { off };
    if len == off { return None; }
    let bytes = fs::read(path).ok()?;
    if off as usize >= bytes.len() { return None; }
    Some((len, String::from_utf8_lossy(&bytes[off as usize..]).into_owned()))
}

fn append_raw(app: &App, chunk: &str) {
    let w = wide(&chunk.replace('\n', "\r\n"));
    unsafe {
        SendMessageW(app.log, EM_SETSEL, usize::MAX, -1);
        SendMessageW(app.log, EM_REPLACESEL, 0, w.as_ptr() as isize);
        scroll_bottom(app.log);
    }
}

fn set_connected(app: &mut App, on: bool) {
    if app.connected == on { return; }
    app.connected = on;
    unsafe {
        EnableWindow(app.cmd, if on { 1 } else { 0 });
        EnableWindow(app.send, if on { 1 } else { 0 });
    }
}

fn note_connection(app: &mut App, chunk: &str) {
    let mut state: Option<bool> = None;
    for line in chunk.lines() {
        if line.contains("NRAP AP not connected") || line.contains("NRAP AP socket ended") || line.contains("NRAP AP disconnecting") {
            state = Some(false);
        } else if line.contains("NRAP AP connected") {
            state = Some(true);
        }
    }
    if let Some(on) = state {
        set_connected(app, on);
    }
}

fn tail(app: &mut App) {
    let primary = app.dir.join("nrap.log");
    let legacy = PathBuf::from(r"C:/Mods/nightreign-ap/nrap.log");
    if let Some((len, chunk)) = read_new(&primary, app.log_off) {
        app.log_off = len;
        append_raw(app, &chunk);
        note_connection(app, &chunk);
    }
    if legacy != primary {
        if let Some((len, chunk)) = read_new(&legacy, app.mod_off) {
            app.mod_off = len;
            append_raw(app, &chunk);
            note_connection(app, &chunk);
        }
    }
}

fn write_command(path: &PathBuf, line: &str) {
    let mut existing = fs::read_to_string(path).unwrap_or_default();
    if !existing.is_empty() && !existing.ends_with('\n') { existing.push('\n'); }
    existing.push_str(line);
    existing.push('\n');
    let _ = fs::write(path, existing);
}

fn command_dirs(app: &App) -> Vec<PathBuf> {
    let mut dirs = vec![app.dir.clone()];
    let legacy = PathBuf::from(r"C:/Mods/nightreign-ap");
    if legacy.is_dir() && legacy != app.dir { dirs.push(legacy); }
    dirs
}

fn queue_command(app: &App, line: &str) {
    for dir in command_dirs(app) {
        write_command(&dir.join("commands.txt"), line);
    }
}

fn request_reconnect(app: &App) {
    save_connection(app);
    for dir in command_dirs(app) {
        let _ = fs::write(dir.join("reconnect.trigger"), "1");
        write_command(&dir.join("commands.txt"), "/reconnect");
    }
    append_log(app, "Reconnect requested");
}

const CMD_HINT: &str = "!hint, !help ...";

fn send_command(app: &App) {
    let line = text_of(app.cmd);
    if line.is_empty() || line == CMD_HINT {
        return;
    }
    let path = app.dir.join("commands.txt");
    let mut existing = fs::read_to_string(&path).unwrap_or_default();
    if !existing.is_empty() && !existing.ends_with('\n') {
        existing.push('\n');
    }
    existing.push_str(&line);
    existing.push('\n');
    match fs::write(&path, existing) {
        Ok(()) => append_log(app, &format!("> {line}")),
        Err(e) => append_log(app, &format!("Command failed: {e}")),
    }
    set_text(app.cmd, "");
}

fn profile_path() -> Option<PathBuf> {
    let local = std::env::var("LOCALAPPDATA").unwrap_or_default();
    if local.is_empty() {
        return None;
    }
    let dir = PathBuf::from(local).join(r"garyttierney\me3\config\profiles");
    if fs::create_dir_all(&dir).is_err() {
        return None;
    }
    Some(dir.join("nightreign-ap.me3"))
}

fn profile_template(nrap: &str, reg: &str, nrsc: &str) -> String {
    format!(
        "profileVersion = \"v1\"\n\n[[supports]]\ngame = \"nightreign\"\n\n# Generated by nrap. Paths use forward slashes.\n\n[[natives]]\npath = '{nrsc}'\noptional = false\nload_early = true\n\n[[natives]]\npath = '{nrap}'\noptional = false\nload_early = false\n\n[[packages]]\nid = \"nrap-regulation\"\npath = '{reg}'\n"
    )
}

fn rotate_log(dir: &PathBuf) {
    let live = dir.join("nrap.log");
    if !live.is_file() {
        let _ = fs::write(&live, "");
        return;
    }
    let logs = dir.join("logs");
    let _ = fs::create_dir_all(&logs);
    let mut now = SystemTime { year: 0, month: 0, dow: 0, day: 0, hour: 0, minute: 0, second: 0, ms: 0 };
    unsafe { GetLocalTime(&mut now); }
    let stamp = format!("nrap-{:04}-{:02}-{:02}-{:02}-{:02}-{:02}.log", now.year, now.month, now.day, now.hour, now.minute, now.second);
    let mut dest = logs.join(&stamp);
    let mut n = 2;
    while dest.exists() {
        dest = logs.join(format!("nrap-{:04}-{:02}-{:02}-{:02}-{:02}-{:02}-{n}.log", now.year, now.month, now.day, now.hour, now.minute, now.second));
        n += 1;
    }
    if fs::rename(&live, &dest).is_err() {
        if fs::copy(&live, &dest).is_ok() {
            let _ = fs::write(&live, "");
        }
    }
    if !live.exists() {
        let _ = fs::write(&live, "");
    }
}

fn exe_dir() -> PathBuf {
    std::env::current_exe().ok().and_then(|p| p.parent().map(|p| p.to_path_buf())).unwrap_or_else(mod_dir)
}

fn steam_nrsc() -> Option<PathBuf> {
    let mut roots = vec![
        PathBuf::from(r"C:\Program Files (x86)\Steam"),
        PathBuf::from(r"C:\Program Files\Steam"),
    ];
    if let Ok(local) = std::env::var("PROGRAMFILES(X86)") {
        roots.push(PathBuf::from(local).join("Steam"));
    }
    let libraries = roots.iter().map(|r| r.join(r"steamapps\libraryfolders.vdf")).collect::<Vec<_>>();
    for vdf in libraries {
        if let Ok(text) = fs::read_to_string(&vdf) {
            for line in text.lines() {
                let line = line.trim().trim_matches('"');
                if line.contains(":\\") || line.contains(":/") {
                    roots.push(PathBuf::from(line.replace("\\\\", "\\")));
                }
            }
        }
    }
    for root in roots {
        let dll = root.join(r"steamapps\common\ELDEN RING NIGHTREIGN\Game\SeamlessCoop\nrsc.dll");
        if dll.is_file() {
            return Some(dll);
        }
    }
    None
}

fn default_nrap() -> PathBuf { exe_dir().join("nightreign_ap.dll") }
fn default_reg() -> PathBuf { exe_dir().join("regulation.bin") }
fn default_nrsc() -> PathBuf {
    steam_nrsc().unwrap_or_else(|| PathBuf::from(r"C:\Program Files (x86)\Steam\steamapps\common\ELDEN RING NIGHTREIGN\Game\SeamlessCoop\nrsc.dll"))
}

fn slash(path: &str) -> String { path.replace('\\', "/") }

fn pick_file(owner: HWND, title: &str, filter_text: &str) -> Option<String> {
    let mut buf = [0u16; 520];
    let filter = wide(filter_text);
    let title = wide(title);
    let mut ofn = unsafe { std::mem::zeroed::<OpenFile>() };
    ofn.size = std::mem::size_of::<OpenFile>() as u32;
    ofn.owner = owner;
    ofn.filter = filter.as_ptr();
    ofn.file = buf.as_mut_ptr();
    ofn.max_file = buf.len() as u32;
    ofn.title = title.as_ptr();
    ofn.flags = 0x00080000 | 0x00001000; // OFN_EXPLORER | OFN_FILEMUSTEXIST
    if unsafe { GetOpenFileNameW(&mut ofn) } == 0 { return None; }
    let n = buf.iter().position(|c| *c == 0).unwrap_or(buf.len());
    Some(String::from_utf16_lossy(&buf[..n]))
}

fn pick_dir(owner: HWND) -> Option<String> {
    let title = wide("Regulation folder");
    let mut display = [0u16; 520];
    let mut info = BrowseInfo { owner, root: std::ptr::null_mut(), display: display.as_mut_ptr(), title: title.as_ptr(), flags: 0x41, callback: 0, param: 0, image: 0 };
    let pidl = unsafe { SHBrowseForFolderW(&mut info) };
    if pidl.is_null() { return None; }
    let mut path = [0u16; 520];
    let ok = unsafe { SHGetPathFromIDListW(pidl, path.as_mut_ptr()) };
    if ok == 0 { return None; }
    let n = path.iter().position(|c| *c == 0).unwrap_or(path.len());
    Some(String::from_utf16_lossy(&path[..n]))
}

fn refresh_paths(app: &App) {
    unsafe {
        InvalidateRect(app.nrsc, std::ptr::null(), 1);
        InvalidateRect(app.me3, std::ptr::null(), 1);
    }
}

fn write_profile_paths(app: &App) -> Result<(), String> {
    let Some(path) = profile_path() else { return Err("me3 profiles folder is not available".into()); };
    let nrap = slash(&default_nrap().display().to_string());
    let nrsc = slash(&text_of(app.nrsc));
    let reg = slash(&exe_dir().display().to_string());
    if !path.exists() {
        return fs::write(&path, profile_template(&nrap, &reg, &nrsc)).map_err(|e| e.to_string());
    }
    let text = fs::read_to_string(&path).map_err(|e| e.to_string())?;
    let mut seen = 0;
    let mut out = Vec::new();
    for line in text.lines() {
        if line.trim_start().starts_with("path") {
            seen += 1;
            let value = match seen {
                1 => &nrsc,
                2 => &nrap,
                _ => &reg,
            };
            let indent = &line[..line.len() - line.trim_start().len()];
            out.push(format!("{indent}path = '{value}'"));
        } else {
            out.push(line.to_string());
        }
    }
    fs::write(&path, out.join("\n") + "\n").map_err(|e| e.to_string())
}

fn launch(app: &App) {
    save_connection(app);
    let exe = PathBuf::from(text_of(app.me3));
    if !exe.exists() {
        append_log(app, &format!("me3 not found at {}", exe.display()));
        return;
    }
    let Some(profile) = profile_path() else {
        append_log(app, r"nightreign-ap.me3 was not found in the me3 profiles folder or C:/Mods/nightreign-ap");
        return;
    };
    if let Err(e) = write_profile_paths(app) {
        append_log(app, &format!("Could not write me3 paths: {e}"));
        return;
    }
    let profile = profile.display().to_string().replace('\\', "/");
    match Command::new(&exe)
        .arg("launch")
        .arg("--game")
        .arg("nightreign")
        .arg("-p")
        .arg(&profile)
        .spawn()
    {
        Ok(_) => { unsafe { ShowWindow(app.reconnect, 5); } append_log(app, &format!("Launched me3 -p {profile}")); }
        Err(e) => append_log(app, &format!("Launch failed: {e}")),
    }
}


unsafe extern "system" fn opt_proc(hwnd: HWND, msg: u32, w: usize, l: isize) -> isize {
    match msg {
        WM_DRAWITEM => {
            let item = l as *const [i32; 16];
            let hwnd_item = std::ptr::read_unaligned((l as *const u8).add(24) as *const HWND);
            let hdc = std::ptr::read_unaligned((l as *const u8).add(32) as *const *mut c_void);
            let rect = (l as *const u8).add(40) as *const [i32; 4];
            let brush = CreateSolidBrush(hex_to_bgr(&text_of(hwnd_item)));
            FillRect(hdc, rect, brush as *mut c_void);
            let _ = item;
            1
        }
        WM_COMMAND => {
            let id = (w & 0xffff) as isize;
            if (211..=215).contains(&id) && !OPT.is_null() {
                let opt = &*OPT;
                let button = match id { 211 => opt.local, 212 => opt.remote, 213 => opt.item, 214 => opt.location, _ => opt.text };
                if let Some(hex) = pick_color(hwnd, &text_of(button)) { set_text(button, &hex); InvalidateRect(button, std::ptr::null(), 1); }
            }
            if id == 220 {
                if !APP.is_null() { save_options(&*APP); }
                OPT = std::ptr::null_mut();
                DestroyWindow(hwnd);
            }
            0
        }
        WM_DESTROY => {
            if !OPT.is_null() && !APP.is_null() { save_options(&*APP); }
            OPT = std::ptr::null_mut();
            0
        }
        _ => DefWindowProcW(hwnd, msg, w, l),
    }
}

unsafe extern "system" fn wnd_proc(hwnd: HWND, msg: u32, w: usize, l: isize) -> isize {
    match msg {
        WM_CTLCOLOREDIT => {
            let bad = !APP.is_null() && {
                let app = &*APP;
                let hwnd = l as HWND;
                (hwnd == app.nrsc && !PathBuf::from(text_of(app.nrsc)).is_file())
                    || (hwnd == app.me3 && !PathBuf::from(text_of(app.me3)).is_file())
            };
            let hint = !APP.is_null() && l as HWND == (*APP).cmd && text_of((*APP).cmd) == CMD_HINT;
            let color = if bad { 0x006464FF } else { 0x00FFFFFF };
            SetBkColor(w as *mut std::ffi::c_void, color);
            SetTextColor(w as *mut std::ffi::c_void, if hint { 0x00888888 } else { 0x00111111 });
            if bad { WARN_BRUSH } else { FIELD_BRUSH }
        }
        WM_CTLCOLORSTATIC => {
            SetBkColor(w as *mut std::ffi::c_void, 0x00F2F2F2);
            SetTextColor(w as *mut std::ffi::c_void, 0x00111111);
            LABEL_BRUSH
        }
        WM_COMMAND => {
            let id = (w & 0xffff) as isize;
            let note = (w >> 16) as u16;
            let app = &mut *APP;
            if id == CMD && note == 0x0100 && text_of(app.cmd) == CMD_HINT { set_text(app.cmd, ""); }
            if id == CMD && note == 0x0200 && text_of(app.cmd).is_empty() { set_text(app.cmd, CMD_HINT); }
            if note == EN_KILLFOCUS && matches!(id, HOST | SLOT | PASS | SEAMLESS | ME3) {
                save_connection(app);
                return 0;
            }
            match id {
                SEND => send_command(app),
                OPTIONS => open_options(app),
                RECONNECT => request_reconnect(app),
                LAUNCH => launch(app),
                BROWSE_NRSC => { if let Some(path) = pick_file(hwnd, "Seamless Coop DLL", "DLL\0*.dll\0All\0*.*\0\0") { set_text(app.nrsc, &path); refresh_paths(app); save_connection(app); } }
                BROWSE_ME3 => { if let Some(path) = pick_file(hwnd, "me3.exe", "Executable\0*.exe\0All\0*.*\0\0") { set_text(app.me3, &path); refresh_paths(app); save_connection(app); } }
                _ => {}
            }
            0
        }
        WM_TIMER => {
            tail(&mut *APP);
            refresh_paths(&*APP);
            if !SCROLLED {
                scroll_bottom((*APP).log);
                SCROLLED = true;
            }
            0
        }
        WM_DESTROY => {
            PostQuitMessage(0);
            0
        }
        _ => DefWindowProcW(hwnd, msg, w, l),
    }
}

fn raise_existing() {
    unsafe {
        let class = wide("NRAPHost");
        let hwnd = FindWindowW(class.as_ptr(), std::ptr::null());
        if !hwnd.is_null() {
            ShowWindow(hwnd, 9);
            SetForegroundWindow(hwnd);
        }
    }
}

fn instance_owned(dir: &std::path::Path) -> bool {
    unsafe {
        let name = wide("Local\\NRAPHostSingle");
        let handle = CreateMutexW(std::ptr::null_mut(), 1, name.as_ptr());
        if GetLastError() == 183 || handle.is_null() {
            raise_existing();
            return false;
        }
        let path = dir.join("nrap.instance");
        let file = std::fs::OpenOptions::new().read(true).write(true).create(true).open(&path);
        if let Ok(file) = file {
            let locked = LockFileEx(std::os::windows::io::AsRawHandle::as_raw_handle(&file) as *mut c_void, 3, 0, 1, 0, std::ptr::null_mut());
            if locked == 0 {
                raise_existing();
                return false;
            }
            std::mem::forget(file);
        }
        std::mem::forget(handle);
        true
    }
}

fn main() {
    let dir = exe_dir();
    if !instance_owned(&dir) {
        return;
    }
    rotate_log(&dir);
    let legacy = PathBuf::from(r"C:/Mods/nightreign-ap");
    if legacy.is_dir() && legacy != dir { rotate_log(&legacy); }
    let _ = fs::create_dir_all(&dir);
    unsafe {
        let class = wide("NRAPHost");
        let title = wide("NRAP");
        let wc = WndClass {
            style: 0,
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
        FIELD_BRUSH = CreateSolidBrush(0x00FFFFFF) as isize;
        LABEL_BRUSH = CreateSolidBrush(0x00F2F2F2) as isize;
        WARN_BRUSH = CreateSolidBrush(0x006464FF) as isize;
        let win = CreateWindowExW(0, class.as_ptr(), title.as_ptr(), WS_OVERLAPPEDWINDOW | WS_VISIBLE, 80, 80, 760, 840, std::ptr::null_mut(), 0, std::ptr::null_mut(), std::ptr::null_mut());
        LoadLibraryW(wide("Msftedit.dll").as_ptr());
        let edit = wide("EDIT");
        let rich = wide("RICHEDIT50W");
        let button = wide("BUTTON");
        let log = CreateWindowExW(WS_EX_CLIENTEDGE, rich.as_ptr(), wide("").as_ptr(), WS_CHILD | WS_VISIBLE | WS_VSCROLL | ES_MULTILINE | ES_AUTOVSCROLL | ES_READONLY, 12, 12, 720, 340, win, LOG, std::ptr::null_mut(), std::ptr::null_mut());
        SendMessageW(log, EM_SETBKGNDCOLOR, 0, 0x00E6E6E6);
        let cmd = CreateWindowExW(WS_EX_CLIENTEDGE, edit.as_ptr(), wide("").as_ptr(), WS_CHILD | WS_VISIBLE | WS_DISABLED, 12, 360, 640, 26, win, CMD, std::ptr::null_mut(), std::ptr::null_mut());
        let cue = wide("!hint, !help ...");
        SendMessageW(cmd, EM_SETCUEBANNER, 1, cue.as_ptr() as isize);
        let send = CreateWindowExW(0, button.as_ptr(), wide("Send").as_ptr(), WS_CHILD | WS_VISIBLE | WS_DISABLED, 660, 360, 72, 26, win, SEND, std::ptr::null_mut(), std::ptr::null_mut());
        CreateWindowExW(0, wide("STATIC").as_ptr(), wide("").as_ptr(), WS_CHILD | WS_VISIBLE | SS_ETCHEDHORZ, 12, 404, 720, 2, win, 0, std::ptr::null_mut(), std::ptr::null_mut());
        CreateWindowExW(0, wide("STATIC").as_ptr(), wide("Host").as_ptr(), WS_CHILD | WS_VISIBLE, 12, 428, 60, 20, win, 0, std::ptr::null_mut(), std::ptr::null_mut());
        let host = CreateWindowExW(WS_EX_CLIENTEDGE, edit.as_ptr(), wide("").as_ptr(), WS_CHILD | WS_VISIBLE, 80, 424, 240, 24, win, HOST, std::ptr::null_mut(), std::ptr::null_mut());
        CreateWindowExW(0, wide("STATIC").as_ptr(), wide("Slot").as_ptr(), WS_CHILD | WS_VISIBLE, 340, 428, 40, 20, win, 0, std::ptr::null_mut(), std::ptr::null_mut());
        let slot = CreateWindowExW(WS_EX_CLIENTEDGE, edit.as_ptr(), wide("").as_ptr(), WS_CHILD | WS_VISIBLE, 384, 424, 180, 24, win, SLOT, std::ptr::null_mut(), std::ptr::null_mut());
        CreateWindowExW(0, wide("STATIC").as_ptr(), wide("Password").as_ptr(), WS_CHILD | WS_VISIBLE, 12, 462, 64, 20, win, 0, std::ptr::null_mut(), std::ptr::null_mut());
        let pass = CreateWindowExW(WS_EX_CLIENTEDGE, edit.as_ptr(), wide("").as_ptr(), WS_CHILD | WS_VISIBLE | ES_PASSWORD, 80, 458, 240, 24, win, PASS, std::ptr::null_mut(), std::ptr::null_mut());
        let reconnect = CreateWindowExW(0, button.as_ptr(), wide("Reconnect").as_ptr(), WS_CHILD, 336, 456, 120, 28, win, RECONNECT, std::ptr::null_mut(), std::ptr::null_mut());
        CreateWindowExW(0, wide("STATIC").as_ptr(), wide("").as_ptr(), WS_CHILD | WS_VISIBLE | SS_ETCHEDHORZ, 12, 500, 720, 2, win, 0, std::ptr::null_mut(), std::ptr::null_mut());
        CreateWindowExW(0, wide("STATIC").as_ptr(), wide("Must point to nrsc.dll").as_ptr(), WS_CHILD | WS_VISIBLE, 88, 514, 220, 18, win, 0, std::ptr::null_mut(), std::ptr::null_mut());
        CreateWindowExW(0, wide("STATIC").as_ptr(), wide("Seamless").as_ptr(), WS_CHILD | WS_VISIBLE, 12, 542, 74, 20, win, 0, std::ptr::null_mut(), std::ptr::null_mut());
        let nrsc = CreateWindowExW(WS_EX_CLIENTEDGE, edit.as_ptr(), wide("").as_ptr(), WS_CHILD | WS_VISIBLE | ES_AUTOHSCROLL, 88, 538, 500, 24, win, SEAMLESS, std::ptr::null_mut(), std::ptr::null_mut());
        CreateWindowExW(0, button.as_ptr(), wide("Browse").as_ptr(), WS_CHILD | WS_VISIBLE, 596, 538, 70, 24, win, BROWSE_NRSC, std::ptr::null_mut(), std::ptr::null_mut());
        CreateWindowExW(0, wide("STATIC").as_ptr(), wide("Must point to me3.exe").as_ptr(), WS_CHILD | WS_VISIBLE, 88, 572, 220, 18, win, 0, std::ptr::null_mut(), std::ptr::null_mut());
        CreateWindowExW(0, wide("STATIC").as_ptr(), wide("me3").as_ptr(), WS_CHILD | WS_VISIBLE, 12, 600, 74, 20, win, 0, std::ptr::null_mut(), std::ptr::null_mut());
        let me3 = CreateWindowExW(WS_EX_CLIENTEDGE, edit.as_ptr(), wide("").as_ptr(), WS_CHILD | WS_VISIBLE | ES_AUTOHSCROLL, 88, 596, 500, 24, win, ME3, std::ptr::null_mut(), std::ptr::null_mut());
        CreateWindowExW(0, button.as_ptr(), wide("Browse").as_ptr(), WS_CHILD | WS_VISIBLE, 596, 596, 70, 24, win, BROWSE_ME3, std::ptr::null_mut(), std::ptr::null_mut());
        CreateWindowExW(0, wide("STATIC").as_ptr(), wide("").as_ptr(), WS_CHILD | WS_VISIBLE | SS_ETCHEDHORZ, 12, 636, 720, 2, win, 0, std::ptr::null_mut(), std::ptr::null_mut());
        CreateWindowExW(0, button.as_ptr(), wide("Options").as_ptr(), WS_CHILD | WS_VISIBLE, 280, 654, 200, 28, win, OPTIONS, std::ptr::null_mut(), std::ptr::null_mut());
        CreateWindowExW(0, button.as_ptr(), wide("Launch").as_ptr(), WS_CHILD | WS_VISIBLE, 280, 694, 200, 42, win, LAUNCH, std::ptr::null_mut(), std::ptr::null_mut());
        let mut app = App { log, cmd, send, host, slot, pass, nrsc, me3, reconnect, dir, log_off: 0, mod_off: 0, connected: false };
        set_text(nrsc, &default_nrsc().display().to_string());
        set_text(me3, &me3_exe().display().to_string());
        refresh_paths(&app);
        set_text(cmd, CMD_HINT);
        load_settings(&app);
        append_log(&app, "NRAP. Alt-tab here to send !commands or /debug on. Launch starts me3.");
        APP = &mut app;
        SetTimer(win, 1, 400, std::ptr::null_mut());
        let mut msg = [0usize; 7];
        while GetMessageW(&mut msg, std::ptr::null_mut(), 0, 0) > 0 {
            if msg[1] == 0x0100 && msg[2] == 0x0D && GetFocus() == app.cmd {
                send_command(&app);
                continue;
            }
            TranslateMessage(&msg);
            DispatchMessageW(&msg);
        }
    }
}
