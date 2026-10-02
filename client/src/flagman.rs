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

/// nightreign.exe 1.3.3.0 CSFD4VirtualMemoryFlag::GetFlag (GiovanavoiG/NightreignAP).
const VMF_GET_FLAG_RVA: usize = 0x60CE40;
const VMF_GET_FLAG_SIG: &[u8] = &[0x44, 0x8B, 0x41, 0x1C, 0x44, 0x8B, 0xDA];

// 110, 150, Delicate Burning, Polite Bow, Warm Welcome, Strength, Heartening Cry, Calm Down
const SHOP_PROBE: &[u32] = &[110, 150, 67000, 67600, 67640, 67650, 67700, 67670];

type GetFlagFn = unsafe extern "C" fn(inst: usize, flag: u32) -> u8;

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

fn bit_at(base: usize, bit: u32, msb: bool) -> Option<bool> {
    let byte = (bit / 8) as usize;
    let shift = if msb { 7 - (bit % 8) } else { bit % 8 };
    let v = read_u8(base.saturating_add(byte))?;
    Some((v >> shift) & 1 == 1)
}

fn fmt_bit(v: Option<bool>) -> char {
    match v {
        Some(true) => '1',
        Some(false) => '0',
        None => '-',
    }
}

fn game_get_fn() -> Option<GetFlagFn> {
    let span = ModuleSpan::nightreign()?;
    let addr = span.base + VMF_GET_FLAG_RVA;
    if !readable(addr, VMF_GET_FLAG_SIG.len()) {
        return None;
    }
    let bytes = unsafe { std::slice::from_raw_parts(addr as *const u8, VMF_GET_FLAG_SIG.len()) };
    if bytes != VMF_GET_FLAG_SIG {
        return None;
    }
    Some(unsafe { std::mem::transmute(addr) })
}

static GAME_FLAG: std::sync::Mutex<()> = std::sync::Mutex::new(());

pub fn lock_game_flags() -> std::sync::MutexGuard<'static, ()> {
    GAME_FLAG.lock().unwrap()
}

fn game_get(inst: usize, flag: u32) -> Option<bool> {
    let _hold = GAME_FLAG.lock().unwrap();
    if inst < 0x10000 {
        return None;
    }
    let f = game_get_fn()?;
    Some(unsafe { f(inst, flag) } != 0)
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
        game_get(self.instance, flag).or_else(|| self.read_flag(flag, true, false))
    }

    /// Slab read only. GetFlag takes the game lock, and a menu holds that lock.
    pub fn get_local(&self, flag: u32) -> Option<bool> {
        self.read_flag(flag, true, false)
    }

    /// Copy each group slab and return flag ids that went 0 to 1 since the last call.
    /// The usize is how many group slabs were readable.
    pub fn diff_rising(&self, prev: &mut Vec<Vec<u8>>) -> (Vec<u32>, usize) {
        let groups = self.entry_count as usize;
        let bytes = (self.divisor as usize).div_ceil(8);
        if groups == 0 || bytes == 0 || bytes > 4096 || groups > 10_000 {
            return (Vec::new(), 0);
        }
        if prev.len() != groups {
            prev.clear();
            prev.resize(groups, Vec::new());
        }
        let mut rose = Vec::new();
        let mut readable_groups = 0usize;
        // Resolved once. VirtualQuery on every group made a scan take about a minute.
        static BASES: std::sync::Mutex<Vec<usize>> = std::sync::Mutex::new(Vec::new());
        let mut bases = BASES.lock().unwrap();
        if bases.len() != groups {
            bases.clear();
            bases.resize(groups, 0);
        }
        for group in 0..groups {
            let mut base = bases[group];
            if base == 0 {
                let ptr = read_usize(self.holder.saturating_add(group.saturating_mul(8))).unwrap_or(0);
                let inline = self.holder.saturating_add(group.saturating_mul(self.entry_size as usize));
                base = if ptr > 0x10000 && readable(ptr, bytes) { ptr } else { inline };
                if !readable(base, bytes) {
                    continue;
                }
                bases[group] = base;
            }
            readable_groups += 1;
            let mut now = vec![0u8; bytes];
            unsafe {
                std::ptr::copy_nonoverlapping(base as *const u8, now.as_mut_ptr(), bytes);
            }
            let old = &prev[group];
            if old.len() == bytes {
                for i in 0..bytes {
                    let up = now[i] & !old[i];
                    if up == 0 {
                        continue;
                    }
                    for bit in 0..8 {
                        if up & (1 << (7 - bit)) != 0 {
                            rose.push((group as u32) * self.divisor + (i as u32) * 8 + bit as u32);
                        }
                    }
                }
            }
            prev[group] = now;
        }
        (rose, readable_groups)
    }

    fn group_base(&self, flag: u32, ptr_table: bool) -> Option<(usize, u32)> {
        if self.divisor == 0 {
            return None;
        }
        let group = flag / self.divisor;
        let bit = flag % self.divisor;
        if group >= self.entry_count {
            return None;
        }
        if ptr_table {
            let slot = self.holder.saturating_add((group as usize).saturating_mul(8));
            let base = read_usize(slot)?;
            if base < 0x10000 || !readable(base, (bit / 8) as usize + 1) {
                return None;
            }
            Some((base, bit))
        } else {
            if self.entry_size == 0 {
                return None;
            }
            let base = self
                .holder
                .saturating_add((group as usize).saturating_mul(self.entry_size as usize));
            if !readable(base, (bit / 8) as usize + 1) {
                return None;
            }
            Some((base, bit))
        }
    }

    fn read_flag(&self, flag: u32, msb: bool, ptr_table: bool) -> Option<bool> {
        let (base, bit) = self.group_base(flag, ptr_table)?;
        bit_at(base, bit, msb)
    }

    fn probe_line(&self) -> String {
        let mut s = format!("inst=0x{:X} {}", self.instance, self.pattern);
        s.push_str(" game=");
        for flag in SHOP_PROBE {
            s.push(fmt_bit(game_get(self.instance, *flag)));
        }
        for (name, msb, ptrs) in [
            ("slab_msb", true, false),
            ("slab_lsb", false, false),
            ("ptr_msb", true, true),
            ("ptr_lsb", false, true),
        ] {
            s.push(' ');
            s.push_str(name);
            s.push('=');
            for flag in SHOP_PROBE {
                s.push(fmt_bit(self.read_flag(*flag, msb, ptrs)));
            }
        }
        s
    }
}

fn looks_like_flagman(span: ModuleSpan, inst: usize) -> Option<(u32, u32, u32, usize)> {
    if !heap_ptr(span, inst, 0x48) {
        return None;
    }
    let divisor = read_u32(inst + OFF_DIVISOR)?;
    let entry_size = read_u32(inst + OFF_ENTRY_SIZE)?;
    let entry_count = read_u32(inst + OFF_ENTRY_COUNT)?;
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
    let holder_bytes = (entry_count as usize)
        .saturating_mul(entry_size as usize)
        .min(4096);
    if !heap_ptr(span, holder, holder_bytes.max(16)) {
        return None;
    }
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

pub fn collect_all() -> Vec<FlagMan> {
    let Some(span) = ModuleSpan::nightreign() else {
        return Vec::new();
    };
    let hay = span.slice();
    let mut out = Vec::new();
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
            if inst < 0x10000 {
                continue;
            }
            let Some((divisor, entry_size, entry_count, holder)) = looks_like_flagman(span, inst)
            else {
                continue;
            };
            if out.iter().any(|m: &FlagMan| m.instance == inst) {
                continue;
            }
            out.push(FlagMan {
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
    out
}

pub fn resolve() -> Result<FlagMan, String> {
    collect_all()
        .into_iter()
        .next()
        .ok_or_else(|| "no CSEventFlagMan singleton with divisor=1000".to_string())
}

