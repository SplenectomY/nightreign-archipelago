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
const ES_PASSWORD: u32 = 0x0020;
const BS_AUTOCHECKBOX: u32 = 0x0003;
const WM_COMMAND: u32 = 0x0111;
const WM_TIMER: u32 = 0x0113;
const WM_DESTROY: u32 = 0x0002;
const BM_GETCHECK: u32 = 0x00F0;
const BM_SETCHECK: u32 = 0x00F1;
const EM_SETSEL: u32 = 0x00B1;
const EM_REPLACESEL: u32 = 0x00C2;

const LOG: isize = 100;
const CMD: isize = 101;
const SEND: isize = 102;
const HOST: isize = 103;
const SLOT: isize = 104;
const PASS: isize = 105;
const SAVE: isize = 106;
const LAUNCH: isize = 107;
const DEBUG: isize = 108;

static mut APP: *mut App = std::ptr::null_mut();

struct App {
    log: HWND,
    cmd: HWND,
    host: HWND,
    slot: HWND,
    pass: HWND,
    debug: HWND,
    dir: PathBuf,
    log_off: u64,
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
    PathBuf::from(r"C:\Mods\nightreign-ap")
}

fn me3_exe() -> PathBuf {
    let local = std::env::var("LOCALAPPDATA").unwrap_or_default();
    PathBuf::from(local).join(r"Programs\garyttierney\me3\bin\me3.exe")
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

fn load_settings(app: &App) {
    let path = app.dir.join("flags.toml");
    let text = fs::read_to_string(&path).unwrap_or_default();
    let mut host = "127.0.0.1:38281".to_string();
    let mut slot = "Player1".to_string();
    let mut pass = String::new();
    let mut debug = false;
    let mut section = "";
    for line in text.lines() {
        let t = line.trim();
        if t.starts_with('[') { section = t; continue; }
        let Some((k, v)) = t.split_once('=') else { continue };
        let v = v.trim().trim_matches('"');
        match (section, k.trim()) {
            ("[ap]", "host") => host = v.to_string(),
            ("[ap]", "slot") => slot = v.to_string(),
            ("[ap]", "password") => pass = v.to_string(),
            (_, "debug") if section.is_empty() || section == "[debug]" => debug = v == "true",
            _ => {}
        }
    }
    set_text(app.host, &host);
    set_text(app.slot, &slot);
    set_text(app.pass, &pass);
    unsafe { SendMessageW(app.debug, BM_SETCHECK, if debug { 1 } else { 0 }, 0) };
}

fn save_settings(app: &App) {
    let path = app.dir.join("flags.toml");
    let mut text = fs::read_to_string(&path).unwrap_or_default();
    text = replace_key(&text, "host", &format!("\"{}\"", text_of(app.host)));
    text = replace_key(&text, "slot", &format!("\"{}\"", text_of(app.slot)));
    text = replace_key(&text, "password", &format!("\"{}\"", text_of(app.pass)));
    let debug = unsafe { SendMessageW(app.debug, BM_GETCHECK, 0, 0) } == 1;
    text = replace_key(&text, "debug", if debug { "true" } else { "false" });
    match fs::write(&path, text) {
        Ok(()) => append_log(app, "Saved flags.toml. Host and password apply on the next connect."),
        Err(e) => append_log(app, &format!("Save failed: {e}")),
    }
}

fn append_log(app: &App, line: &str) {
    let w = wide(&format!("{line}\r\n"));
    unsafe {
        SendMessageW(app.log, EM_SETSEL, usize::MAX, -1);
        SendMessageW(app.log, EM_REPLACESEL, 0, w.as_ptr() as isize);
    }
}

fn tail(app: &mut App) {
    let path = app.dir.join("nrap.log");
    let Ok(meta) = fs::metadata(&path) else { return };
    let len = meta.len();
    if len < app.log_off {
        app.log_off = 0;
    }
    if len == app.log_off {
        return;
    }
    let Ok(bytes) = fs::read(&path) else { return };
    let start = app.log_off as usize;
    if start >= bytes.len() {
        return;
    }
    let chunk = String::from_utf8_lossy(&bytes[start..]).replace('\n', "\r\n");
    app.log_off = len;
    let w = wide(&chunk);
    unsafe {
        SendMessageW(app.log, EM_SETSEL, usize::MAX, -1);
        SendMessageW(app.log, EM_REPLACESEL, 0, w.as_ptr() as isize);
    }
}

fn send_command(app: &App) {
    let line = text_of(app.cmd);
    if line.is_empty() {
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
    let installed = PathBuf::from(local).join(r"garyttierney\me3\config\profiles\nightreign-ap.me3");
    if installed.exists() {
        return Some(installed);
    }
    let shipped = mod_dir().join("nightreign-ap.me3");
    shipped.exists().then_some(shipped)
}

fn launch(app: &App) {
    let exe = me3_exe();
    if !exe.exists() {
        append_log(app, &format!("me3 not found at {}", exe.display()));
        return;
    }
    let Some(profile) = profile_path() else {
        append_log(app, "nightreign-ap.me3 was not found in the me3 profiles folder or C:\Mods\nightreign-ap");
        return;
    };
    let profile = profile.display().to_string().replace('\\', "/");
    match Command::new(&exe)
        .arg("launch")
        .arg("--game")
        .arg("nightreign")
        .arg("-p")
        .arg(&profile)
        .spawn()
    {
        Ok(_) => append_log(app, &format!("Launched me3 -p {profile}")),
        Err(e) => append_log(app, &format!("Launch failed: {e}")),
    }
}

unsafe extern "system" fn wnd_proc(hwnd: HWND, msg: u32, w: usize, l: isize) -> isize {
    match msg {
        WM_COMMAND => {
            let id = (w & 0xffff) as isize;
            let app = &mut *APP;
            match id {
                SEND => send_command(app),
                SAVE => save_settings(app),
                LAUNCH => launch(app),
                _ => {}
            }
            0
        }
        WM_TIMER => {
            tail(&mut *APP);
            0
        }
        WM_DESTROY => {
            PostQuitMessage(0);
            0
        }
        _ => DefWindowProcW(hwnd, msg, w, l),
    }
}

fn main() {
    let dir = mod_dir();
    let _ = fs::create_dir_all(&dir);
    unsafe {
        let class = wide("NRAPHost");
        let title = wide("NRAP Host");
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
        let win = CreateWindowExW(0, class.as_ptr(), title.as_ptr(), WS_OVERLAPPEDWINDOW | WS_VISIBLE, 80, 80, 760, 640, std::ptr::null_mut(), 0, std::ptr::null_mut(), std::ptr::null_mut());
        let edit = wide("EDIT");
        let button = wide("BUTTON");
        let log = CreateWindowExW(0, edit.as_ptr(), wide("").as_ptr(), WS_CHILD | WS_VISIBLE | WS_VSCROLL | ES_MULTILINE | ES_AUTOVSCROLL | ES_READONLY, 12, 12, 720, 360, win, LOG, std::ptr::null_mut(), std::ptr::null_mut());
        let cmd = CreateWindowExW(0, edit.as_ptr(), wide("").as_ptr(), WS_CHILD | WS_VISIBLE, 12, 382, 560, 24, win, CMD, std::ptr::null_mut(), std::ptr::null_mut());
        CreateWindowExW(0, button.as_ptr(), wide("Send").as_ptr(), WS_CHILD | WS_VISIBLE, 580, 380, 70, 26, win, SEND, std::ptr::null_mut(), std::ptr::null_mut());
        CreateWindowExW(0, button.as_ptr(), wide("Launch").as_ptr(), WS_CHILD | WS_VISIBLE, 656, 380, 76, 26, win, LAUNCH, std::ptr::null_mut(), std::ptr::null_mut());
        CreateWindowExW(0, wide("STATIC").as_ptr(), wide("Host").as_ptr(), WS_CHILD | WS_VISIBLE, 12, 420, 60, 20, win, 0, std::ptr::null_mut(), std::ptr::null_mut());
        let host = CreateWindowExW(0, edit.as_ptr(), wide("").as_ptr(), WS_CHILD | WS_VISIBLE, 80, 418, 240, 22, win, HOST, std::ptr::null_mut(), std::ptr::null_mut());
        CreateWindowExW(0, wide("STATIC").as_ptr(), wide("Slot").as_ptr(), WS_CHILD | WS_VISIBLE, 330, 420, 40, 20, win, 0, std::ptr::null_mut(), std::ptr::null_mut());
        let slot = CreateWindowExW(0, edit.as_ptr(), wide("").as_ptr(), WS_CHILD | WS_VISIBLE, 372, 418, 160, 22, win, SLOT, std::ptr::null_mut(), std::ptr::null_mut());
        CreateWindowExW(0, wide("STATIC").as_ptr(), wide("Password").as_ptr(), WS_CHILD | WS_VISIBLE, 12, 450, 64, 20, win, 0, std::ptr::null_mut(), std::ptr::null_mut());
        let pass = CreateWindowExW(0, edit.as_ptr(), wide("").as_ptr(), WS_CHILD | WS_VISIBLE | ES_PASSWORD, 80, 448, 240, 22, win, PASS, std::ptr::null_mut(), std::ptr::null_mut());
        let debug = CreateWindowExW(0, button.as_ptr(), wide("Debug log").as_ptr(), WS_CHILD | WS_VISIBLE | BS_AUTOCHECKBOX, 340, 448, 120, 22, win, DEBUG, std::ptr::null_mut(), std::ptr::null_mut());
        CreateWindowExW(0, button.as_ptr(), wide("Save settings").as_ptr(), WS_CHILD | WS_VISIBLE, 480, 446, 120, 26, win, SAVE, std::ptr::null_mut(), std::ptr::null_mut());
        let mut app = App { log, cmd, host, slot, pass, debug, dir, log_off: 0 };
        load_settings(&app);
        append_log(&app, "NRAP Host. Alt-tab here to send !commands or /debug on. Launch starts me3.");
        APP = &mut app;
        SetTimer(win, 1, 400, std::ptr::null_mut());
        let mut msg = [0usize; 7];
        while GetMessageW(&mut msg, std::ptr::null_mut(), 0, 0) > 0 {
            TranslateMessage(&msg);
            DispatchMessageW(&msg);
        }
    }
}
