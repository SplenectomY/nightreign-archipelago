//! In-process SetEventFlag via EventFlagBaseA.
//! AOB from jacksonstubblefield/nightreign-ap.

#![cfg(windows)]

use crate::aob::{self, ModuleSpan};
use crate::flagman;

const BASE_A: &str = "48 89 5C 24 08 44 8B 49 1C 44";

type SetFlagFn = unsafe extern "C" fn(inst: usize, flag: u32, on: u32);

fn find_setter(span: ModuleSpan) -> Option<usize> {
    let rel = aob::find_pattern(span.slice(), BASE_A)?;
    Some(span.base + rel)
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

/// Set `flag` to on/off using the live CSFD4 instance.
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
        839_100_001 => Some(110),
        839_100_008 => Some(115),
        839_100_009 => Some(135),
        839_100_010 => Some(136),
        _ => None,
    }
}

pub fn apply_item(item_id: i64) -> Option<String> {
    let flag = flag_for_item(item_id)?;
    match set_flag(flag, true) {
        Ok(msg) => Some(msg),
        Err(e) => Some(format!("NRAP SetEventFlag failed item={item_id} flag={flag}: {e}")),
    }
}
