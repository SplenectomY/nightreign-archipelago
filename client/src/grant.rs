//! Capture the Hold Murk object via the game getter, then add currency.

#![cfg(windows)]

use crate::aob::{self, ModuleSpan};
use std::fs;
use std::path::PathBuf;
use std::sync::atomic::{AtomicI32, AtomicUsize, Ordering};
use std::sync::Mutex;

const GETTER: &str = "8B 81 D0 00 00 00 C3 48 8D 64";
const MURK_OFF: usize = 0xD0;
const MURK_BUNDLE_ID: i64 = 839_100_100;
const DEFAULT_BUNDLE: i32 = 1000;

static MURK_OBJ: AtomicUsize = AtomicUsize::new(0);
static BUNDLE_AMOUNT: AtomicI32 = AtomicI32::new(DEFAULT_BUNDLE);
static NEXT_GRANT: AtomicUsize = AtomicUsize::new(0);
static STATE_PATH: Mutex<Option<PathBuf>> = Mutex::new(None);

#[link(name = "kernel32")]
extern "system" {
    fn VirtualAlloc(addr: *mut u8, size: usize, ty: u32, prot: u32) -> *mut u8;
    fn VirtualProtect(addr: *mut u8, size: usize, prot: u32, old: *mut u32) -> i32;
}

const MEM_COMMIT: u32 = 0x1000;
const MEM_RESERVE: u32 = 0x2000;
const PAGE_EXECUTE_READWRITE: u32 = 0x40;

/// Hook site writes the getter's rcx (Murk object) here, then runs the original load.
#[repr(C)]
struct HookPayload {
    capture: usize,
}

fn write_rel32(buf: &mut [u8], at: usize, target: usize, after: usize) {
    let rel = target.wrapping_sub(after) as i32;
    buf[at..at + 4].copy_from_slice(&rel.to_le_bytes());
}

fn install_hook(span: ModuleSpan) -> Result<usize, String> {
    let hay = span.slice();
    let rel = aob::find_pattern(hay, GETTER).ok_or_else(|| "Murk getter AOB not found".to_string())?;
    let src = span.base + rel;
    unsafe {
        let tramp = VirtualAlloc(
            std::ptr::null_mut(),
            64,
            MEM_COMMIT | MEM_RESERVE,
            PAGE_EXECUTE_READWRITE,
        );
        if tramp.is_null() {
            return Err("VirtualAlloc trampoline failed".into());
        }
        // trampoline:
        // 48 89 0D disp32    mov [rip+disp], rcx   ; capture
        // 8B 81 D0 00 00 00  mov eax, [rcx+0xD0]
        // C3                 ret
        let capture = tramp as usize + 32;
        let mut code = [0u8; 16];
        code[0] = 0x48;
        code[1] = 0x89;
        code[2] = 0x0D;
        write_rel32(&mut code, 3, capture, tramp as usize + 7);
        code[7] = 0x8B;
        code[8] = 0x81;
        code[9] = 0xD0;
        code[10] = 0x00;
        code[11] = 0x00;
        code[12] = 0x00;
        code[13] = 0xC3;
        std::ptr::copy_nonoverlapping(code.as_ptr(), tramp, 16);

        let mut old = 0u32;
        if VirtualProtect(src as *mut u8, 8, PAGE_EXECUTE_READWRITE, &mut old) == 0 {
            return Err("VirtualProtect getter failed".into());
        }
        // E9 rel32 ; nop nop
        let mut jmp = [0xE9u8, 0, 0, 0, 0, 0x90, 0x90];
        let rel32 = (tramp as usize).wrapping_sub(src + 5) as i32;
        jmp[1..5].copy_from_slice(&rel32.to_le_bytes());
        std::ptr::copy_nonoverlapping(jmp.as_ptr(), src as *mut u8, 7);
        let _ = capture;
        // Store capture address so the worker can read it.
        CAPTURE_SLOT.store(capture, Ordering::SeqCst);
        Ok(src)
    }
}

static CAPTURE_SLOT: AtomicUsize = AtomicUsize::new(0);

fn refresh_obj() {
    let slot = CAPTURE_SLOT.load(Ordering::SeqCst);
    if slot == 0 {
        return;
    }
    let obj = unsafe { std::ptr::read_unaligned(slot as *const usize) };
    if obj > 0x10000 {
        MURK_OBJ.store(obj, Ordering::SeqCst);
    }
}

fn add_murk(amount: i32) -> Result<i32, String> {
    refresh_obj();
    let obj = MURK_OBJ.load(Ordering::SeqCst);
    if obj == 0 {
        return Err("Murk object not captured yet (open the Hold HUD / bazaar)".into());
    }
    unsafe {
        let p = (obj + MURK_OFF) as *mut i32;
        let old = std::ptr::read_unaligned(p);
        let new = old.saturating_add(amount);
        std::ptr::write_unaligned(p, new);
        Ok(new)
    }
}

fn load_next(path: &PathBuf) -> usize {
    fs::read_to_string(path)
        .ok()
        .and_then(|s| s.trim().parse().ok())
        .unwrap_or(0)
}

fn save_next(path: &PathBuf, n: usize) {
    let _ = fs::write(path, n.to_string());
}

pub fn init(dir: Option<&PathBuf>, bundle_amount: i32) -> Result<String, String> {
    BUNDLE_AMOUNT.store(bundle_amount.max(1), Ordering::SeqCst);
    if let Some(dir) = dir {
        let path = dir.join("granted_index.txt");
        NEXT_GRANT.store(load_next(&path), Ordering::SeqCst);
        *STATE_PATH.lock().unwrap() = Some(path);
    }
    let span = ModuleSpan::nightreign().ok_or_else(|| "no nightreign module".to_string())?;
    let src = install_hook(span)?;
    Ok(format!(
        "NRAP murk hook getter=0x{src:X} bundle={}",
        BUNDLE_AMOUNT.load(Ordering::SeqCst)
    ))
}

/// Apply one ReceivedItems entry. `index` is the AP inventory index.
pub fn apply_received(item_id: i64, index: i64) -> Option<String> {
    let next = NEXT_GRANT.load(Ordering::SeqCst) as i64;
    if index < next {
        return None;
    }
    NEXT_GRANT.store((index + 1) as usize, Ordering::SeqCst);
    if let Some(path) = STATE_PATH.lock().unwrap().as_ref() {
        save_next(path, (index + 1) as usize);
    }
    if item_id != MURK_BUNDLE_ID {
        return Some(format!("NRAP grant skipped {item_id} (no handler)"));
    }
    let amt = BUNDLE_AMOUNT.load(Ordering::SeqCst);
    match add_murk(amt) {
        Ok(new) => Some(format!("NRAP granted Murk +{amt} wallet={new}")),
        Err(e) => Some(format!("NRAP grant queued, {e}")),
    }
}

pub fn retry_pending() -> Option<String> {
    refresh_obj();
    if MURK_OBJ.load(Ordering::SeqCst) == 0 {
        return None;
    }
    None
}
