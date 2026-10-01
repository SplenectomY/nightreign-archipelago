//! In-process SetEventFlag via EventFlagBaseA.

#![cfg(windows)]

use crate::aob::{self, ModuleSpan};
use crate::flagman;
use crate::menu;
use std::sync::Mutex;

const BASE_A: &str = "48 89 5C 24 08 44 8B 49 1C 44";

type SetFlagFn = unsafe extern "C" fn(inst: usize, flag: u32, on: u32);

static PENDING: Mutex<Vec<u32>> = Mutex::new(Vec::new());
static SHOP_GRANTED: Mutex<Vec<u32>> = Mutex::new(Vec::new());
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


fn shop_release(item_id: i64) -> Option<u32> {
    match item_id {
        839100501 => Some(68800),
        839100503 => Some(68801),
        839100504 => Some(68802),
        839100505 => Some(68803),
        839100506 => Some(68804),
        839100507 => Some(68805),
        839100510 => Some(68806),
        839100511 => Some(68807),
        839100512 => Some(68808),
        839100513 => Some(68809),
        839100514 => Some(68810),
        839100515 => Some(68811),
        839100516 => Some(68812),
        839100517 => Some(68813),
        839100518 => Some(68814),
        839100519 => Some(68815),
        839100520 => Some(68816),
        839100521 => Some(68817),
        839100522 => Some(68818),
        839100523 => Some(68819),
        839100524 => Some(68820),
        839100525 => Some(68821),
        839100526 => Some(68822),
        839100527 => Some(68823),
        839100528 => Some(68824),
        839100529 => Some(68825),
        839100530 => Some(68826),
        839100531 => Some(68827),
        839100532 => Some(68828),
        839100533 => Some(68829),
        839100534 => Some(68830),
        839100535 => Some(68831),
        839100536 => Some(68832),
        839100537 => Some(68833),
        839100538 => Some(68834),
        839100539 => Some(68835),
        839100540 => Some(68836),
        839100541 => Some(68837),
        839100542 => Some(68838),
        839100543 => Some(68839),
        839100544 => Some(68840),
        839100545 => Some(68841),
        839100546 => Some(68842),
        839100547 => Some(68843),
        839100548 => Some(68844),
        839100549 => Some(68845),
        839100550 => Some(68846),
        839100551 => Some(68847),
        839100552 => Some(68848),
        839100553 => Some(68849),
        839100554 => Some(68850),
        839100555 => Some(68851),
        839100556 => Some(68852),
        839100557 => Some(68853),
        839100558 => Some(68854),
        839100560 => Some(68855),
        839100561 => Some(68856),
        839100562 => Some(68857),
        839100563 => Some(68858),
        839100564 => Some(68859),
        839100565 => Some(68860),
        839100566 => Some(68861),
        839100567 => Some(68862),
        839100568 => Some(68863),
        839100569 => Some(68864),
        839100570 => Some(68865),
        839100571 => Some(68866),
        839100572 => Some(68867),
        839100573 => Some(68868),
        839100574 => Some(68869),
        _ => None,
    }
}

fn shop_flag(item_id: i64) -> Option<u32> {
    match item_id {
        839_100_501 => Some(67600),
        839_100_503 => Some(67650),
        839_100_504 => Some(67640),
        839_100_505 => Some(67000),
        839_100_506 => Some(67700),
        839_100_507 => Some(67670),
        839_100_510 => Some(68500),
        839_100_511 => Some(60010),
        839_100_512 => Some(60060),
        839_100_513 => Some(60110),
        839_100_514 => Some(60160),
        839_100_515 => Some(60210),
        839_100_516 => Some(60260),
        839_100_517 => Some(60310),
        839_100_518 => Some(60360),
        839_100_519 => Some(60400),
        839_100_520 => Some(60410),
        839_100_521 => Some(60420),
        839_100_522 => Some(67730),
        839_100_523 => Some(67770),
        839_100_524 => Some(67780),
        839_100_525 => Some(67790),
        839_100_526 => Some(67720),
        839_100_527 => Some(67660),
        839_100_528 => Some(67740),
        839_100_529 => Some(67840),
        839_100_530 => Some(67860),
        839_100_531 => Some(67870),
        839_100_532 => Some(67880),
        839_100_533 => Some(67610),
        839_100_534 => Some(67620),
        839_100_535 => Some(67750),
        839_100_536 => Some(67680),
        839_100_537 => Some(67760),
        839_100_538 => Some(67800),
        839_100_539 => Some(67850),
        839_100_540 => Some(67710),
        839_100_541 => Some(67890),
        839_100_542 => Some(67830),
        839_100_543 => Some(67630),
        839_100_544 => Some(67690),
        839_100_545 => Some(67810),
        839_100_546 => Some(67820),
        839_100_547 => Some(67900),
        839_100_548 => Some(67910),
        839_100_549 => Some(67920),
        839_100_550 => Some(67940),
        839_100_551 => Some(67950),
        839_100_552 => Some(67960),
        839_100_553 => Some(67930),
        839_100_554 => Some(67970),
        839_100_555 => Some(67980),
        839_100_556 => Some(60510),
        839_100_557 => Some(60560),
        839_100_558 => Some(60430),
        _ => None,
    }
}

pub fn flag_for_item(item_id: i64) -> Option<u32> {
    if let Some(flag) = nightfarer_flag(item_id) {
        return Some(flag);
    }
    if let Some(flag) = shop_flag(item_id) {
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

fn is_defeat_flag(flag: u32) -> bool {
    matches!(flag, 150..=156 | 161 | 162)
}

pub fn note_defeat(flag: u32) -> Option<String> {
    if !is_defeat_flag(flag) || *HEOLSTOR_IN_POOL.lock().unwrap() {
        return None;
    }
    let mut got = UNLOCKS.lock().unwrap();
    let id = flag as i64;
    if !got.contains(&id) {
        got.push(id);
    }
    let need = *HEOLSTOR_NEED.lock().unwrap();
    if got.len() < need as usize {
        return Some(format!(
            "NRAP Heolstor gate {}/{} defeats (local, not in pool)",
            got.len(),
            need
        ));
    }
    match set_flag(115, true) {
        Ok(msg) => Some(format!("{msg} after {} Nightlord defeats", got.len())),
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


pub fn shop_granted(flag: u32) -> bool {
    SHOP_GRANTED.lock().unwrap().contains(&flag)
}

pub fn apply_item(item_id: i64) -> Option<String> {
    if let Some(release) = shop_release(item_id) {
        return Some(match set_flag(release, true) {
            Ok(msg) => format!("{msg} (shop row unlocked)"),
            Err(e) => format!("NRAP shop release queued flag={release} ({e})"),
        });
    }
    let gate = None;
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
