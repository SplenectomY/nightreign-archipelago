//! CSEventFlagMan lookup matching Elden Ring CSFD4VirtualMemoryFlag.

#![cfg(windows)]

use crate::aob::{self, ModuleSpan};
use std::ffi::c_void;

const MEM_COMMIT: u32 = 0x1000;
const PAGE_NOACCESS: u32 = 0x01;
const PAGE_GUARD: u32 = 0x100;
const MEM_IMAGE: u32 = 0x0100_0000;

const OFF_DIVISOR: usize = 0x1C;
const OFF_ENTRY_SIZE: usize = 0x20;
const OFF_ENTRY_COUNT: usize = 0x24;
const OFF_HOLDER: usize = 0x28;
const OFF_ROOT: usize = 0x38;

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
    pub divisor: u32,
    pub entry_size: u32,
    pub entry_count: u32,
    holder: usize,
}

impl FlagMan {
    pub fn describe(&self) -> String {
        format!(
            "NRAP flagman pattern={} layout={} slot=0x{:X} inst=0x{:X} holder=0x{:X} divisor={} entry_size={} entry_count={}",
            self.pattern,
            self.layout,
            self.singleton_slot,
            self.instance,
            self.holder,
            self.divisor,
            self.entry_size,
            self.entry_count
        )
    }

    pub fn get(&self, flag: u32) -> Option<bool> {
        let (base, bit) = self.loc(flag)?;
        let byte = (bit / 8) as usize;
        let shift = 7 - (bit % 8);
        let v = read_u8(base.saturating_add(byte))?;
        Some((v >> shift) & 1 == 1)
    }

    fn loc(&self, flag: u32) -> Option<(usize, u32)> {
        if self.divisor == 0 || self.entry_size == 0 {
            return None;
        }
        let group = flag / self.divisor;
        let bit = flag % self.divisor;
        if group >= self.entry_count {
            return None;
        }
        let base = self
            .holder
            .saturating_add((group as usize).saturating_mul(self.entry_size as usize));
        let need = (bit / 8) as usize + 1;
        if !readable(base, need) {
            return None;
        }
        Some((base, bit))
    }
}

fn looks_like_flagman(span: ModuleSpan, inst: usize) -> Option<(u32, u32, u32, usize)> {
    if !heap_ptr(span, inst, 0x48) {
        return None;
    }
    let divisor = read_u32(inst + OFF_DIVISOR)?;
    let entry_size = read_u32(inst + OFF_ENTRY_SIZE)?;
    let entry_count = read_u32(inst + OFF_ENTRY_COUNT)?;
    // ER/NR family: 1000 bits/group, 125-byte rows.
    if divisor != 1000 {
        return None;
    }
    if entry_size < 16 || entry_size > 4096 {
        return None;
    }
    if entry_count == 0 || entry_count > 10_000 {
        return None;
    }
    let holder = read_usize(inst + OFF_HOLDER)?;
    let root = read_usize(inst + OFF_ROOT).unwrap_or(0);
    let holder_bytes = (entry_count as usize).saturating_mul(entry_size as usize).min(4096);
    if !heap_ptr(span, holder, holder_bytes.max(16)) {
        return None;
    }
    let _ = root;
    Some((divisor, entry_size, entry_count, holder))
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
    let mut last = "no CSEventFlagMan singleton with divisor=1000".to_string();

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
            let Some((divisor, entry_size, entry_count, holder)) = looks_like_flagman(span, inst)
            else {
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
                layout: "CSFD4 holder[group]",
                pattern: pat.name,
                divisor,
                entry_size,
                entry_count,
                holder,
            });
        }
    }
    Err(last)
}
