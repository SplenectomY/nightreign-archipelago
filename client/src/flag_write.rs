//! In-process SetEventFlag via EventFlagBaseA.

#![cfg(windows)]

use crate::aob::{self, ModuleSpan};
use crate::flagman;
use std::sync::Mutex;

const BASE_A: &str = "48 89 5C 24 08 44 8B 49 1C 44";

type SetFlagFn = unsafe extern "C" fn(inst: usize, flag: u32, on: u32);

static PENDING: Mutex<Vec<u32>> = Mutex::new(Vec::new());
static HEOLSTOR_IN_POOL: Mutex<bool> = Mutex::new(false);
static HEOLSTOR_NEED: Mutex<u32> = Mutex::new(4);
static UNLOCKS: Mutex<Vec<i64>> = Mutex::new(Vec::new());

fn find_setter(span: ModuleSpan) -> Option<usize> {
    let rel = aob::find_pattern(span.slice(), BASE_A)?;
    Some(span.base + rel)
}

pub fn configure_heolstor(in_pool: bool, count: u32) {
    *HEOLSTOR_IN_POOL.lock().unwrap() = in_pool;
    *HEOLSTOR_NEED.lock().unwrap() = count.max(1);
}

pub fn debug_flag_from_toml(text: &str) -> Option<u32> {
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
            if k.trim() == "set_flag" {
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

pub fn flag_for_item(item_id: i64) -> Option<u32> {
    match item_id {
        839_100_001..=839_100_007 => Some(110),
        839_100_008 => Some(115),
        839_100_009 => Some(135),
        839_100_010 => Some(136),
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

pub fn apply_item(item_id: i64) -> Option<String> {
    let gate = note_unlock(item_id);
    let flag = flag_for_item(item_id)?;
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
    let mut q = PENDING.lock().unwrap();
    if q.is_empty() {
        return None;
    }
    let flag = q[0];
    match set_flag(flag, true) {
        Ok(msg) => {
            q.remove(0);
            Some(msg)
        }
        Err(_) => None,
    }
}
