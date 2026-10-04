//! Client. Attach, resolve event flags, submit AP checks, grant items.

#![cfg(windows)]

mod aob;
mod ap;
mod drop;
mod flag_write;
mod flagman;
mod grant;
mod hero;
mod menu;
mod names;
mod overlay;
mod watches;

use std::ffi::c_void;
use std::fs::{self, OpenOptions};
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

#[link(name = "user32")]
extern "system" {
    fn ShowWindow(hwnd: *mut c_void, cmd: i32) -> BOOL;
}

#[link(name = "kernel32")]
extern "system" {
    fn AllocConsole() -> BOOL;
    fn GetConsoleWindow() -> *mut c_void;
    fn SetConsoleTitleW(title: *const u16) -> BOOL;
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
        let title: Vec<u16> = format!("NRAP {}", env!("CARGO_PKG_VERSION"))
            .encode_utf16()
            .chain(std::iter::once(0))
            .collect();
        SetConsoleTitleW(title.as_ptr());
        let hwnd = GetConsoleWindow();
        if !hwnd.is_null() {
            ShowWindow(hwnd, 0);
        }
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
            CONOUT.store(con as usize, std::sync::atomic::Ordering::SeqCst);
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


fn take_command_file(dir: &Option<PathBuf>) -> Option<String> {
    let path = dir.as_ref()?.join("commands.txt");
    let text = std::fs::read_to_string(&path).ok()?;
    let line = text.lines().find(|l| !l.trim().is_empty())?.trim().to_string();
    let rest: String = text.lines().skip_while(|l| l.trim().is_empty()).skip(1).collect::<Vec<_>>().join("\n");
    let _ = std::fs::write(&path, if rest.is_empty() { String::new() } else { rest + "\n" });
    Some(line)
}

static CONOUT: std::sync::atomic::AtomicUsize = std::sync::atomic::AtomicUsize::new(0);

fn write_console(msg: &str) {
    let line = format!("{msg}\r\n");
    let stored = CONOUT.load(std::sync::atomic::Ordering::SeqCst) as HANDLE;
    unsafe {
        let h = if !stored.is_null() && stored != INVALID_HANDLE_VALUE {
            stored
        } else {
            GetStdHandle(STD_OUTPUT_HANDLE)
        };
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

static DEBUG: AtomicBool = AtomicBool::new(false);
static LOG_STICKY: AtomicBool = AtomicBool::new(false);
static LOG_DAYFLAGS: AtomicBool = AtomicBool::new(false);
static LOG_REWRITES: AtomicBool = AtomicBool::new(false);

fn apply_debug_flags(text: &str) {
    let on = |key: &str| text.lines().any(|l| l.trim() == format!("{key} = true"));
    DEBUG.store(on("debug"), Ordering::SeqCst);
    LOG_STICKY.store(on("log_sticky"), Ordering::SeqCst);
    LOG_DAYFLAGS.store(on("log_dayflags"), Ordering::SeqCst);
    LOG_REWRITES.store(on("log_rewrites"), Ordering::SeqCst);
    FLAG_DIFF.store(on("debug") && on("log_flagdiffs"), Ordering::SeqCst);
}

fn console_visible(msg: &str) -> bool {
    DEBUG.load(Ordering::SeqCst)
        || msg.starts_with("NRAP attached")
        || msg.starts_with("NRAP AP connected")
        || msg.starts_with("NRAP AP socket")
        || msg.starts_with("NRAP AP not connected")
        || msg.starts_with("NRAP AP |")
        || msg.starts_with("NRAP debug")
}

fn log_line(dir: &Option<PathBuf>, msg: &str) {
    let line = format!("[{}] {msg}", timestamp());
    if console_visible(msg) {
        write_console(&line);
        overlay::push(&line);
    }
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
    ignored: bool,
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
    if let Some(body) = text.as_deref() {
        overlay::start(body);
        log_line(&dir, "NRAP overlay started");
    }
    let watches: Vec<Watch> = watches::WATCHES
        .iter()
        .map(|w| Watch {
            location: w.location.to_string(),
            location_id: w.location_id,
            flag: w.flag,
            last: None,
            miss_logged: false,
            submitted: false,
            ignored: false,
        })
        .collect();
    if watches.is_empty() {
        log_line(&dir, "NRAP no hardcoded location watches");
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
        let drop_goods = 0;
        thread::spawn(move || {
            let config_path = dir_ap.clone().unwrap_or_default().join("flags.toml");
            ap::run(ap_cfg, rx, say_rx, drop_goods, config_path, |msg| log_line(&dir_ap, msg));
        });
    }
    if let Some(text) = text.as_deref() {
        apply_debug_flags(text);
    }
    let console_dir = dir.clone();
    thread::spawn(move || {
        loop {
            let Some(line) = take_command_file(&console_dir) else {
                thread::sleep(Duration::from_millis(200));
                continue;
            };
            log_line(&console_dir, &format!("NRAP command {line}"));
            if line.starts_with('!') {
                let _ = say_tx.send(line);
                continue;
            }
            let lower = line.to_ascii_lowercase();
            if lower == "/reconnect" {
                ap::request_reconnect();
                log_line(&console_dir, "NRAP AP reconnect requested");
                continue;
            }
            if lower == "/reload" {
                if let Some(dir) = console_dir.as_ref() {
                    if let Ok(text) = std::fs::read_to_string(dir.join("flags.toml")) {
                        overlay::apply(&text);
                        apply_debug_flags(&text);
                        log_line(&console_dir, &format!("NRAP options applied, debug {}", if DEBUG.load(Ordering::SeqCst) { "on" } else { "off" }));
                    }
                }
                continue;
            }
            if lower.starts_with("/debug") {
                let arg = lower.split_whitespace().nth(1).unwrap_or("");
                let on = match arg {
                    "on" | "1" | "true" => true,
                    "off" | "0" | "false" => false,
                    _ => !DEBUG.load(Ordering::SeqCst),
                };
                DEBUG.store(on, Ordering::SeqCst);
                log_line(&console_dir, &format!("NRAP debug {}", if on { "on" } else { "off" }));
            }
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

    let watch_tx = tx.clone();
    let watch_dir = dir.clone();
    log_line(&dir, "NRAP watch thread started");
    thread::spawn(move || {
        let mut watches = watches;
        let mut man = None;
        let mut fail_logged = false;
        log_line(&watch_dir, "NRAP watch thread running");
        loop {
        if man.is_none() {
            match flagman::resolve() {
                Ok(found) => {
                    log_line(&watch_dir, &found.describe());
                    man = Some(found);
                }
                Err(e) => {
                    if !fail_logged {
                        log_line(&watch_dir, &format!("NRAP flagman not ready: {e}"));
                        fail_logged = true;
                    }
                }
            }
        }
        if let Some(found) = man {
            let in_game = found.get(2030) == Some(true);
            if flag_write::in_session() != in_game {
                flag_write::set_in_session(in_game);
                log_line(&watch_dir, &format!("NRAP session {}", if in_game { "in game" } else { "title, flag work paused" }));
            }
            if !in_game {
                if flag_write::in_expedition() {
                    flag_write::set_in_expedition(false);
                }
                thread::sleep(Duration::from_millis(200));
                continue;
            }
            let expedition = found.get(7500) == Some(true) || found.get(7505) == Some(true) || found.get(7510) == Some(true);
            if flag_write::in_expedition() != expedition {
                flag_write::set_in_expedition(expedition);
                log_line(&watch_dir, &format!("NRAP expedition {}", if expedition { "started, unlock writes paused" } else { "ended" }));
            }
            static DAY: std::sync::Mutex<[(u32, Option<bool>); 24]> = std::sync::Mutex::new([
                (7500, None), (7505, None), (7510, None), (7502, None), (7507, None), (8140, None), (8145, None), (7001, None), (2000, None), (2030, None), (7512, None), (8155, None),
                (8120, None), (8121, None), (8122, None), (8123, None), (8124, None), (8125, None),
                (8126, None), (8127, None), (8128, None), (8129, None), (8130, None), (8131, None),
            ]);
            let mut day = DAY.lock().unwrap();
            for (flag, last) in day.iter_mut() {
                if let Some(on) = found.get(*flag) {
                    if *last != Some(on) {
                        if DEBUG.load(Ordering::SeqCst) && LOG_DAYFLAGS.load(Ordering::SeqCst) {
                            log_line(&watch_dir, &format!("NRAP dayflag {flag} {}->{}", last.map(|v| if v {"1"} else {"0"}).unwrap_or("?"), if on {"1"} else {"0"}));
                        }
                        if last.is_some() && on && matches!(*flag, 7512 | 7001 | 2000) {
                            let mut latched = Vec::new();
                            for counter in [8140u32, 8145, 8155] {
                                if found.get(counter) == Some(true) {
                                    flag_write::note_return(counter);
                                    latched.push(counter);
                                }
                            }
                            log_line(&watch_dir, &format!("NRAP return signal {flag}, latching clears {latched:?}"));
                        }
                        if last.is_some() && on {
                            if let Some((n, loc)) = flag_write::note_day_boss(*flag) {
                                let _ = watch_tx.send(loc);
                                log_line(&watch_dir, &format!("NRAP day boss {flag} count {n} loc {loc}"));
                            }
                        }
                        if last.is_some() && matches!(*flag, 8140 | 8145 | 8155) {
                            if let Some((n, loc)) = flag_write::note_toggle(*flag, on) {
                                let _ = watch_tx.send(loc);
                                log_line(&watch_dir, &format!("NRAP toggle {flag} count {n} loc {loc}"));
                            } else if !on {
                                log_line(&watch_dir, &format!("NRAP toggle {flag} clear ignored"));
                            }
                        }
                        if last.is_some() && on && (8120..=8131).contains(flag) {
                            if let Some((n, loc)) = flag_write::note_toggle(*flag, true) {
                                let _ = watch_tx.send(loc);
                                log_line(&watch_dir, &format!("NRAP buried treasure {flag} count {n} loc {loc}"));
                            }
                        }
                        *last = Some(on);
                    }
                }
            }
            drop(day);
            for w in &mut watches {
                if flag_write::in_expedition() && w.location.starts_with("Shop - ") {
                    continue;
                }
                match found.get(w.flag) {
                    Some(on) => {
                        if w.last != Some(on) {
                            if w.last == Some(false) && on {
                                if let Some(gate) = flag_write::unlock_for_defeat(w.flag) {
                                    if found.get(gate) != Some(true) {
                                        w.ignored = true;
                                        log_line(&watch_dir, &format!("NRAP check blocked: {} (flag {} 0->1, unlock {} off)", w.location, w.flag, gate));
                                    }
                                }
                                if !w.ignored {
                                    log_line(
                                        &watch_dir,
                                        &format!("NRAP check: {} (flag {} 0->1)", w.location, w.flag),
                                    );
                                    if let Some(msg) = flag_write::note_defeat(w.flag) {
                                        log_line(&watch_dir, &msg);
                                    }
                                    if let Some(loc) = flag_write::note_count_goal(w.flag) {
                                        let _ = watch_tx.send(loc);
                                        log_line(&watch_dir, &format!("NRAP count goal met, LocationChecks {loc}"));
                                    }
                                }
                            } else {
                                log_line(
                                    &watch_dir,
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
                        if on && !w.submitted && !w.ignored && w.location_id != 0 {
                            if let Some(gate) = flag_write::unlock_for_defeat(w.flag) {
                                if found.get(gate) != Some(true) {
                                    w.ignored = true;
                                    log_line(&watch_dir, &format!("NRAP check blocked: {} (flag {} on, unlock {} off)", w.location, w.flag, gate));
                                }
                            }
                        }
                        if on && !w.submitted && !w.ignored && w.location_id != 0 && (w.flag != 6012 || crate::ap::tutorial_margit()) {
                            let _ = watch_tx.send(w.location_id);
                            w.submitted = true;
                        }
                    }
                    None => {
                        if !w.miss_logged {
                            log_line(
                                &watch_dir,
                                &format!("NRAP {} flag {} not in holder", w.location, w.flag),
                            );
                            w.miss_logged = true;
                        }
                    }
                }
            }
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
                        log_line(&watch_dir, &format!("NRAP flag diff +{} groups={groups} {}", rose.len(), show.join(",")));
                        *IDLE.lock().unwrap() = Some(std::time::Instant::now());
                    } else if IDLE.lock().unwrap().map(|t| t.elapsed().as_secs() >= 10).unwrap_or(true) {
                        *IDLE.lock().unwrap() = Some(std::time::Instant::now());
                        log_line(&watch_dir, &format!("NRAP flag diff idle groups={groups}"));
                    }
                }
            }
        }
            thread::sleep(Duration::from_millis(200));
        }
    });
    let sticky_dir = dir.clone();
    thread::spawn(move || loop {
        if flag_write::in_session() && !flag_write::in_expedition() {
            if let Some(msg) = flag_write::reapply_cached() {
                if DEBUG.load(Ordering::SeqCst) && LOG_STICKY.load(Ordering::SeqCst) {
                    log_line(&sticky_dir, &msg);
                }
            }
        }
        thread::sleep(Duration::from_millis(200));
    });
    loop {
        if !flag_write::in_session() {
            thread::sleep(Duration::from_millis(200));
            continue;
        }
        let hold = !flag_write::in_expedition();
        if hold {
        if let Some((index, id)) = flag_write::pop_queued() {
            let mut landed = false;
            if let Some(msg) = flag_write::apply_item(id) {
                log_line(&dir, &msg);
                landed = flag_write::grant_landed(&msg);
            }
            if let Some(msg) = hero::want(id) {
                log_line(&dir, &msg);
            }
            if let Some(msg) = grant::want_once(dir.as_ref(), &flag_write::cache_seed(), index, id) {
                log_line(&dir, &msg);
                if msg.contains("murk +") || msg.contains("skip already") {
                    landed = true;
                }
            }
            if landed {
                flag_write::mark_granted(index);
            }
        }
        if let Some(msg) = flag_write::retry_pending() {
            let rewrite = msg.contains("rewrite");
            if rewrite {
                if DEBUG.load(Ordering::SeqCst) && LOG_REWRITES.load(Ordering::SeqCst) {
                    log_line(&dir, &msg);
                }
            } else if DEBUG.load(Ordering::SeqCst) {
                log_line(&dir, &msg);
            }
        }
        if let Some(msg) = flag_write::stock_shop_rows() {
            log_line(&dir, &msg);
        }
        hero::arm(&flag_write::cached_flags());
        if let Some(msg) = hero::apply() {
            log_line(&dir, &msg);
        }
        }
        if hold {
            if let Some(msg) = grant::retry() {
                log_line(&dir, &msg);
            }
        }
        thread::sleep(Duration::from_millis(200));
    }
}
