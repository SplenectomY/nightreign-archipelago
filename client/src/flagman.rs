//! Locate CSEventFlagMan and read a flag bit.
//!
//! Nightreign has no public binding. We try Elden Ring-family AOBs and a few
//! holder layouts, then log which combination produced a readable bitfield.

#![cfg(windows)]

use crate::aob::{self, ModuleSpan};
use std::ffi::c_void;

const MEM_COMMIT: u32 = 0x1000;
const PAGE_NOACCESS: u32 = 0x01;
const PAGE_GUARD: u32 = 0x100;

#[repr(C)]
struct MemoryBasicInformation {
    base_address: *mut c_void,
    allocation_base: *mut c_void,
    allocation_protect: u32,
    _pad1: u32,
    region_size: usize,
    state: u32,
    protect: u32,
    type_: u32,
    _pad2: u32,
}

#[link(name = "kernel32")]
extern "system" {
    fn VirtualQuery(
        addr: *const c_void,
        info: *mut MemoryBasicInformation,
        len: usize,
    ) -> usize;
}

fn readable(ptr: usize, len: usize) -> bool {
    if ptr == 0 || ptr < 0x10000 {
        return false;
    }
    unsafe {
        let mut info = std::mem::zeroed::<MemoryBasicInformation>();
        let got = VirtualQuery(ptr as *const c_void, &mut info, std::mem::size_of::<MemoryBasicInformation>());
        if got == 0 {
            return false;
        }
        if info.state != MEM_COMMIT {
            return false;
        }
        if info.protect & (PAGE_NOACCESS | PAGE_GUARD) != 0 {
            return false;
        }
        let start = info.base_address as usize;
        start.saturating_add(info.region_size) >= ptr.saturating_add(len)
    }
}

fn read_usize(ptr: usize) -> Option<usize> {
    if !readable(ptr, 8) {
        return None;
    }
    Some(unsafe { std::ptr::read_unaligned(ptr as *const usize) })
}

fn read_u8(ptr: usize) -> Option<u8> {
    if !readable(ptr, 1) {
        return None;
    }
    Some(unsafe { std::ptr::read_unaligned(ptr as *const u8) })
}

#[derive(Clone, Copy, Debug)]
pub struct FlagMan {
    pub singleton_slot: usize,
    pub instance: usize,
    pub bits: usize,
    pub layout: &'static str,
    pub pattern: &'static str,
}

impl FlagMan {
    pub fn get(&self, flag: u32) -> Option<bool> {
        let byte = (flag / 8) as usize;
        let bit = flag % 8;
        let v = read_u8(self.bits.saturating_add(byte))?;
        Some((v >> bit) & 1 == 1)
    }
}

struct Pattern {
    name: &'static str,
    pat: &'static str,
    disp_at: usize,
    next_at: usize,
}

/// RIP-relative loads/comparisons used around CSEventFlagMan in ER-family builds.
const PATTERNS: &[Pattern] = &[
    Pattern {
        name: "cmp_slot_zero",
        pat: "48 83 3D ?? ?? ?? ?? 00",
        disp_at: 3,
        next_at: 8,
    },
    Pattern {
        name: "mov_rcx_slot",
        pat: "48 8B 0D ?? ?? ?? ?? 44 8B C2",
        disp_at: 3,
        next_at: 7,
    },
    Pattern {
        name: "mov_rcx_slot_b2",
        pat: "48 8B 0D ?? ?? ?? ?? B2 01",
        disp_at: 3,
        next_at: 7,
    },
    Pattern {
        name: "mov_rax_slot",
        pat: "48 8B 05 ?? ?? ?? ?? 48 85 C0 74",
        disp_at: 3,
        next_at: 7,
    },
];

const HOLDER_OFFS: &[usize] = &[0x20, 0x28, 0x30, 0x38, 0x40, 0x48, 0x58];
const BITS_OFFS: &[usize] = &[0x00, 0x08, 0x10, 0x18, 0x20, 0x28];

fn looks_like_bitfield(bits: usize) -> bool {
    // Flag 150 lives in byte 18. Need a committed block that can hold flags
    // through at least 200, and is not all 0xFF (unmapped-looking).
    if !readable(bits, 32) {
        return false;
    }
    let mut nonzero = 0u32;
    let mut ff = 0u32;
    for i in 0..32 {
        if let Some(b) = read_u8(bits + i) {
            if b != 0 {
                nonzero += 1;
            }
            if b == 0xFF {
                ff += 1;
            }
        } else {
            return false;
        }
    }
    ff < 28 && nonzero < 32
}

fn try_layouts(instance: usize) -> Option<(usize, &'static str)> {
    for &hold_off in HOLDER_OFFS {
        let holder = match read_usize(instance + hold_off) {
            Some(p) if readable(p, 8) => p,
            _ => continue,
        };
        for &bits_off in BITS_OFFS {
            let bits = if bits_off == 0 {
                holder
            } else {
                match read_usize(holder + bits_off) {
                    Some(p) => p,
                    None => continue,
                }
            };
            if looks_like_bitfield(bits) {
                let name: &'static str = match (hold_off, bits_off) {
                    (0x28, 0) => "man+0x28 as bits",
                    (0x28, 0x18) => "man+0x28 -> +0x18 bits",
                    (0x20, 0) => "man+0x20 as bits",
                    (0x20, 0x18) => "man+0x20 -> +0x18 bits",
                    (0x30, 0) => "man+0x30 as bits",
                    (0x30, 0x18) => "man+0x30 -> +0x18 bits",
                    _ => "probed layout",
                };
                return Some((bits, name));
            }
        }
    }
    None
}

pub fn resolve() -> Result<FlagMan, String> {
    let span = ModuleSpan::nightreign().ok_or_else(|| "no nightreign module".to_string())?;
    let hay = span.slice();

    if let Some(off) = aob::find_ascii(hay, b"CSEventFlagMan") {
        // Presence confirms the type is in this build. Address is not the singleton.
        let _ = off;
    }

    let mut last_err = "no CSEventFlagMan pattern matched".to_string();
    for pat in PATTERNS {
        let mut search_from = 0usize;
        while search_from < hay.len() {
            let Some(rel) = aob::find_pattern(&hay[search_from..], pat.pat) else {
                break;
            };
            let instr_off = search_from + rel;
            search_from = instr_off + 1;
            let Some(slot) = aob::rip_rel(span, instr_off, pat.disp_at, pat.next_at) else {
                continue;
            };
            if !readable(slot, 8) {
                continue;
            }
            let Some(instance) = read_usize(slot) else {
                continue;
            };
            if instance == 0 || !readable(instance, 0x60) {
                last_err = format!(
                    "{} hit slot=0x{slot:X} instance unset (game not in session yet)",
                    pat.name
                );
                continue;
            }
            if let Some((bits, layout)) = try_layouts(instance) {
                return Ok(FlagMan {
                    singleton_slot: slot,
                    instance,
                    bits,
                    layout,
                    pattern: pat.name,
                });
            }
            last_err = format!(
                "{} instance=0x{instance:X} but no bitfield layout fit",
                pat.name
            );
        }
    }
    Err(last_err)
}
