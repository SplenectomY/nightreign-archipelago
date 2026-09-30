//! Phase 0 client. Attach, write nrap.log, poll flags.toml.
//! Event-flag resolution and the Archipelago socket land once testers
//! return a real flag ID.

#![cfg(windows)]

use std::ffi::c_void;
use std::fs::{self, OpenOptions};
use std::io::Write;
use std::path::PathBuf;
use std::sync::OnceLock;
use std::thread;
use std::time::Duration;

type BOOL = i32;
type DWORD = u32;
type HMODULE = *mut c_void;
type LPVOID = *mut c_void;

const DLL_PROCESS_ATTACH: DWORD = 1;

static DLL_MODULE: OnceLock<usize> = OnceLock::new();

#[link(name = "kernel32")]
extern "system" {
    fn AllocConsole() -> BOOL;
    fn GetModuleHandleA(name: *const u8) -> HMODULE;
    fn GetModuleFileNameA(module: HMODULE, buf: *mut u8, size: DWORD) -> DWORD;
}

#[no_mangle]
pub extern "system" fn DllMain(
    module: HMODULE,
    reason: DWORD,
    _reserved: LPVOID,
) -> BOOL {
    if reason == DLL_PROCESS_ATTACH {
        let _ = DLL_MODULE.set(module as usize);
        thread::spawn(worker);
    }
    1
}

fn log_line(dir: &Option<PathBuf>, msg: &str) {
    println!("{msg}");
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
    let mut buf = vec![0u8; 260];
    let n = unsafe { GetModuleFileNameA(module, buf.as_mut_ptr(), buf.len() as DWORD) };
    if n == 0 {
        return None;
    }
    buf.truncate(n as usize);
    let path = String::from_utf8(buf).ok()?;
    PathBuf::from(path).parent().map(|p| p.to_path_buf())
}

fn worker() {
    let dir = dll_dir();
    unsafe {
        AllocConsole();
    }
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

    let mut announced_watch = false;
    loop {
        if let Some(path) = config.as_ref() {
            if let Ok(text) = fs::read_to_string(path) {
                if !announced_watch {
                    if let Some(id) = parse_gladius_flag_id(&text) {
                        if id == 0 {
                            log_line(
                                &dir,
                                "NRAP watching Nightlord - Gladius; event_flag_id is still 0 (not discovered)",
                            );
                        } else {
                            log_line(
                                &dir,
                                &format!("NRAP watching Nightlord - Gladius; event_flag_id={id} (memory read not wired yet)"),
                            );
                        }
                        announced_watch = true;
                    }
                }
            }
        }
        thread::sleep(Duration::from_secs(2));
    }
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

fn parse_gladius_flag_id(text: &str) -> Option<u32> {
    let mut in_gladius = false;
    for line in text.lines() {
        let line = line.trim();
        if line.starts_with("location") && line.contains("Nightlord - Gladius") {
            in_gladius = true;
        }
        if in_gladius && line.starts_with("event_flag_id") {
            let value = line.split('=').nth(1)?.trim();
            return value.parse().ok();
        }
        if line.starts_with("[[flag]]") {
            in_gladius = false;
        }
    }
    None
}
