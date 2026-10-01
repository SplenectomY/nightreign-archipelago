//! Expedition menu gate. Flag 110 opens every secondary Nightlord, so it is not used.
//! The menu check at `test r15b, r15b / jz` is detoured and r15b is replaced per boss id.

#![cfg(windows)]

use crate::aob::{self, ModuleSpan};
use std::ffi::c_void;
use std::sync::Mutex;

const MENU_AOB: &str = "FF 50 18 49 8B CE 45 84 FF 74 07";
const PATCH_AT: usize = 6;
const PAGE_EXECUTE_READWRITE: u32 = 0x40;
const MEM_COMMIT: u32 = 0x1000;
const MEM_RESERVE: u32 = 0x2000;

const ADEL: u32 = 1 << 0;
const GNOSTER: u32 = 1 << 1;
const MARIS: u32 = 1 << 2;
const LIBRA: u32 = 1 << 3;
const FULGHOR: u32 = 1 << 4;
const CALIGO: u32 = 1 << 5;
const HEOLSTOR: u32 = 1 << 6;
const HARMONIA: u32 = 1 << 7;
const STRAGHESS: u32 = 1 << 8;

static OWNED: Mutex<u32> = Mutex::new(0);
static LOGS: Mutex<Vec<String>> = Mutex::new(Vec::new());
static SEEN: Mutex<Vec<u32>> = Mutex::new(Vec::new());

#[link(name = "kernel32")]
extern "system" {
    fn VirtualAlloc(addr: *mut c_void, size: usize, kind: u32, protect: u32) -> *mut c_void;
    fn VirtualProtect(addr: *mut c_void, size: usize, protect: u32, old: *mut u32) -> i32;
}

fn log(msg: impl Into<String>) {
    if let Ok(mut q) = LOGS.lock() {
        if q.len() < 40 {
            q.push(msg.into());
        }
    }
}

pub fn drain_logs() -> Vec<String> {
    LOGS.lock().map(|mut q| std::mem::take(&mut *q)).unwrap_or_default()
}

pub fn grant(item_id: i64) {
    let bit = match item_id {
        839_100_002 => ADEL,
        839_100_003 => GNOSTER,
        839_100_004 => MARIS,
        839_100_005 => LIBRA,
        839_100_006 => FULGHOR,
        839_100_007 => CALIGO,
        839_100_008 => HEOLSTOR,
        839_100_009 => HARMONIA,
        839_100_010 => STRAGHESS,
        _ => return,
    };
    if let Ok(mut owned) = OWNED.lock() {
        *owned |= bit;
        log(format!("NRAP menu owned bit=0x{bit:X} mask=0x{:X}", *owned));
    }
}

fn read_u32(ptr: usize) -> Option<u32> {
    if ptr < 0x10000 {
        return None;
    }
    Some(unsafe { std::ptr::read_unaligned(ptr as *const u32) })
}

fn boss_id(entry: usize) -> Option<u32> {
    const IDS: &[u32] = &[2, 12, 22, 23, 32, 43, 53, 61, 73, 1080, 1090];
    if entry < 0x10000 {
        return None;
    }
    for off in (0..0x180).step_by(4) {
        let v = read_u32(entry + off)?;
        if IDS.contains(&v) {
            return Some(v);
        }
        if off % 8 == 0 {
            if let Some(p) = read_u32(entry + off).and_then(|_| {
                let ptr = unsafe { std::ptr::read_unaligned((entry + off) as *const usize) };
                if ptr > 0x10000 {
                    read_u32(ptr)
                } else {
                    None
                }
            }) {
                if IDS.contains(&p) {
                    return Some(p);
                }
            }
        }
    }
    None
}

fn allowed(id: u32) -> bool {
    let owned = OWNED.lock().map(|g| *g).unwrap_or(0);
    match id {
        2 => true,
        12 => owned & ADEL != 0,
        22 | 23 => owned & GNOSTER != 0,
        32 => owned & MARIS != 0,
        43 => owned & LIBRA != 0,
        53 => owned & FULGHOR != 0,
        61 => owned & CALIGO != 0,
        73 => owned & HEOLSTOR != 0,
        1080 => owned & HARMONIA != 0,
        1090 => owned & STRAGHESS != 0,
        _ => false,
    }
}

/// Called from the trampoline. r14 is the menu entry, vanilla is the game boolean.
pub extern "C" fn decide(entry: usize, vanilla: u32) -> u32 {
    let Some(id) = boss_id(entry) else {
        if let Ok(mut seen) = SEEN.lock() {
            let mark = read_u32(entry).unwrap_or(0);
            if seen.len() < 8 && !seen.contains(&mark) {
                seen.push(mark);
                log(format!(
                    "NRAP menu unknown entry=0x{entry:X} vanilla={vanilla} head={mark}"
                ));
            }
        }
        return vanilla;
    };
    if let Ok(mut seen) = SEEN.lock() {
        if !seen.contains(&id) {
            seen.push(id);
            log(format!("NRAP menu boss id={id} vanilla={vanilla} entry=0x{entry:X}"));
        }
    }
    u32::from(allowed(id))
}

fn write_jmp(at: usize, to: usize) -> bool {
    let rel = to.wrapping_sub(at.wrapping_add(5)) as i32;
    let bytes = [
        0xE9,
        rel as u8,
        (rel >> 8) as u8,
        (rel >> 16) as u8,
        (rel >> 24) as u8,
    ];
    unsafe {
        let mut old = 0u32;
        if VirtualProtect(at as *mut c_void, 5, PAGE_EXECUTE_READWRITE, &mut old) == 0 {
            return false;
        }
        std::ptr::copy_nonoverlapping(bytes.as_ptr(), at as *mut u8, 5);
        VirtualProtect(at as *mut c_void, 5, old, &mut old);
    }
    true
}

pub fn init() -> Result<String, String> {
    let span = ModuleSpan::nightreign().ok_or_else(|| "no nightreign module".to_string())?;
    let rel = aob::find_pattern(span.slice(), MENU_AOB).ok_or_else(|| "menu AOB not found".to_string())?;
    let site = span.base + rel + PATCH_AT;
    let fallthrough = site + 5;
    let taken = site + 12;
    let cave = unsafe { VirtualAlloc(std::ptr::null_mut(), 0x100, MEM_COMMIT | MEM_RESERVE, PAGE_EXECUTE_READWRITE) };
    if cave.is_null() {
        return Err("menu cave alloc failed".into());
    }
    let cave = cave as usize;
    let decide_addr = decide as usize;
    let mut code = Vec::new();
    code.extend_from_slice(&[0x50, 0x51, 0x52, 0x41, 0x50, 0x41, 0x51, 0x41, 0x52, 0x41, 0x53]);
    code.extend_from_slice(&[0x48, 0x83, 0xEC, 0x28]);
    code.extend_from_slice(&[0x4C, 0x89, 0xF1]);
    code.extend_from_slice(&[0x41, 0x0F, 0xB6, 0xD7]);
    code.extend_from_slice(&[0x48, 0xB8]);
    code.extend_from_slice(&decide_addr.to_le_bytes());
    code.extend_from_slice(&[0xFF, 0xD0, 0x41, 0x88, 0xC7]);
    code.extend_from_slice(&[0x48, 0x83, 0xC4, 0x28]);
    code.extend_from_slice(&[0x41, 0x5B, 0x41, 0x5A, 0x41, 0x59, 0x41, 0x58, 0x5A, 0x59, 0x58]);
    code.extend_from_slice(&[0x45, 0x84, 0xFF, 0x74, 0x05]);
    let jz_from = cave + code.len();
    code.extend_from_slice(&[0xE9, 0, 0, 0, 0]);
    let taken_from = cave + code.len();
    code.extend_from_slice(&[0xE9, 0, 0, 0, 0]);
    let rel_fall = fallthrough.wrapping_sub(jz_from.wrapping_add(5)) as i32;
    let rel_taken = taken.wrapping_sub(taken_from.wrapping_add(5)) as i32;
    code[jz_from - cave + 1..jz_from - cave + 5].copy_from_slice(&rel_fall.to_le_bytes());
    code[taken_from - cave + 1..taken_from - cave + 5].copy_from_slice(&rel_taken.to_le_bytes());
    unsafe {
        std::ptr::copy_nonoverlapping(code.as_ptr(), cave as *mut u8, code.len());
    }
    if !write_jmp(site, cave) {
        return Err("menu jmp protect failed".into());
    }
    Ok(format!("NRAP menu gate site=0x{site:X} cave=0x{cave:X}"))
}
