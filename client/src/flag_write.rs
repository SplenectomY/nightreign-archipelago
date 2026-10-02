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
        839100600 => Some(68800),
        839100601 => Some(68801),
        839100602 => Some(68802),
        839100603 => Some(68803),
        839100604 => Some(68804),
        839100605 => Some(68805),
        839100606 => Some(68806),
        839100607 => Some(68807),
        839100608 => Some(68808),
        839100609 => Some(68809),
        839100610 => Some(68810),
        839100611 => Some(68811),
        839100612 => Some(68812),
        839100613 => Some(68813),
        839100614 => Some(68814),
        839100615 => Some(68815),
        839100616 => Some(68816),
        839100617 => Some(68817),
        839100618 => Some(68818),
        839100619 => Some(68819),
        839100620 => Some(68820),
        839100621 => Some(68821),
        839100622 => Some(68822),
        839100623 => Some(68823),
        839100624 => Some(68824),
        839100625 => Some(68825),
        839100626 => Some(68826),
        839100627 => Some(68827),
        839100628 => Some(68828),
        839100629 => Some(68829),
        839100630 => Some(68830),
        839100631 => Some(68831),
        839100632 => Some(68832),
        839100633 => Some(68833),
        839100634 => Some(68834),
        839100635 => Some(68835),
        839100636 => Some(68836),
        839100637 => Some(68837),
        839100638 => Some(68838),
        839100639 => Some(68839),
        839100640 => Some(68840),
        839100641 => Some(68841),
        839100642 => Some(68842),
        839100643 => Some(68843),
        839100644 => Some(68844),
        839100645 => Some(68845),
        839100646 => Some(68846),
        839100647 => Some(68847),
        839100648 => Some(68848),
        839100649 => Some(68849),
        839100650 => Some(68850),
        839100651 => Some(68851),
        839100652 => Some(68852),
        839100653 => Some(68853),
        839100654 => Some(68854),
        839100655 => Some(68855),
        839100656 => Some(68856),
        839100657 => Some(68857),
        839100658 => Some(68858),
        839100659 => Some(68859),
        839100660 => Some(68860),
        839100661 => Some(68861),
        839100662 => Some(68862),
        839100663 => Some(68863),
        839100664 => Some(68864),
        839100665 => Some(68865),
        839100666 => Some(68866),
        839100667 => Some(68867),
        839100668 => Some(68868),
        839100669 => Some(68869),
        839100670 => Some(68870),
        839100671 => Some(68871),
        839100672 => Some(68872),
        839100673 => Some(68873),
        839100674 => Some(68874),
        839100675 => Some(68875),
        839100676 => Some(68876),
        839100677 => Some(68877),
        839100678 => Some(68878),
        839100679 => Some(68879),
        839100680 => Some(68880),
        839100681 => Some(68881),
        839100682 => Some(68882),
        839100683 => Some(68883),
        _ => None,
    }
}

fn shop_flag(item_id: i64) -> Option<u32> {
    match item_id {
        839100600 => Some(67000),
        839100601 => Some(67010),
        839100602 => Some(67020),
        839100603 => Some(67030),
        839100604 => Some(67040),
        839100605 => Some(67050),
        839100606 => Some(67060),
        839100607 => Some(67070),
        839100608 => Some(67080),
        839100609 => Some(67090),
        839100610 => Some(67100),
        839100611 => Some(67110),
        839100612 => Some(67120),
        839100613 => Some(67130),
        839100614 => Some(67140),
        839100615 => Some(67150),
        839100616 => Some(67160),
        839100617 => Some(67170),
        839100618 => Some(67180),
        839100619 => Some(67190),
        839100620 => Some(67200),
        839100621 => Some(67210),
        839100622 => Some(67220),
        839100623 => Some(67230),
        839100624 => Some(67240),
        839100625 => Some(67250),
        839100626 => Some(67260),
        839100627 => Some(67270),
        839100628 => Some(68500),
        839100629 => Some(60010),
        839100630 => Some(60060),
        839100631 => Some(60110),
        839100632 => Some(60160),
        839100633 => Some(60210),
        839100634 => Some(60260),
        839100635 => Some(60310),
        839100636 => Some(60360),
        839100637 => Some(60400),
        839100638 => Some(60410),
        839100639 => Some(60420),
        839100640 => Some(67600),
        839100641 => Some(67650),
        839100642 => Some(67640),
        839100643 => Some(67730),
        839100644 => Some(67770),
        839100645 => Some(67780),
        839100646 => Some(67790),
        839100647 => Some(67720),
        839100648 => Some(67660),
        839100649 => Some(67700),
        839100650 => Some(67670),
        839100651 => Some(67740),
        839100652 => Some(67840),
        839100653 => Some(67860),
        839100654 => Some(67870),
        839100655 => Some(67880),
        839100656 => Some(67610),
        839100657 => Some(67620),
        839100658 => Some(67750),
        839100659 => Some(67680),
        839100660 => Some(67760),
        839100661 => Some(67800),
        839100662 => Some(67850),
        839100663 => Some(67710),
        839100664 => Some(67890),
        839100665 => Some(67830),
        839100666 => Some(67630),
        839100667 => Some(67690),
        839100668 => Some(67810),
        839100669 => Some(67820),
        839100670 => Some(67900),
        839100671 => Some(67910),
        839100672 => Some(67920),
        839100673 => Some(67940),
        839100674 => Some(67950),
        839100675 => Some(67960),
        839100676 => Some(67930),
        839100677 => Some(67970),
        839100678 => Some(67980),
        839100679 => Some(60510),
        839100680 => Some(60560),
        839100681 => Some(60430),
        839100682 => Some(67280),
        839100683 => Some(67290),
        _ => None,
    }
}

fn unlock_flag(item_id: i64) -> Option<u32> {
    if let Some(flag) = shop_release(item_id) {
        return Some(flag);
    }
    if shop_flag(item_id).is_some() {
        return None;
    }
    flag_for_item(item_id)
}

static CACHED: Mutex<Vec<u32>> = Mutex::new(Vec::new());
static CACHE_PATH: Mutex<Option<std::path::PathBuf>> = Mutex::new(None);
static CACHE_SEED: Mutex<String> = Mutex::new(String::new());
static ARMED: std::sync::atomic::AtomicBool = std::sync::atomic::AtomicBool::new(false);

fn write_cache(path: &std::path::PathBuf, seed: &str, flags: &[u32]) {
    let mut body = format!("seed={seed}\n");
    for flag in flags {
        body.push_str(&flag.to_string());
        body.push('\n');
    }
    let _ = std::fs::write(path, body);
}

pub fn load_cache(dir: Option<&std::path::PathBuf>) -> String {
    let Some(dir) = dir else {
        return "NRAP unlock cache no dir".into();
    };
    let path = dir.join("received_unlocks.txt");
    let text = std::fs::read_to_string(&path).unwrap_or_default();
    let mut seed = String::new();
    let mut flags = Vec::new();
    for line in text.lines() {
        if let Some(rest) = line.trim().strip_prefix("seed=") {
            seed = rest.to_string();
        } else if let Ok(flag) = line.trim().parse() {
            flags.push(flag);
        }
    }
    *CACHE_SEED.lock().unwrap() = seed.clone();
    *CACHED.lock().unwrap() = flags.clone();
    *CACHE_PATH.lock().unwrap() = Some(path);
    format!("NRAP unlock cache loaded {} seed={seed}", flags.len())
}

static BOSS_KILLS: Mutex<u32> = Mutex::new(0);
static DAY1_KILLS: Mutex<u32> = Mutex::new(0);
static DAY2_KILLS: Mutex<u32> = Mutex::new(0);
static EVERGAOL: Mutex<u32> = Mutex::new(0);
static TOWER: Mutex<u32> = Mutex::new(0);
static IGNORE_CLEAR: Mutex<Option<std::time::Instant>> = Mutex::new(None);
static BOSS_PATH: Mutex<Option<std::path::PathBuf>> = Mutex::new(None);

fn boss_path(dir: &std::path::PathBuf) -> std::path::PathBuf {
    dir.join("boss_kills.txt")
}

pub fn load_boss_kills(dir: Option<&std::path::PathBuf>) -> String {
    let Some(dir) = dir else { return "NRAP boss kills no dir".into() };
    let path = boss_path(dir);
    let text = std::fs::read_to_string(&path).unwrap_or_default();
    let mut seed = String::new();
    let mut count = 0u32;
    for line in text.lines() {
        if let Some(rest) = line.trim().strip_prefix("seed=") { seed = rest.to_string(); }
        if let Some(rest) = line.trim().strip_prefix("count=") { count = rest.parse().unwrap_or(0); }
        if let Some(rest) = line.trim().strip_prefix("day1=") { *DAY1_KILLS.lock().unwrap() = rest.parse().unwrap_or(0); }
        if let Some(rest) = line.trim().strip_prefix("day2=") { *DAY2_KILLS.lock().unwrap() = rest.parse().unwrap_or(0); }
        if let Some(rest) = line.trim().strip_prefix("evergaol=") { *EVERGAOL.lock().unwrap() = rest.parse().unwrap_or(0); }
        if let Some(rest) = line.trim().strip_prefix("tower=") { *TOWER.lock().unwrap() = rest.parse().unwrap_or(0); }
    }
    *BOSS_PATH.lock().unwrap() = Some(path);
    *BOSS_KILLS.lock().unwrap() = count;
    format!("NRAP boss kills loaded {count} day1={} day2={} seed={seed}", *DAY1_KILLS.lock().unwrap(), *DAY2_KILLS.lock().unwrap())
}

pub fn note_boss_kill() -> Option<String> {
    if !ARMED.load(std::sync::atomic::Ordering::SeqCst) {
        return Some("NRAP boss kill ignored, seed not armed".into());
    }
    let mut count = BOSS_KILLS.lock().unwrap();
    *count += 1;
    let n = *count;
    if let Some(path) = BOSS_PATH.lock().unwrap().as_ref() {
        let seed = CACHE_SEED.lock().unwrap().clone();
        let _ = std::fs::write(path, format!("seed={seed}\ncount={n}\n"));
    }
    Some(format!("NRAP boss kills {n}"))
}

pub fn boss_kill_count() -> u32 {
    *BOSS_KILLS.lock().unwrap()
}

fn write_counts() {
    let Some(path) = BOSS_PATH.lock().unwrap().clone() else { return };
    let seed = CACHE_SEED.lock().unwrap().clone();
    let _ = std::fs::write(path, format!(
        "seed={seed}\ncount={}\nday1={}\nday2={}\nevergaol={}\ntower={}\n",
        *BOSS_KILLS.lock().unwrap(), *DAY1_KILLS.lock().unwrap(), *DAY2_KILLS.lock().unwrap(),
        *EVERGAOL.lock().unwrap(), *TOWER.lock().unwrap()
    ));
}

pub fn note_day_boss(flag: u32) -> Option<(u32, i64)> {
    if !ARMED.load(std::sync::atomic::Ordering::SeqCst) {
        return None;
    }
    let (count, loc) = if flag == 7502 {
        let mut n = DAY1_KILLS.lock().unwrap();
        *n += 1;
        (*n, 839001001 + *n as i64)
    } else if flag == 7507 {
        let mut n = DAY2_KILLS.lock().unwrap();
        *n += 1;
        (*n, 839001101 + *n as i64)
    } else {
        return None;
    };
    write_counts();
    Some((count, loc))
}

pub fn note_return() {
    *IGNORE_CLEAR.lock().unwrap() = Some(std::time::Instant::now());
}

pub fn note_toggle(flag: u32, rising: bool) -> Option<(u32, i64)> {
    if !ARMED.load(std::sync::atomic::Ordering::SeqCst) {
        return None;
    }
    if !rising {
        let ignore = IGNORE_CLEAR.lock().unwrap().map(|t| t.elapsed().as_secs() < 20).unwrap_or(false);
        if ignore {
            return None;
        }
    }
    let (count, loc) = if flag == 8145 {
        let mut n = EVERGAOL.lock().unwrap();
        *n += 1;
        (*n, 839001201 + *n as i64)
    } else if flag == 8140 {
        let mut n = TOWER.lock().unwrap();
        *n += 1;
        (*n, 839001301 + *n as i64)
    } else {
        return None;
    };
    write_counts();
    Some((count, loc))
}

pub fn bind_seed(seed: &str) -> String {
    let known = CACHE_SEED.lock().unwrap().clone();
    if !known.is_empty() && known != seed {
        CACHED.lock().unwrap().clear();
        if let Some(path) = CACHE_PATH.lock().unwrap().as_ref() {
            write_cache(path, seed, &[]);
        }
        *CACHE_SEED.lock().unwrap() = seed.to_string();
        *BOSS_KILLS.lock().unwrap() = 0;
        *DAY1_KILLS.lock().unwrap() = 0;
        *DAY2_KILLS.lock().unwrap() = 0;
        *EVERGAOL.lock().unwrap() = 0;
        *TOWER.lock().unwrap() = 0;
        write_counts();
        ARMED.store(true, std::sync::atomic::Ordering::SeqCst);
        return format!("NRAP unlock cache cleared, seed {known} -> {seed}");
    }
    *CACHE_SEED.lock().unwrap() = seed.to_string();
    ARMED.store(true, std::sync::atomic::Ordering::SeqCst);
    if let Some(path) = BOSS_PATH.lock().unwrap().as_ref() {
        let text = std::fs::read_to_string(path).unwrap_or_default();
        let saved = text.lines().find_map(|l| l.trim().strip_prefix("seed=")).unwrap_or("");
        if saved != seed {
            *BOSS_KILLS.lock().unwrap() = 0;
            let _ = std::fs::write(path, format!("seed={seed}\ncount=0\n"));
        }
    }
    format!("NRAP unlock cache armed seed={seed} flags={} boss_kills={}", CACHED.lock().unwrap().len(), boss_kill_count())
}

pub fn remember_unlock(item_id: i64) {
    let Some(flag) = unlock_flag(item_id) else { return };
    let mut flags = CACHED.lock().unwrap();
    if !flags.contains(&flag) {
        flags.push(flag);
        if let Some(path) = CACHE_PATH.lock().unwrap().as_ref() {
            write_cache(path, &CACHE_SEED.lock().unwrap(), &flags);
        }
    }
}

pub fn reapply_cached() -> Option<String> {
    if !ARMED.load(std::sync::atomic::Ordering::SeqCst) {
        return None;
    }
    let flags = CACHED.lock().unwrap().clone();
    if flags.is_empty() {
        return None;
    }
    let mut set = 0u32;
    for flag in flags {
        if set_flag(flag, true).is_ok() {
            set += 1;
        }
    }
    if set == 0 {
        return None;
    }
    None
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
