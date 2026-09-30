//! Locate CSEventFlagMan and read a flag bit.

#![cfg(windows)]

use crate::aob::{self, ModuleSpan};
use std::ffi::c_void;

const MEM_COMMIT: u32 = 0x1000;
const PAGE_NOACCESS: u32 = 0x01;
const PAGE_GUARD: u32 = 0x100;
const MEM_IMAGE: u32 = 0x0100_0000;

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

fn query(ptr: usize) -> Option<MemoryBasicInformation> {
    unsafe {
        let mut info = std::mem::zeroed::<MemoryBasicInformation>();
        let got = VirtualQuery(
            ptr as *const c_void,
            &mut info,
            std::mem::size_of::<MemoryBasicInformation>(),
        );
        if got == 0 {
            None
        } else {
            Some(info)
        }
    }
}

fn readable(ptr: usize, len: usize) -> bool {
    if ptr == 0 || ptr < 0x10000 {
        return false;
    }
    let Some(info) = query(ptr) else {
        return false;
    };
    if info.state != MEM_COMMIT {
        return false;
    }
    if info.protect & (PAGE_NOACCESS | PAGE_GUARD) != 0 {
        return false;
    }
    let start = info.base_address as usize;
    start.saturating_add(info.region_size) >= ptr.saturating_add(len)
}

fn is_image(ptr: usize) -> bool {
    query(ptr).map(|i| i.type_ == MEM_IMAGE).unwrap_or(true)
}

fn in_module(span: ModuleSpan, ptr: usize) -> bool {
    ptr >= span.base && ptr < span.base.saturating_add(span.size)
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

/// Specific GetEventFlag-style loads. The generic `cmp [rip],0` AOB is too noisy.
const PATTERNS: &[Pattern] = &[
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

fn looks_like_bitfield(span: ModuleSpan, bits: usize) -> bool {
    if in_module(span, bits) || is_image(bits) {
        return false;
    }
    if !readable(bits, 256) {
        return false;
    }
    true
}

fn try_layouts(span: ModuleSpan, instance: usize) -> Option<(usize, &'static str)> {
    for &hold_off in HOLDER_OFFS {
        let holder = match read_usize(instance + hold_off) {
            Some(p) if readable(p, 8) && !in_module(span, p) && !is_image(p) => p,
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
            if looks_like_bitfield(span, bits) {
                let name: &'static str = match (hold_off, bits_off) {
                    (0x28, 0) => "man+0x28 as bits",
                    (0x28, 0x18) => "man+0x28 -> +0x18 bits",
                    (0x20, 0) => "man+0x20 as bits",
                    (0x20, 0x18) => "man+0x20 -> +0x18 bits",
                    (0x30, 0) => "man+0x30 as bits",
                    (0x30, 0x18) => "man+0x30 -> +0x18 bits",
                    _ => "probed heap layout",
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
    let named = aob::find_ascii(hay, b"CSEventFlagMan").is_some();

    let mut last_err = if named {
        "CSEventFlagMan string present; no live heap bitfield yet".to_string()
    } else {
        "no CSEventFlagMan pattern matched".to_string()
    };

    for pat in PATTERNS {
        let mut search_from = 0usize;
        let mut hits = 0u32;
        while search_from < hay.len() && hits < 32 {
            let Some(rel) = aob::find_pattern(&hay[search_from..], pat.pat) else {
                break;
            };
            let instr_off = search_from + rel;
            search_from = instr_off + 1;
            hits += 1;
            let Some(slot) = aob::rip_rel(span, instr_off, pat.disp_at, pat.next_at) else {
                continue;
            };
            if !readable(slot, 8) {
                continue;
            }
            let Some(instance) = read_usize(slot) else {
                continue;
            };
            if instance == 0 || in_module(span, instance) || is_image(instance) || !readable(instance, 0x60)
            {
                last_err = format!(
                    "{} slot=0x{slot:X} instance not a heap object yet",
                    pat.name
                );
                continue;
            }
            if let Some((bits, layout)) = try_layouts(span, instance) {
                return Ok(FlagMan {
                    singleton_slot: slot,
                    instance,
                    bits,
                    layout,
                    pattern: pat.name,
                });
            }
            last_err = format!(
                "{} instance=0x{instance:X} but no heap bitfield layout fit",
                pat.name
            );
        }
    }
    Err(last_err)
}
