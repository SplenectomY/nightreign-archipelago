//! Current HP from the Hexinton table.
//! WorldChrMan AOB: 48 8B 05 ?? ?? ?? ?? 0F 28 F1 48 85 C0
//! [WorldChrMan] + 0x174E8 -> +0x1B8 -> +0 -> +0x140 is current HP (i32).

#![cfg(windows)]

use crate::aob::{self, ModuleSpan};
use std::sync::atomic::{AtomicUsize, Ordering};

const WORLD_AOB: &str = "48 8B 05 ?? ?? ?? ?? 0F 28 F1 48 85 C0";
const PLAYER_OFFSET: usize = 0x174E8;
const MODULE_OFFSET: usize = 0x1B8;
const HP_OFFSET: usize = 0x140;

static SLOT: AtomicUsize = AtomicUsize::new(0);

fn rip_slot(span: ModuleSpan, at: usize) -> Option<usize> {
    let hay = span.slice();
    if at + 7 > hay.len() {
        return None;
    }
    let disp = i32::from_le_bytes(hay[at + 3..at + 7].try_into().ok()?);
    Some(span.base + at + 7 + disp as isize as usize)
}

fn read_usize(addr: usize) -> Option<usize> {
    if addr < 0x10000 {
        return None;
    }
    let v = unsafe { std::ptr::read_unaligned(addr as *const usize) };
    if v < 0x10000 { None } else { Some(v) }
}

pub fn init() -> Result<String, String> {
    let span = ModuleSpan::nightreign().ok_or_else(|| "no module".to_string())?;
    let rel = aob::find_pattern(span.slice(), WORLD_AOB).ok_or_else(|| "WorldChrMan AOB not found".to_string())?;
    let slot = rip_slot(span, rel).ok_or_else(|| "WorldChrMan slot unresolved".to_string())?;
    SLOT.store(slot, Ordering::SeqCst);
    Ok(format!("NRAP hp ready worldchr=0x{slot:X}"))
}

fn current_hp_addr() -> Option<usize> {
    let slot = SLOT.load(Ordering::SeqCst);
    let man = read_usize(slot)?;
    let player = read_usize(man + PLAYER_OFFSET)?;
    let module = read_usize(player + MODULE_OFFSET)?;
    let stats = read_usize(module)?;
    Some(stats + HP_OFFSET)
}

fn live_hp() -> Result<(usize, i32, i32), String> {
    if SLOT.load(Ordering::SeqCst) == 0 {
        init()?;
    }
    let addr = current_hp_addr().ok_or_else(|| "player HP not live".to_string())?;
    let current = unsafe { std::ptr::read_unaligned(addr as *const i32) };
    let raw_max = unsafe { std::ptr::read_unaligned((addr + 4) as *const i32) };
    let max = if (1..=200_000).contains(&raw_max) && raw_max >= current { raw_max } else { current.max(1) };
    Ok((addr, current, max))
}

pub fn set_zero() -> Result<String, String> {
    let (addr, before, _) = live_hp()?;
    unsafe { std::ptr::write_unaligned(addr as *mut i32, 0) };
    let after = unsafe { std::ptr::read_unaligned(addr as *const i32) };
    Ok(format!("NRAP hp {before}->{after} at 0x{addr:X}"))
}

pub fn apply_percent(percent: u32) -> Result<String, String> {
    let (addr, before, max) = live_hp()?;
    let pct = percent.clamp(1, 100);
    let damage = ((max as i64) * pct as i64 / 100).max(1) as i32;
    let after_target = before.saturating_sub(damage);
    unsafe { std::ptr::write_unaligned(addr as *mut i32, after_target) };
    let after = unsafe { std::ptr::read_unaligned(addr as *const i32) };
    Ok(format!("NRAP hp percent {pct} max={max} damage={damage} {before}->{after} at 0x{addr:X}"))
}
