//! CSEventFlagMan lookup matching Elden Ring CSFD4VirtualMemoryFlag.
//! Groups of `divisor` bits (usually 1000), tree at +0x38, holder at +0x28,
//! bits stored MSB-first in each byte.

#![cfg(windows)]

use crate::aob::{self, ModuleSpan};
use std::ffi::c_void;

const MEM_COMMIT: u32 = 0x1000;
const PAGE_NOACCESS: u32 = 0x01;
const PAGE_GUARD: u32 = 0x100;
const MEM_IMAGE: u32 = 0x0100_0000;

const OFF_DIVISOR: usize = 0x1C;
const OFF_ENTRY_SIZE: usize = 0x20;
const OFF_HOLDER: usize = 0x28;
const OFF_ROOT: usize = 0x38;

const NODE_LEFT: usize = 0x0;
const NODE_PARENT: usize = 0x8;
const NODE_RIGHT: usize = 0x10;
const NODE_IS_LEAF: usize = 0x19;
const NODE_GROUP: usize = 0x20;
const NODE_LOC_MODE: usize = 0x28;
const NODE_LOCATION: usize = 0x30;

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
    if ptr < 0x10000 {
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

fn heap_ptr(span: ModuleSpan, ptr: usize, len: usize) -> bool {
    readable(ptr, len) && !is_image(ptr) && !(ptr >= span.base && ptr < span.base + span.size)
}

fn read_usize(ptr: usize) -> Option<usize> {
    if !readable(ptr, 8) {
        return None;
    }
    Some(unsafe { std::ptr::read_unaligned(ptr as *const usize) })
}

fn read_u32(ptr: usize) -> Option<u32> {
    if !readable(ptr, 4) {
        return None;
    }
    Some(unsafe { std::ptr::read_unaligned(ptr as *const u32) })
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
    divisor: u32,
    entry_size: u32,
    holder: usize,
    root: usize,
}

impl FlagMan {
    pub fn get(&self, flag: u32) -> Option<bool> {
        let (base, bit) = self.loc(flag)?;
        let byte = (bit / 8) as usize;
        let shift = 7 - (bit % 8); // CSFD4 stores MSB first
        let v = read_u8(base.saturating_add(byte))?;
        Some((v >> shift) & 1 == 1)
    }

    fn loc(&self, flag: u32) -> Option<(usize, u32)> {
        if self.divisor == 0 || self.entry_size == 0 {
            return None;
        }
        let group = flag / self.divisor;
        let bit = flag % self.divisor;
        let root = self.root;
        let parent = read_usize(root + NODE_PARENT)?;
        let mut current = parent;
        let mut found = root;
        let mut walks = 0u32;
        while read_u8(current + NODE_IS_LEAF)? == 0 {
            walks += 1;
            if walks > 1000 {
                return None;
            }
            let current_group = read_u32(current + NODE_GROUP)?;
            let next = if current_group < group {
                let right = read_usize(current + NODE_RIGHT)?;
                found = current;
                right
            } else {
                read_usize(current + NODE_LEFT)?
            };
            current = next;
        }
        if found == root {
            return None;
        }
        let found_group = read_u32(found + NODE_GROUP)?;
        if group < found_group {
            return None;
        }
        let mode = read_u32(found + NODE_LOC_MODE)?;
        let base = match mode {
            2 => read_usize(found + NODE_LOCATION)?,
            1 => {
                let loc = read_u32(found + NODE_LOCATION)? as usize;
                self.holder.saturating_add(loc.saturating_mul(self.entry_size as usize))
            }
            _ => return None,
        };
        if !readable(base, ((bit / 8) + 1) as usize) {
            return None;
        }
        Some((base, bit))
    }
}

fn looks_like_flagman(span: ModuleSpan, inst: usize) -> Option<(u32, u32, usize, usize)> {
    if !heap_ptr(span, inst, 0x48) {
        return None;
    }
    let divisor = read_u32(inst + OFF_DIVISOR)?;
    let entry_size = read_u32(inst + OFF_ENTRY_SIZE)?;
    if !(100..=10_000).contains(&divisor) {
        return None;
    }
    if !(8..=4096).contains(&entry_size) {
        return None;
    }
    let holder = read_usize(inst + OFF_HOLDER)?;
    let root = read_usize(inst + OFF_ROOT)?;
    if !heap_ptr(span, holder, 16) || !heap_ptr(span, root, 0x38) {
        return None;
    }
    Some((divisor, entry_size, holder, root))
}

struct Pattern {
    name: &'static str,
    pat: &'static str,
    disp_at: usize,
    next_at: usize,
}

const PATTERNS: &[Pattern] = &[
    Pattern {
        name: "kh0nsu_cmp_slot",
        pat: "48 83 3D ?? ?? ?? ?? 00 0F 84 ?? ?? 00 00 44 8B E6 85 C0",
        disp_at: 3,
        next_at: 8,
    },
    Pattern {
        name: "mov_rdi_slot",
        pat: "48 8B 3D ?? ?? ?? ?? 48 85 FF",
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

pub fn resolve() -> Result<FlagMan, String> {
    let span = ModuleSpan::nightreign().ok_or_else(|| "no nightreign module".to_string())?;
    let hay = span.slice();
    let mut last = "no CSEventFlagMan singleton with valid divisor/holder".to_string();

    for pat in PATTERNS {
        let mut from = 0usize;
        let mut hits = 0u32;
        while from < hay.len() && hits < 48 {
            let Some(rel) = aob::find_pattern(&hay[from..], pat.pat) else {
                break;
            };
            let instr = from + rel;
            from = instr + 1;
            hits += 1;
            let Some(slot) = aob::rip_rel(span, instr, pat.disp_at, pat.next_at) else {
                continue;
            };
            if !readable(slot, 8) {
                continue;
            }
            let Some(inst) = read_usize(slot) else {
                continue;
            };
            let Some((divisor, entry_size, holder, root)) = looks_like_flagman(span, inst) else {
                last = format!(
                    "{} slot=0x{slot:X} inst=0x{inst:X} failed flagman shape check",
                    pat.name
                );
                continue;
            };
            return Ok(FlagMan {
                singleton_slot: slot,
                instance: inst,
                bits: holder,
                layout: "CSFD4VirtualMemoryFlag",
                pattern: pat.name,
                divisor,
                entry_size,
                holder,
                root,
            });
        }
    }
    Err(last)
}
