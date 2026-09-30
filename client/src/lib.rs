//! Phase 0 client. Attach, resolve event flags, log 0->1 edges.

#![cfg(windows)]

mod aob;
mod flagman;

use std::ffi::c_void;
use std::fs::{self, OpenOptions};
use std::io::Write;
use std::path::PathBuf;
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
    write_console(msg);
    if let Some(dir) = dir {
        if let Ok(mut f) = OpenOptions::new()
            .create(true)
            .append(true)
            .open(dir.join("nrap.log"))
        {
            let _ = writeln!(f, "{msg}");
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
    flag: u32,
    last: Option<bool>,
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
    let mut flag = None::<u32>;
    let flush = |location: &mut Option<String>, flag: &mut Option<u32>, out: &mut Vec<Watch>| {
        if let (Some(loc), Some(id)) = (location.take(), flag.take()) {
            if id != 0 {
                out.push(Watch {
                    location: loc,
                    flag: id,
                    last: None,
                });
            }
        }
    };
    for line in text.lines() {
        if line.trim().starts_with("[[flag]]") {
            flush(&mut location, &mut flag, &mut out);
            continue;
        }
        let Some((k, v)) = toml_key_value(line) else {
            continue;
        };
        if k == "location" {
            location = Some(v.to_string());
        }
        if k == "event_flag_id" {
            flag = v.parse().ok();
        }
    }
    flush(&mut location, &mut flag, &mut out);
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

fn worker() {
    let dir = dll_dir();
    bind_console();
    log_line(&dir, "NRAP attached");
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

    let mut watches = config
        .as_ref()
        .and_then(|p| fs::read_to_string(p).ok())
        .map(|t| parse_watches(&t))
        .unwrap_or_default();
    if watches.is_empty() {
        log_line(&dir, "NRAP no event_flag_id values in flags.toml");
    } else {
        for w in &watches {
            log_line(
                &dir,
                &format!("NRAP watching {} flag {}", w.location, w.flag),
            );
        }
    }

    let mut man = None;
    let mut fail_logged = false;
    loop {
        if man.is_none() {
            match flagman::resolve() {
                Ok(found) => {
                    log_line(
                        &dir,
                        &format!(
                            "NRAP flagman pattern={} layout={} slot=0x{:X} inst=0x{:X} bits=0x{:X}",
                            found.pattern,
                            found.layout,
                            found.singleton_slot,
                            found.instance,
                            found.bits
                        ),
                    );
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
            for w in &mut watches {
                match found.get(w.flag) {
                    Some(on) => {
                        if w.last != Some(on) {
                            if w.last == Some(false) && on {
                                log_line(
                                    &dir,
                                    &format!("NRAP check: {} (flag {} 0->1)", w.location, w.flag),
                                );
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
                    }
                    None => {
                        log_line(&dir, "NRAP flag read failed; rescanning flagman");
                        man = None;
                        fail_logged = false;
                        break;
                    }
                }
            }
        }
        thread::sleep(Duration::from_millis(500));
    }
}
