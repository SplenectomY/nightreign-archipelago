//! Session Nightfarer id. The hold getter reads one byte at [object+0x5A0]+0xC7.
//! Wylder is 1, Duchess is 4: HeroParam row + 1. The model reloads at the bell.

#![cfg(windows)]

use crate::aob::{self, ModuleSpan};
use std::ffi::c_void;
use std::sync::atomic::{AtomicU32, AtomicUsize, Ordering};

const GETTER: &str = "48 8B 81 A0 05 00 00 48 85 C0 74 ?? 0F B6 80 C7 00 00 00 C3";
const PAGE_EXECUTE_READWRITE: u32 = 0x40;

#[link(name = "kernel32")]
extern "system" {
    fn VirtualAlloc(addr: *mut c_void, size: usize, typ: u32, prot: u32) -> *mut c_void;
    fn VirtualProtect(addr: *mut c_void, size: usize, prot: u32, old: *mut u32) -> i32;
}

static OBJECT: AtomicUsize = AtomicUsize::new(0);
static WANTED: AtomicU32 = AtomicU32::new(0);
static GETTER_ADDR: AtomicUsize = AtomicUsize::new(0);

pub fn session_id(item_id: i64) -> Option<u8> {
    Some(match item_id {
        839_100_301 => 1, // Wylder
        839_100_307 => 2, // Guardian
        839_100_306 => 3, // Ironeye
        839_100_302 => 4, // Duchess
        839_100_303 => 5, // Raider
        839_100_308 => 6, // Revenant
        839_100_305 => 7, // Recluse
        839_100_304 => 8, // Executor
        839_100_309 => 9, // Scholar
        839_100_310 => 10, // Undertaker
        _ => return None,
    })
}

fn write_byte(addr: usize, value: u8) -> bool {
    if addr < 0x10000 {
        return false;
    }
    unsafe {
        let mut old = 0u32;
        if VirtualProtect(addr as *mut c_void, 1, PAGE_EXECUTE_READWRITE, &mut old) == 0 {
            return false;
        }
        std::ptr::write_unaligned(addr as *mut u8, value);
        VirtualProtect(addr as *mut c_void, 1, old, &mut old);
    }
    true
}

fn slot_addr(object: usize) -> Option<usize> {
    if object < 0x10000 {
        return None;
    }
    let inner = unsafe { std::ptr::read_unaligned((object + 0x5A0) as *const usize) };
    if inner < 0x10000 {
        return None;
    }
    Some(inner + 0xC7)
}

pub fn init() -> Result<String, String> {
    let span = ModuleSpan::nightreign().ok_or_else(|| "no module".to_string())?;
    let rel = aob::find_pattern(span.slice(), GETTER).ok_or_else(|| "hero getter AOB not found".to_string())?;
    let site = span.base + rel;
    let cave = unsafe {
        VirtualAlloc(std::ptr::null_mut(), 64, 0x1000 | 0x2000, PAGE_EXECUTE_READWRITE)
    };
    if cave.is_null() {
        return Err("hero cave alloc failed".into());
    }
    let saved = Box::into_raw(Box::new(0usize)) as usize;
    // mov [rip+disp], rcx; then the original getter body; ret
    let mut code = Vec::new();
    code.extend_from_slice(&[0x48, 0x89, 0x0D]);
    let disp = saved.wrapping_sub(cave as usize + 7) as i32;
    code.extend_from_slice(&disp.to_le_bytes());
    code.extend_from_slice(&[
        0x48, 0x8B, 0x81, 0xA0, 0x05, 0x00, 0x00, // mov rax, [rcx+5A0]
        0x48, 0x85, 0xC0, // test rax, rax
        0x74, 0x08, // jz ret0
        0x0F, 0xB6, 0x80, 0xC7, 0x00, 0x00, 0x00, // movzx eax, byte [rax+C7]
        0xC3, // ret
        0x31, 0xC0, // xor eax, eax
        0xC3,
    ]);
    unsafe {
        std::ptr::copy_nonoverlapping(code.as_ptr(), cave as *mut u8, code.len());
        let mut old = 0u32;
        if VirtualProtect(site as *mut c_void, 19, PAGE_EXECUTE_READWRITE, &mut old) == 0 {
            return Err("hero getter protect failed".into());
        }
        let mut patch = [0x90u8; 19];
        patch[0] = 0xFF;
        patch[1] = 0x25;
        patch[2] = 0;
        patch[3] = 0;
        patch[4] = 0;
        patch[5] = 0;
        patch[6..14].copy_from_slice(&(cave as usize).to_le_bytes());
        std::ptr::copy_nonoverlapping(patch.as_ptr(), site as *mut u8, 19);
        VirtualProtect(site as *mut c_void, 19, old, &mut old);
    }
    GETTER_ADDR.store(site, Ordering::SeqCst);
    // The cave stores rcx at `saved`. Remember that slot.
    OBJECT.store(saved, Ordering::SeqCst);
    Ok(format!("NRAP hero getter hooked site=0x{site:X}"))
}

pub fn want(item_id: i64) -> Option<String> {
    let id = session_id(item_id)?;
    WANTED.store(id as u32, Ordering::SeqCst);
    Some(apply())
}

pub fn apply() -> String {
    let id = WANTED.load(Ordering::SeqCst) as u8;
    if id == 0 {
        return "NRAP hero no starting id yet".into();
    }
    let saved = OBJECT.load(Ordering::SeqCst);
    if saved < 0x10000 {
        return "NRAP hero object not captured".into();
    }
    let object = unsafe { std::ptr::read_unaligned(saved as *const usize) };
    let Some(slot) = slot_addr(object) else {
        return format!("NRAP hero object not live saved=0x{saved:X} rcx=0x{object:X}");
    };
    let before = unsafe { std::ptr::read_unaligned(slot as *const u8) };
    if before == id {
        return format!("NRAP hero slot=0x{slot:X} already {id}");
    }
    if !write_byte(slot, id) {
        return format!("NRAP hero write failed slot=0x{slot:X}");
    }
    let after = unsafe { std::ptr::read_unaligned(slot as *const u8) };
    format!("NRAP hero slot=0x{slot:X} {before}->{after}")
}
