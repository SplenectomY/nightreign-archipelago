//! Expedition menu probe. The 5-byte jmp can only reach +/-2GB, so the cave
//! must be allocated beside the game module.

#![cfg(windows)]

use crate::aob::{self, ModuleSpan};
use std::ffi::c_void;
use std::sync::atomic::{AtomicU32, Ordering};
use std::sync::Mutex;

const MENU_AOB: &str = "FF 50 18 49 8B CE 45 84 FF 74 07";
const PATCH_AT: usize = 6;
const PAGE_EXECUTE_READWRITE: u32 = 0x40;
const MEM_COMMIT: u32 = 0x1000;
const MEM_RESERVE: u32 = 0x2000;
const MEM_RELEASE: u32 = 0x8000;

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
static HITS: AtomicU32 = AtomicU32::new(0);
static LOGGED_HITS: AtomicU32 = AtomicU32::new(0);

#[link(name = "kernel32")]
extern "system" {
    fn VirtualAlloc(addr: *mut c_void, size: usize, kind: u32, protect: u32) -> *mut c_void;
    fn VirtualFree(addr: *mut c_void, size: usize, kind: u32) -> i32;
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
    let hits = HITS.load(Ordering::Relaxed);
    let prev = LOGGED_HITS.swap(hits, Ordering::Relaxed);
    let mut out = LOGS.lock().map(|mut q| std::mem::take(&mut *q)).unwrap_or_default();
    if hits != prev {
        out.push(format!("NRAP menu probe hits={hits}"));
    }
    out
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

fn alloc_near(site: usize) -> *mut c_void {
    let page = site & !0xFFFF;
    for i in 1..2048 {
        for sign in [-1isize, 1] {
            let hint = page.wrapping_add((i * 0x10000).wrapping_mul(sign as usize));
            let p = unsafe {
                VirtualAlloc(hint as *mut c_void, 0x1000, MEM_COMMIT | MEM_RESERVE, PAGE_EXECUTE_READWRITE)
            };
            if p.is_null() {
                continue;
            }
            let dist = (p as isize).wrapping_sub(site as isize);
            if dist.abs() < 0x7000_0000 {
                return p;
            }
            unsafe { VirtualFree(p, 0, MEM_RELEASE); }
        }
    }
    std::ptr::null_mut()
}

fn write_jmp(at: usize, to: usize) -> bool {
    let rel = to.wrapping_sub(at.wrapping_add(5)) as isize;
    if rel < i32::MIN as isize || rel > i32::MAX as isize {
        return false;
    }
    let rel = rel as i32;
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
    let cave_ptr = alloc_near(site);
    if cave_ptr.is_null() {
        return Err("menu cave not allocated within jmp range".into());
    }
    let cave = cave_ptr as usize;
    let hits = &HITS as *const AtomicU32 as usize;
    let mut code = Vec::new();
    code.push(0x50);
    code.extend_from_slice(&[0x48, 0xB8]);
    code.extend_from_slice(&hits.to_le_bytes());
    code.extend_from_slice(&[0xF0, 0xFF, 0x00, 0x58]);
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
        return Err(format!("menu jmp out of range site=0x{site:X} cave=0x{cave:X}"));
    }
    Ok(format!("NRAP menu probe site=0x{site:X} cave=0x{cave:X}"))
}
