//! In-process SetEventFlag via EventFlagBaseA.

#![cfg(windows)]

use crate::aob::{self, ModuleSpan};
use crate::flagman;
use crate::menu;
use std::sync::Mutex;

const BASE_A: &str = "48 89 5C 24 08 44 8B 49 1C 44";

type SetFlagFn = unsafe extern "C" fn(inst: usize, flag: u32, on: u32);

static PENDING: Mutex<Vec<u32>> = Mutex::new(Vec::new());
static HEOLSTOR_IN_POOL: Mutex<bool> = Mutex::new(false);
static HEOLSTOR_NEED: Mutex<u32> = Mutex::new(4);
static UNLOCKS: Mutex<Vec<i64>> = Mutex::new(Vec::new());
static GRANTED_NIGHTFARERS: Mutex<Vec<u32>> = Mutex::new(Vec::new());
static SUPPRESSED: Mutex<Vec<u32>> = Mutex::new(Vec::new());

fn find_setter(span: ModuleSpan) -> Option<usize> {
    let rel = aob::find_pattern(span.slice(), BASE_A)?;
    Some(span.base + rel)
}

pub fn configure_heolstor(in_pool: bool, count: u32) {
    *HEOLSTOR_IN_POOL.lock().unwrap() = in_pool;
    *HEOLSTOR_NEED.lock().unwrap() = count.max(1);
}

fn debug_value(text: &str, key: &str) -> Option<u32> {
    let mut in_debug = false;
    for line in text.lines() {
        let line = line.trim();
        if line.starts_with('[') {
            in_debug = line == "[debug]";
            continue;
        }
        if !in_debug || line.starts_with('#') {
            continue;
        }
        if let Some((k, v)) = line.split_once('=') {
            if k.trim() == key {
                let n: u32 = v.trim().parse().ok()?;
                if n == 0 {
                    return None;
                }
                return Some(n);
            }
        }
    }
    None
}

pub fn debug_flag_from_toml(text: &str) -> Option<u32> {
    debug_value(text, "set_flag")
}

pub fn debug_clear_flag_from_toml(text: &str) -> Option<u32> {
    debug_value(text, "clear_flag")
}

pub fn suppress_flag(flag: u32) {
    let mut held = SUPPRESSED.lock().unwrap();
    if !held.contains(&flag) {
        held.push(flag);
    }
    GRANTED_NIGHTFARERS.lock().unwrap().retain(|f| *f != flag);
}

pub fn set_flag(flag: u32, on: bool) -> Result<String, String> {
    let span = ModuleSpan::nightreign().ok_or_else(|| "no nightreign module".to_string())?;
    let fn_addr = find_setter(span).ok_or_else(|| "EventFlagBaseA AOB not found".to_string())?;
    let man = flagman::resolve().map_err(|e| format!("flagman: {e}"))?;
    if man.instance < 0x10000 {
        return Err(format!("flagman instance not live (0x{:X})", man.instance));
    }
    let before = man.get(flag);
    let setter: SetFlagFn = unsafe { std::mem::transmute(fn_addr) };
    unsafe {
        setter(man.instance, flag, u32::from(on));
    }
    let after = man.get(flag);
    Ok(format!(
        "NRAP SetEventFlag {flag} {} fn=0x{fn_addr:X} inst=0x{:X} before={before:?} after={after:?}",
        on as u8,
        man.instance
    ))
}

/// HeroParam.characterUnlockFlag. Defaults were 0, so they are regulation flags.
/// Duchess, Revenant, Scholar, and Undertaker already had quest flags.
fn nightfarer_flag(item_id: i64) -> Option<u32> {
    match item_id {
        839_100_301 => Some(222),
        839_100_302 => Some(6031),
        839_100_303 => Some(225),
        839_100_304 => Some(227),
        839_100_305 => Some(226),
        839_100_306 => Some(224),
        839_100_307 => Some(223),
        839_100_308 => Some(6037),
        839_100_309 => Some(6038),
        839_100_310 => Some(6039),
        _ => None,
    }
}

fn is_nightfarer(flag: u32) -> bool {
    matches!(flag, 222..=227 | 6031 | 6037 | 6038 | 6039)
}

/// Regulation override flags. 210 and 211 are named, so Everdark starts at 212.
pub fn flag_for_item(item_id: i64) -> Option<u32> {
    if let Some(flag) = nightfarer_flag(item_id) {
        return Some(flag);
    }
    match item_id {
        839_100_001 => Some(189),
        839_100_002 => Some(190),
        839_100_003 => Some(191),
        839_100_004 => Some(192),
        839_100_005 => Some(193),
        839_100_006 => Some(194),
        839_100_007 => Some(195),
        839_100_008 => Some(115),
        839_100_009 => Some(135),
        839_100_010 => Some(136),
        839_100_111 => Some(212),
        839_100_112 => Some(213),
        839_100_113 => Some(214),
        839_100_114 => Some(215),
        839_100_115 => Some(216),
        839_100_116 => Some(217),
        839_100_117 => Some(218),
        839_100_118 => Some(219),
        _ => None,
    }
}

fn note_unlock(item_id: i64) -> Option<String> {
    if !(839_100_001..=839_100_007).contains(&item_id) && item_id != 839_100_009 && item_id != 839_100_010 {
        return None;
    }
    if *HEOLSTOR_IN_POOL.lock().unwrap() {
        return None;
    }
    let mut got = UNLOCKS.lock().unwrap();
    if !got.contains(&item_id) {
        got.push(item_id);
    }
    let need = *HEOLSTOR_NEED.lock().unwrap();
    if got.len() < need as usize {
        return Some(format!(
            "NRAP Heolstor gate {}/{} (local, not in pool)",
            got.len(),
            need
        ));
    }
    match set_flag(115, true) {
        Ok(msg) => Some(format!("{msg} after {} Nightlord unlocks", got.len())),
        Err(e) => {
            let mut q = PENDING.lock().unwrap();
            if !q.contains(&115) {
                q.push(115);
            }
            Some(format!("NRAP Heolstor gate met, flag 115 queued ({e})"))
        }
    }
}

fn remember_nightfarer(flag: u32) {
    if SUPPRESSED.lock().unwrap().contains(&flag) {
        return;
    }
    if is_nightfarer(flag) {
        let mut got = GRANTED_NIGHTFARERS.lock().unwrap();
        if !got.contains(&flag) {
            got.push(flag);
        }
    }
}

fn reapply_nightfarers() -> Option<String> {
    let flags = GRANTED_NIGHTFARERS.lock().unwrap().clone();
    if flags.is_empty() {
        return None;
    }
    let mut out = Vec::new();
    for flag in flags {
        match set_flag(flag, true) {
            Ok(msg) if msg.contains("after=Some(true)") && msg.contains("before=Some(false)") => {
                out.push(msg);
            }
            Ok(_) => {}
            Err(e) => return Some(format!("NRAP nightfarer grant waiting flag={flag} ({e})")),
        }
    }
    if out.is_empty() {
        None
    } else {
        Some(out.join("; "))
    }
}

pub fn apply_item(item_id: i64) -> Option<String> {
    let gate = note_unlock(item_id);
    let Some(flag) = flag_for_item(item_id) else {
        return gate.or_else(|| Some(format!("NRAP grant {item_id} (no unlock flag)")));
    };
    remember_nightfarer(flag);
    let msg = match set_flag(flag, true) {
        Ok(msg) => msg,
        Err(e) => {
            let mut q = PENDING.lock().unwrap();
            if !q.contains(&flag) {
                q.push(flag);
            }
            format!("NRAP SetEventFlag queued flag={flag} item={item_id} ({e})")
        }
    };
    Some(match gate {
        Some(extra) => format!("{msg}; {extra}"),
        None => msg,
    })
}

pub fn retry_pending() -> Option<String> {
    // Wylder is the default active body. Skip this while clear_flag is holding 222 off.
    if !SUPPRESSED.lock().unwrap().contains(&222) {
        remember_nightfarer(222);
    }
    let reapplied = reapply_nightfarers();
    let mut q = PENDING.lock().unwrap();
    if q.is_empty() {
        return reapplied;
    }
    let flag = q[0];
    match set_flag(flag, true) {
        Ok(msg) => {
            q.remove(0);
            remember_nightfarer(flag);
            Some(match reapplied {
                Some(extra) => format!("{extra}; {msg}"),
                None => msg,
            })
        }
        Err(_) => reapplied,
    }
}
