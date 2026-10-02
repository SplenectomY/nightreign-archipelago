//! Phase 0 client. Attach, resolve event flags, submit AP checks, grant items.

#![cfg(windows)]

mod aob;
mod ap;
mod drop;
mod flag_write;
mod flagman;
mod grant;
mod hero;
mod menu;
mod scan;

use std::ffi::c_void;
use std::fs::{self, OpenOptions};
use std::io::{self, BufRead};
use std::io::Write;
use std::path::PathBuf;
use std::sync::mpsc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::OnceLock;
use std::thread;
use std::time::Duration;

type BOOL = i32;
type DWORD = u32;
type HANDLE = *mut c_void;
type HMODULE = *mut c_void;
type LPVOID = *mut c_void;

const DLL_PROCESS_ATTACH: DWORD = 1;
const STD_OUTPUT_HANDLE: DWORD = 0xFFFF_FFF5;
const STD_ERROR_HANDLE: DWORD = 0xFFFF_FFF4;
const GENERIC_READ: DWORD = 0x8000_0000;
const GENERIC_WRITE: DWORD = 0x4000_0000;
const FILE_SHARE_READ: DWORD = 1;
const FILE_SHARE_WRITE: DWORD = 2;
const OPEN_EXISTING: DWORD = 3;
const INVALID_HANDLE_VALUE: HANDLE = -1isize as HANDLE;
const ERROR_ALREADY_EXISTS: DWORD = 183;

static DLL_MODULE: OnceLock<usize> = OnceLock::new();

#[repr(C)]
struct SystemTime {
    year: u16,
    month: u16,
    day_of_week: u16,
    day: u16,
    hour: u16,
    minute: u16,
    second: u16,
    milliseconds: u16,
}

#[link(name = "kernel32")]
extern "system" {
    fn AllocConsole() -> BOOL;
    fn GetModuleHandleA(name: *const u8) -> HMODULE;
    fn GetModuleFileNameA(module: HMODULE, buf: *mut u8, size: DWORD) -> DWORD;
    fn GetStdHandle(kind: DWORD) -> HANDLE;
    fn SetStdHandle(kind: DWORD, handle: HANDLE) -> BOOL;
    fn CreateFileA(
        name: *const u8,
        access: DWORD,
        share: DWORD,
        security: LPVOID,
        creation: DWORD,
        flags: DWORD,
        template: HANDLE,
    ) -> HANDLE;
    fn ReadFile(
        file: HANDLE,
        buf: *mut u8,
        len: DWORD,
        read: *mut DWORD,
        overlapped: LPVOID,
    ) -> BOOL;
    fn SetConsoleMode(handle: HANDLE, mode: DWORD) -> BOOL;
    fn WriteFile(
        file: HANDLE,
        buf: *const u8,
        len: DWORD,
        written: *mut DWORD,
        overlapped: LPVOID,
    ) -> BOOL;
    fn CreateMutexA(sa: LPVOID, owner: BOOL, name: *const u8) -> HANDLE;
    fn GetLastError() -> DWORD;
    fn SetLastError(code: DWORD);
    fn GetLocalTime(out: *mut SystemTime);
}

fn timestamp() -> String {
    unsafe {
        let mut st = std::mem::zeroed::<SystemTime>();
        GetLocalTime(&mut st);
        format!(
            "{:04}-{:02}-{:02} {:02}:{:02}:{:02}.{:03}",
            st.year, st.month, st.day, st.hour, st.minute, st.second, st.milliseconds
        )
    }
}

#[no_mangle]
pub extern "system" fn DllMain(
    module: HMODULE,
    reason: DWORD,
    _reserved: LPVOID,
) -> BOOL {
    if reason == DLL_PROCESS_ATTACH {
        let _ = DLL_MODULE.set(module as usize);
        unsafe {
            SetLastError(0);
            let mtx = CreateMutexA(std::ptr::null_mut(), 1, b"Local\\NRAP_worker\0".as_ptr());
            if mtx.is_null() || GetLastError() == ERROR_ALREADY_EXISTS {
                return 1;
            }
        }
        thread::spawn(worker);
    }
    1
}

fn bind_console() {
    unsafe {
        AllocConsole();
        let con = CreateFileA(
            b"CONOUT$\0".as_ptr(),
            GENERIC_READ | GENERIC_WRITE,
            FILE_SHARE_READ | FILE_SHARE_WRITE,
            std::ptr::null_mut(),
            OPEN_EXISTING,
            0,
            std::ptr::null_mut(),
        );
        if con != INVALID_HANDLE_VALUE && !con.is_null() {
            SetStdHandle(STD_OUTPUT_HANDLE, con);
            SetStdHandle(STD_ERROR_HANDLE, con);
        }
        let input = CreateFileA(
            b"CONIN$\0".as_ptr(),
            GENERIC_READ | GENERIC_WRITE,
            FILE_SHARE_READ | FILE_SHARE_WRITE,
            std::ptr::null_mut(),
            OPEN_EXISTING,
            0,
            std::ptr::null_mut(),
        );
        if input != INVALID_HANDLE_VALUE && !input.is_null() {
            SetStdHandle(-10i32 as DWORD, input);
            SetConsoleMode(input, 0x1 | 0x2 | 0x4);
        }
    }
}

fn read_console_line() -> Option<String> {
    unsafe {
        let h = GetStdHandle(-10i32 as DWORD);
        if h.is_null() || h == INVALID_HANDLE_VALUE {
            return None;
        }
        let mut buf = [0u8; 512];
        let mut n = 0u32;
        if ReadFile(h, buf.as_mut_ptr(), buf.len() as DWORD, &mut n, std::ptr::null_mut()) == 0 {
            return None;
        }
        Some(String::from_utf8_lossy(&buf[..n as usize]).trim().to_string())
    }
}

fn write_console(msg: &str) {
    let line = format!("{msg}\r\n");
    unsafe {
        let h = GetStdHandle(STD_OUTPUT_HANDLE);
        if !h.is_null() && h != INVALID_HANDLE_VALUE {
            let mut written = 0u32;
            WriteFile(
                h,
                line.as_ptr(),
                line.len() as DWORD,
                &mut written,
                std::ptr::null_mut(),
            );
        }
    }
}

fn log_line(dir: &Option<PathBuf>, msg: &str) {
    let line = format!("[{}] {msg}", timestamp());
    write_console(&line);
    if let Some(dir) = dir {
        if let Ok(mut f) = OpenOptions::new()
            .create(true)
            .append(true)
            .open(dir.join("nrap.log"))
        {
            let _ = writeln!(f, "{line}");
        }
    }
}

fn dll_dir() -> Option<PathBuf> {
    let module = *DLL_MODULE.get().unwrap_or(&0) as HMODULE;
    let mut buf = vec![0u8; 520];
    let n = unsafe { GetModuleFileNameA(module, buf.as_mut_ptr(), buf.len() as DWORD) };
    if n == 0 {
        return None;
    }
    buf.truncate(n as usize);
    let path = String::from_utf8(buf).ok()?;
    PathBuf::from(path).parent().map(|p| p.to_path_buf())
}

#[derive(Clone)]
struct Watch {
    location: String,
    location_id: i64,
    flag: u32,
    last: Option<bool>,
    miss_logged: bool,
    submitted: bool,
}

fn toml_key_value(line: &str) -> Option<(&str, &str)> {
    let line = line.trim();
    if line.starts_with('#') || line.starts_with('[') {
        return None;
    }
    let (k, v) = line.split_once('=')?;
    Some((k.trim(), v.trim().trim_matches('"')))
}

fn parse_watches(text: &str) -> Vec<Watch> {
    let mut out = Vec::new();
    let mut location = None::<String>;
    let mut location_id = None::<i64>;
    let mut flag = None::<u32>;
    let flush = |location: &mut Option<String>,
                 location_id: &mut Option<i64>,
                 flag: &mut Option<u32>,
                 out: &mut Vec<Watch>| {
        if let (Some(loc), Some(id)) = (location.take(), flag.take()) {
            if id != 0 {
                out.push(Watch {
                    location: loc,
                    location_id: location_id.take().unwrap_or(0),
                    flag: id,
                    last: None,
                    miss_logged: false,
                    submitted: false,
                });
            }
        }
        let _ = location_id.take();
    };
    for line in text.lines() {
        if line.trim().starts_with("[[flag]]") {
            flush(&mut location, &mut location_id, &mut flag, &mut out);
            continue;
        }
        let Some((k, v)) = toml_key_value(line) else {
            continue;
        };
        if k == "location" {
            location = Some(v.to_string());
        }
        if k == "location_id" {
            location_id = v.parse().ok();
        }
        if k == "event_flag_id" {
            flag = v.parse().ok();
        }
    }
    flush(&mut location, &mut location_id, &mut flag, &mut out);
    out
}

fn find_config(dll_dir: Option<&PathBuf>) -> Option<PathBuf> {
    if let Some(dir) = dll_dir {
        let local = dir.join("flags.toml");
        if local.exists() {
            return Some(local);
        }
    }
    let appdata = std::env::var_os("LOCALAPPDATA")?;
    let path = PathBuf::from(appdata)
        .join("nightreign-archipelago")
        .join("flags.toml");
    if path.exists() {
        Some(path)
    } else {
        None
    }
}

static FLAG_DIFF: AtomicBool = AtomicBool::new(false);

fn worker() {
    let dir = dll_dir();
    bind_console();
    log_line(&dir, &format!("NRAP attached {}", env!("CARGO_PKG_VERSION")));
    let base = unsafe { GetModuleHandleA(std::ptr::null()) };
    log_line(&dir, &format!("NRAP nightreign.exe base = {base:p}"));
    log_line(
        &dir,
        &format!(
            "NRAP dll dir = {}",
            dir.as_ref()
                .map(|p| p.display().to_string())
                .unwrap_or_else(|| "<unknown>".into())
        ),
    );
    match drop::init() {
        Ok(msg) => log_line(&dir, &msg),
        Err(e) => log_line(&dir, &format!("NRAP drop init failed: {e}")),
    }
    match grant::init() {
        Ok(msg) => log_line(&dir, &msg),
        Err(e) => log_line(&dir, &format!("NRAP murk init failed: {e}")),
    }
    match hero::init() {
        Ok(msg) => log_line(&dir, &msg),
        Err(e) => log_line(&dir, &format!("NRAP hero hook failed: {e}")),
    }
    let config = find_config(dir.as_ref());
    log_line(
        &dir,
        &format!(
            "NRAP config = {}",
            config
                .as_ref()
                .map(|p| p.display().to_string())
                .unwrap_or_else(|| "<missing flags.toml>".into())
        ),
    );

    log_line(&dir, &flag_write::load_cache(dir.as_ref()));
    log_line(&dir, &flag_write::load_boss_kills(dir.as_ref()));
    let text = config.as_ref().and_then(|p| fs::read_to_string(p).ok());
    if let Some(text) = text.as_ref() {
        let in_pool = text.lines().any(|l| l.trim() == "heolstor_in_pool = true");
        let count = text.lines().find_map(|l| l.trim().strip_prefix("heolstor_unlock_count = ").and_then(|v| v.parse().ok())).unwrap_or(4);
        flag_write::configure_heolstor(in_pool, count);
    }
    let mut watches = text.as_deref().map(parse_watches).unwrap_or_default();
    if watches.is_empty() {
        log_line(&dir, "NRAP no event_flag_id values in flags.toml");
    } else {
        for w in &watches {
            log_line(
                &dir,
                &format!(
                    "NRAP watching {} flag {} loc {}",
                    w.location, w.flag, w.location_id
                ),
            );
        }
    }

    let (tx, rx) = mpsc::channel::<i64>();
    let (say_tx, say_rx) = mpsc::channel::<String>();
    if let Some(text) = text.as_deref() {
        let ap_cfg = ap::ApConfig::from_toml(text);
        let dir_ap = dir.clone();
        let drop_goods = drop::drop_item_id_from_toml(text);
        thread::spawn(move || {
            ap::run(ap_cfg, rx, say_rx, drop_goods, |msg| log_line(&dir_ap, msg));
        });
    }
    if text.as_deref().is_some_and(|t| t.lines().any(|l| l.trim() == "flag_diff = true")) {
        FLAG_DIFF.store(true, Ordering::SeqCst);
    }
    let console_dir = dir.clone();
    thread::spawn(move || {
        loop {
            let Some(line) = read_console_line() else {
                thread::sleep(Duration::from_millis(200));
                continue;
            };
            if line.starts_with('!') {
                let _ = say_tx.send(line);
                continue;
            }
            let lower = line.to_ascii_lowercase();
            if lower.starts_with("/flagdiff") {
                let arg = lower.split_whitespace().nth(1).unwrap_or("");
                let on = match arg {
                    "on" | "1" | "true" => true,
                    "off" | "0" | "false" => false,
                    _ => !FLAG_DIFF.load(Ordering::SeqCst),
                };
                FLAG_DIFF.store(on, Ordering::SeqCst);
                log_line(&console_dir, &format!("NRAP flagdiff {}", if on { "on" } else { "off" }));
            }
        }
    });

    let mut man = None;
    let mut fail_logged = false;
    let mut last_debug_flag = 0u32;
    let mut last_clear_flag = 0u32;
    let mut last_debug_drop = 0i32;
    loop {
        if man.is_none() {
            match flagman::resolve() {
                Ok(found) => {
                    log_line(&dir, &found.describe());
                    man = Some(found);
                }
                Err(e) => {
                    if !fail_logged {
                        log_line(&dir, &format!("NRAP flagman not ready: {e}"));
                        fail_logged = true;
                    }
                }
            }
        }
        if let Some(found) = man {
            static DAY: std::sync::Mutex<[(u32, Option<bool>); 8]> = std::sync::Mutex::new([
                (7502, None), (7507, None), (8140, None), (8145, None), (7001, None), (2000, None), (7512, None), (8155, None),
            ]);
            let mut day = DAY.lock().unwrap();
            for (flag, last) in day.iter_mut() {
                if let Some(on) = found.get(*flag) {
                    if *last != Some(on) {
                        log_line(&dir, &format!("NRAP dayflag {flag} {}->{}", last.map(|v| if v {"1"} else {"0"}).unwrap_or("?"), if on {"1"} else {"0"}));
                        if last.is_some() && on && matches!(*flag, 7512 | 7001 | 2000) {
                            let mut latched = Vec::new();
                            for counter in [8140u32, 8145] {
                                if found.get(counter) == Some(true) {
                                    flag_write::note_return(counter);
                                    latched.push(counter);
                                }
                            }
                            log_line(&dir, &format!("NRAP return signal {flag}, latching clears {latched:?}"));
                        }
                        if last.is_some() && on {
                            if let Some((n, loc)) = flag_write::note_day_boss(*flag) {
                                let _ = tx.send(loc);
                                log_line(&dir, &format!("NRAP day boss {flag} count {n} loc {loc}"));
                            }
                        }
                        if last.is_some() && matches!(*flag, 8140 | 8145) {
                            if let Some((n, loc)) = flag_write::note_toggle(*flag, on) {
                                let _ = tx.send(loc);
                                log_line(&dir, &format!("NRAP toggle {flag} count {n} loc {loc}"));
                            } else if !on {
                                log_line(&dir, &format!("NRAP toggle {flag} clear ignored"));
                            }
                        }
                        *last = Some(on);
                    }
                }
            }
            drop(day);
            for w in &mut watches {
                match found.get(w.flag) {
                    Some(on) => {
                        if w.last != Some(on) {
                            if w.last == Some(false) && on {
                                log_line(
                                    &dir,
                                    &format!("NRAP check: {} (flag {} 0->1)", w.location, w.flag),
                                );
                                if let Some(msg) = flag_write::note_defeat(w.flag) {
                                    log_line(&dir, &msg);
                                }
                            } else {
                                log_line(
                                    &dir,
                                    &format!(
                                        "NRAP {} flag {} = {}",
                                        w.location,
                                        w.flag,
                                        on as u8
                                    ),
                                );
                            }
                            w.last = Some(on);
                        }
                        if on && !w.submitted && w.location_id != 0 {
                            let _ = tx.send(w.location_id);
                            w.submitted = true;
                        }
                    }
                    None => {
                        if !w.miss_logged {
                            log_line(
                                &dir,
                                &format!("NRAP {} flag {} not in holder", w.location, w.flag),
                            );
                            w.miss_logged = true;
                        }
                    }
                }
            }
        }
        if let Some(msg) = flag_write::retry_pending() {
            log_line(&dir, &msg);
        }
        let _ = flag_write::reapply_cached();
        if let Some(msg) = hero::apply() {
            log_line(&dir, &msg);
        }
        if let Some(msg) = grant::retry() {
            log_line(&dir, &msg);
        }
        if let Some(msg) = drop::retry_pending() {
            log_line(&dir, &msg);
        }
        if FLAG_DIFF.load(Ordering::SeqCst) {
            if let Some(found) = man {
                static LAST: std::sync::Mutex<Option<std::time::Instant>> = std::sync::Mutex::new(None);
                static IDLE: std::sync::Mutex<Option<std::time::Instant>> = std::sync::Mutex::new(None);
                static PREV: std::sync::Mutex<Vec<Vec<u8>>> = std::sync::Mutex::new(Vec::new());
                let due = LAST.lock().unwrap().map(|t| t.elapsed().as_millis() >= 2000).unwrap_or(true);
                if due {
                    *LAST.lock().unwrap() = Some(std::time::Instant::now());
                    let (rose, groups) = found.diff_rising(&mut PREV.lock().unwrap());
                    if !rose.is_empty() {
                        let show: Vec<_> = rose.iter().take(24).map(|f| f.to_string()).collect();
                        log_line(&dir, &format!("NRAP flag diff +{} groups={groups} {}", rose.len(), show.join(",")));
                        *IDLE.lock().unwrap() = Some(std::time::Instant::now());
                    } else if IDLE.lock().unwrap().map(|t| t.elapsed().as_secs() >= 10).unwrap_or(true) {
                        *IDLE.lock().unwrap() = Some(std::time::Instant::now());
                        log_line(&dir, &format!("NRAP flag diff idle groups={groups}"));
                    }
                }
            }
        }
        if let Some(text) = config.as_ref().and_then(|p| fs::read_to_string(p).ok()) {
            if let Some(flag) = flag_write::debug_flag_from_toml(&text) {
                if flag != last_debug_flag {
                    last_debug_flag = flag;
                    match flag_write::set_flag(flag, true) {
                        Ok(msg) => log_line(&dir, &msg),
                        Err(e) => log_line(&dir, &format!("NRAP SetEventFlag failed: {e}")),
                    }
                }
            } else {
                last_debug_flag = 0;
            }
            if let Some(flag) = flag_write::debug_clear_flag_from_toml(&text) {
                if flag != last_clear_flag {
                    last_clear_flag = flag;
                    flag_write::suppress_flag(flag);
                    match flag_write::set_flag(flag, false) {
                        Ok(msg) => log_line(&dir, &msg),
                        Err(e) => log_line(&dir, &format!("NRAP clear_flag failed: {e}")),
                    }
                }
            } else {
                last_clear_flag = 0;
            }
            if let Some(id) = drop::debug_drop_from_toml(&text) {
                if id != last_debug_drop {
                    last_debug_drop = id;
                    log_line(&dir, &drop::debug_drop(id));
                }
            } else {
                last_debug_drop = 0;
            }
        }
        thread::sleep(Duration::from_millis(200));
    }
}
