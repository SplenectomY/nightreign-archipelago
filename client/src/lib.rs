//! Phase 0 client. Attach, open a console, poll data/flags.toml.
//! Event-flag resolution and the Archipelago socket land once testers
//! return a real flag ID.

#![cfg(windows)]

use std::ffi::c_void;
use std::fs;
use std::path::PathBuf;
use std::thread;
use std::time::Duration;

type BOOL = i32;
type DWORD = u32;
type HMODULE = *mut c_void;
type LPVOID = *mut c_void;

const DLL_PROCESS_ATTACH: DWORD = 1;

#[link(name = "kernel32")]
extern "system" {
    fn AllocConsole() -> BOOL;
    fn GetModuleHandleA(name: *const u8) -> HMODULE;
    fn GetModuleFileNameA(module: HMODULE, buf: *mut u8, size: DWORD) -> DWORD;
}

#[no_mangle]
pub extern "system" fn DllMain(
    _module: HMODULE,
    reason: DWORD,
    _reserved: LPVOID,
) -> BOOL {
    if reason == DLL_PROCESS_ATTACH {
        thread::spawn(worker);
    }
    1
}

fn worker() {
    unsafe {
        AllocConsole();
    }
    println!("NRAP attached");
    let base = unsafe { GetModuleHandleA(std::ptr::null()) };
    println!("NRAP nightreign.exe base = {:p}", base);

    let config = find_config();
    println!(
        "NRAP config = {}",
        config
            .as_ref()
            .map(|p| p.display().to_string())
            .unwrap_or_else(|| "<missing flags.toml>".into())
    );

    let mut announced_watch = false;
    loop {
        if let Some(path) = config.as_ref() {
            if let Ok(text) = fs::read_to_string(path) {
                if !announced_watch {
                    if let Some(id) = parse_gladius_flag_id(&text) {
                        if id == 0 {
                            println!("NRAP watching Nightlord - Gladius; event_flag_id is still 0 (not discovered)");
                        } else {
                            println!("NRAP watching Nightlord - Gladius; event_flag_id={id} (memory read not wired yet)");
                        }
                        announced_watch = true;
                    }
                }
            }
        }
        thread::sleep(Duration::from_secs(2));
    }
}

fn find_config() -> Option<PathBuf> {
    let mut buf = vec![0u8; 260];
    let n = unsafe { GetModuleFileNameA(std::ptr::null_mut(), buf.as_mut_ptr(), buf.len() as DWORD) };
    if n > 0 {
        buf.truncate(n as usize);
        if let Ok(dll) = String::from_utf8(buf) {
            let dir = PathBuf::from(dll).parent()?.to_path_buf();
            let local = dir.join("flags.toml");
            if local.exists() {
                return Some(local);
            }
            let sibling = dir
                .ancestors()
                .nth(3)
                .map(|root| root.join("data").join("flags.toml"));
            if let Some(path) = sibling {
                if path.exists() {
                    return Some(path);
                }
            }
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
