//! Murk Bundle via the game AddSoul function. Target 1 is Murk.
//! GameDataMan: 48 8B 0D ?? ?? ?? ?? F3 48 0F 2C C0
//! Function:    ?? 8B 81 D0 00 00 00 ?? 8B D1 B9

#![cfg(windows)]

use crate::aob::{self, ModuleSpan};
use std::sync::atomic::{AtomicI32, AtomicUsize, Ordering};

const MURK_BUNDLE_ID: i64 = 839_100_100;
const DEFAULT_BUNDLE: i32 = 1000;
const GAMEDATA_AOB: &str = "48 8B 0D ?? ?? ?? ?? F3 48 0F 2C C0";
const MURK_AOB: &str = "?? 8B 81 D0 00 00 00 ?? 8B D1 B9";

static BUNDLE: AtomicI32 = AtomicI32::new(DEFAULT_BUNDLE);
static PENDING: AtomicI32 = AtomicI32::new(0);
static SLOT: AtomicUsize = AtomicUsize::new(0);
static FUNC: AtomicUsize = AtomicUsize::new(0);

type AddFn = unsafe extern "C" fn(player: usize, amount: i32) -> i32;

fn rip_slot(span: ModuleSpan, at: usize) -> Option<usize> {
    let hay = span.slice();
    if at + 7 > hay.len() {
        return None;
    }
    let disp = i32::from_le_bytes(hay[at + 3..at + 7].try_into().ok()?);
    Some(span.base + at + 7 + disp as isize as usize)
}

pub fn init() -> Result<String, String> {
    let span = ModuleSpan::nightreign().ok_or_else(|| "no module".to_string())?;
    let data_rel = aob::find_pattern(span.slice(), GAMEDATA_AOB)
        .ok_or_else(|| "GameDataMan AOB not found".to_string())?;
    let fn_rel = aob::find_pattern(span.slice(), MURK_AOB)
        .ok_or_else(|| "murk function AOB not found".to_string())?;
    let slot = rip_slot(span, data_rel).ok_or_else(|| "GameDataMan slot unresolved".to_string())?;
    SLOT.store(slot, Ordering::SeqCst);
    FUNC.store(span.base + fn_rel, Ordering::SeqCst);
    Ok(format!(
        "NRAP murk fn=0x{:X} gamedata=0x{slot:X} bundle={}",
        span.base + fn_rel,
        BUNDLE.load(Ordering::SeqCst)
    ))
}

fn player_data() -> Option<usize> {
    let slot = SLOT.load(Ordering::SeqCst);
    if slot < 0x10000 {
        return None;
    }
    let man = unsafe { std::ptr::read_unaligned(slot as *const usize) };
    if man < 0x10000 {
        return None;
    }
    let player = unsafe { std::ptr::read_unaligned((man + 8) as *const usize) };
    if player < 0x10000 { None } else { Some(player) }
}

fn add_murk(amount: i32) -> Result<String, String> {
    let func = FUNC.load(Ordering::SeqCst);
    let player = player_data().ok_or_else(|| "GameDataMan+8 not live".to_string())?;
    if func < 0x10000 || amount <= 0 {
        return Err("murk function missing".into());
    }
    let f: AddFn = unsafe { std::mem::transmute(func) };
    let ret = unsafe { f(player, amount) };
    Ok(format!("player=0x{player:X} ret={ret}"))
}

fn murk_amount(item_id: i64) -> Option<i32> {
    match item_id {
        839_100_100 | 839_100_101 => Some(if item_id == 839_100_100 { 1000 } else { 2000 }),
        839_100_102 => Some(4000),
        839_100_103 => Some(6000),
        839_100_104 => Some(10000),
        _ => None,
    }
}

pub fn want(item_id: i64) -> Option<String> {
    let amt = murk_amount(item_id)?;
    match add_murk(amt) {
        Ok(detail) => Some(format!("NRAP murk +{amt} {detail}")),
        Err(e) => {
            PENDING.fetch_add(amt, Ordering::SeqCst);
            Some(format!("NRAP murk queued +{amt} ({e})"))
        }
    }
}

pub fn retry() -> Option<String> {
    let amt = PENDING.load(Ordering::SeqCst);
    if amt <= 0 {
        return None;
    }
    match add_murk(amt) {
        Ok(detail) => {
            PENDING.store(0, Ordering::SeqCst);
            Some(format!("NRAP murk granted queued +{amt} {detail}"))
        }
        Err(_) => None,
    }
}
