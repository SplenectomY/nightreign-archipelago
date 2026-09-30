//! Find the Hold Murk object without patching. Add currency at +0xD0.

#![cfg(windows)]

use crate::aob::{self, ModuleSpan};
use std::fs;
use std::path::PathBuf;
use std::sync::atomic::{AtomicI32, AtomicUsize, Ordering};
use std::sync::Mutex;

const GETTER: &str = "8B 81 D0 00 00 00 C3";
const MOV_RCX_SLOT: &str = "48 8B 0D ?? ?? ?? ??";
const MURK_OFF: usize = 0xD0;
const MURK_BUNDLE_ID: i64 = 839_100_100;
const DEFAULT_BUNDLE: i32 = 1000;

static MURK_SLOT: AtomicUsize = AtomicUsize::new(0);
static MURK_OBJ: AtomicUsize = AtomicUsize::new(0);
static BUNDLE_AMOUNT: AtomicI32 = AtomicI32::new(DEFAULT_BUNDLE);
static PENDING_MURK: AtomicI32 = AtomicI32::new(0);
static NEXT_GRANT: AtomicUsize = AtomicUsize::new(0);
static STATE_PATH: Mutex<Option<PathBuf>> = Mutex::new(None);

fn load_next(path: &PathBuf) -> usize {
    fs::read_to_string(path)
        .ok()
        .and_then(|s| s.trim().parse().ok())
        .unwrap_or(0)
}

fn save_next(path: &PathBuf, n: usize) {
    let _ = fs::write(path, n.to_string());
}

fn readable(ptr: usize, len: usize) -> bool {
    if ptr < 0x10000 {
        return false;
    }
    unsafe {
        let mut info = std::mem::zeroed::<[u8; 48]>();
        extern "system" {
            fn VirtualQuery(addr: *const u8, info: *mut u8, len: usize) -> usize;
        }
        if VirtualQuery(ptr as *const u8, info.as_mut_ptr(), 48) == 0 {
            return false;
        }
        let state = u32::from_le_bytes(info[32..36].try_into().unwrap_or([0; 4]));
        let protect = u32::from_le_bytes(info[36..40].try_into().unwrap_or([0; 4]));
        state == 0x1000 && protect & 0x101 == 0
    }
}

fn read_usize(ptr: usize) -> Option<usize> {
    if !readable(ptr, 8) {
        return None;
    }
    Some(unsafe { std::ptr::read_unaligned(ptr as *const usize) })
}

fn resolve_slot(span: ModuleSpan) -> Result<(usize, usize), String> {
    let hay = span.slice();
    let getter_rel =
        aob::find_pattern(hay, GETTER).ok_or_else(|| "Murk getter AOB not found".to_string())?;
    let getter = span.base + getter_rel;

    // Callers: E8 rel32 targeting getter. Preceding mov rcx, [rip+slot].
    let mut from = 0usize;
    while from + 5 < hay.len() {
        let Some(rel) = hay[from..].iter().position(|&b| b == 0xE8) else {
            break;
        };
        let at = from + rel;
        from = at + 1;
        if at + 5 > hay.len() {
            break;
        }
        let disp = i32::from_le_bytes(hay[at + 1..at + 5].try_into().unwrap());
        let target = (span.base + at + 5).wrapping_add(disp as usize);
        if target != getter {
            continue;
        }
        let back = if at >= 16 { at - 16 } else { 0 };
        if let Some(mrel) = aob::find_pattern(&hay[back..at + 5], MOV_RCX_SLOT) {
            let instr = back + mrel;
            if let Some(slot) = aob::rip_rel(span, instr, 3, 7) {
                return Ok((getter, slot));
            }
        }
    }
    Err(format!("Murk getter=0x{getter:X} but no rcx slot xref"))
}

fn refresh_obj() {
    let slot = MURK_SLOT.load(Ordering::SeqCst);
    if slot == 0 {
        return;
    }
    if let Some(obj) = read_usize(slot) {
        if obj > 0x10000 {
            MURK_OBJ.store(obj, Ordering::SeqCst);
        }
    }
}

fn add_murk(amount: i32) -> Result<i32, String> {
    refresh_obj();
    let obj = MURK_OBJ.load(Ordering::SeqCst);
    if obj == 0 {
        return Err("Murk object not live yet".into());
    }
    let p = obj + MURK_OFF;
    if !readable(p, 4) {
        return Err(format!("Murk +0xD0 not readable (obj=0x{obj:X})"));
    }
    unsafe {
        let old = std::ptr::read_unaligned(p as *const i32);
        let new = old.saturating_add(amount);
        std::ptr::write_unaligned(p as *mut i32, new);
        Ok(new)
    }
}

pub fn init(dir: Option<&PathBuf>, bundle_amount: i32) -> Result<String, String> {
    BUNDLE_AMOUNT.store(bundle_amount.max(1), Ordering::SeqCst);
    if let Some(dir) = dir {
        let path = dir.join("granted_index.txt");
        NEXT_GRANT.store(load_next(&path), Ordering::SeqCst);
        *STATE_PATH.lock().unwrap() = Some(path);
    }
    let span = ModuleSpan::nightreign().ok_or_else(|| "no nightreign module".to_string())?;
    let (getter, slot) = resolve_slot(span)?;
    MURK_SLOT.store(slot, Ordering::SeqCst);
    refresh_obj();
    Ok(format!(
        "NRAP murk getter=0x{getter:X} slot=0x{slot:X} obj=0x{:X} bundle={}",
        MURK_OBJ.load(Ordering::SeqCst),
        BUNDLE_AMOUNT.load(Ordering::SeqCst)
    ))
}

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
        Err(e) => {
            PENDING_MURK.fetch_add(amt, Ordering::SeqCst);
            Some(format!("NRAP grant queued +{amt} Murk ({e})"))
        }
    }
}

pub fn retry_pending() -> Option<String> {
    let amt = PENDING_MURK.load(Ordering::SeqCst);
    if amt <= 0 {
        return None;
    }
    match add_murk(amt) {
        Ok(new) => {
            PENDING_MURK.store(0, Ordering::SeqCst);
            Some(format!("NRAP granted queued Murk +{amt} wallet={new}"))
        }
        Err(_) => None,
    }
}
