//! Murk Bundle via the game AddSoul function. Target 1 is Murk.
//! GameDataMan: 48 8B 0D ?? ?? ?? ?? F3 48 0F 2C C0
//! Function:    ?? 8B 81 D0 00 00 00 ?? 8B D1 B9

#![cfg(windows)]

use crate::aob::{self, ModuleSpan};
use std::sync::atomic::{AtomicI32, AtomicUsize, Ordering};

const DEFAULT_BUNDLE: i32 = 1000;
const GAMEDATA_AOB: &str = "48 8B 0D ?? ?? ?? ?? F3 48 0F 2C C0";
const MURK_AOB: &str = "?? 8B 81 D0 00 00 00 ?? 8B D1 B9";
const RUNE_AOB: &str = "?? 8B D9 ?? 8D 04 17";

static BUNDLE: AtomicI32 = AtomicI32::new(DEFAULT_BUNDLE);
static PENDING: AtomicI32 = AtomicI32::new(0);
static SLOT: AtomicUsize = AtomicUsize::new(0);
static FUNC: AtomicUsize = AtomicUsize::new(0);
static RUNE_FUNC: AtomicUsize = AtomicUsize::new(0);

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
    let rune = aob::find_pattern(span.slice(), RUNE_AOB).map(|rel| span.base + rel.saturating_sub(0xD)).unwrap_or(0);
    RUNE_FUNC.store(rune, Ordering::SeqCst);
    Ok(format!(
        "NRAP murk fn=0x{:X} rune fn=0x{rune:X} gamedata=0x{slot:X} bundle={}",
        span.base + fn_rel,
        BUNDLE.load(Ordering::SeqCst)
    ))
}

pub fn add_runes(amount: i32) -> Result<String, String> {
    let func = RUNE_FUNC.load(Ordering::SeqCst);
    let player = player_data().ok_or_else(|| "GameDataMan+8 not live".to_string())?;
    if func < 0x10000 || amount == 0 {
        return Err("rune function missing".into());
    }
    let f: AddFn = unsafe { std::mem::transmute(func) };
    let ret = unsafe { f(player, amount) };
    Ok(format!("NRAP runes +{amount} player=0x{player:X} ret={ret}"))
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

static PURSE: AtomicI32 = AtomicI32::new(150);
static BUNDLE_AMT: AtomicI32 = AtomicI32::new(300);
static COFFER: AtomicI32 = AtomicI32::new(500);
static CHEST: AtomicI32 = AtomicI32::new(750);
static HOARD: AtomicI32 = AtomicI32::new(1300);

pub fn configure(purse: i32, bundle: i32, coffer: i32, chest: i32, hoard: i32) -> String {
    if purse > 0 { PURSE.store(purse, Ordering::SeqCst); }
    if bundle > 0 { BUNDLE_AMT.store(bundle, Ordering::SeqCst); }
    if coffer > 0 { COFFER.store(coffer, Ordering::SeqCst); }
    if chest > 0 { CHEST.store(chest, Ordering::SeqCst); }
    if hoard > 0 { HOARD.store(hoard, Ordering::SeqCst); }
    format!(
        "NRAP murk amounts purse={} bundle={} coffer={} chest={} hoard={}",
        PURSE.load(Ordering::SeqCst),
        BUNDLE_AMT.load(Ordering::SeqCst),
        COFFER.load(Ordering::SeqCst),
        CHEST.load(Ordering::SeqCst),
        HOARD.load(Ordering::SeqCst),
    )
}

fn murk_amount(item_id: i64) -> Option<i32> {
    let amount = match item_id {
        839_100_105 => PURSE.load(Ordering::SeqCst),
        839_100_101 => BUNDLE_AMT.load(Ordering::SeqCst),
        839_100_102 => COFFER.load(Ordering::SeqCst),
        839_100_103 => CHEST.load(Ordering::SeqCst),
        839_100_104 => HOARD.load(Ordering::SeqCst),
        _ => return None,
    };
    (amount > 0).then_some(amount)
}

fn murk_path(dir: &std::path::Path) -> std::path::PathBuf {
    dir.join("murk_received.txt")
}

fn murk_seen(dir: &std::path::Path, seed: &str, index: i64) -> bool {
    let text = std::fs::read_to_string(murk_path(dir)).unwrap_or_default();
    let mut file_seed = "";
    let mut hit = false;
    for line in text.lines() {
        if let Some(rest) = line.strip_prefix("seed=") {
            file_seed = rest.trim();
        } else if file_seed == seed && line.starts_with(&format!("{index} ")) {
            hit = true;
        }
    }
    hit
}

fn remember_murk(dir: &std::path::Path, seed: &str, index: i64, item_id: i64, amt: i32) {
    let path = murk_path(dir);
    let text = std::fs::read_to_string(&path).unwrap_or_default();
    let mut body = if text.lines().any(|l| l == format!("seed={seed}")) {
        text
    } else {
        format!("seed={seed}\n")
    };
    if !body.ends_with('\n') {
        body.push('\n');
    }
    body.push_str(&format!("{index} {item_id} {amt}\n"));
    let _ = std::fs::write(path, body);
}

pub fn want_once(dir: Option<&std::path::PathBuf>, seed: &str, index: i64, item_id: i64) -> Option<String> {
    let amt = murk_amount(item_id)?;
    if let Some(dir) = dir {
        if murk_seen(dir, seed, index) {
            return Some(format!("NRAP murk skip already {} ({item_id}) index={index}", crate::ap::item_name(item_id)));
        }
    }
    match add_murk(amt) {
        Ok(detail) => {
            if let Some(dir) = dir {
                remember_murk(dir, seed, index, item_id, amt);
            }
            Some(format!("NRAP murk +{amt} {} ({item_id}) index={index} {detail}", crate::ap::item_name(item_id)))
        }
        Err(e) => {
            PENDING.fetch_add(amt, Ordering::SeqCst);
            Some(format!("NRAP murk queued +{amt} {} ({item_id}) index={index} ({e})", crate::ap::item_name(item_id)))
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

use std::collections::{HashMap, HashSet};
use std::sync::Mutex;
use std::time::{Duration, Instant};

const RUNE_1000: i64 = 839_100_121;
const RUNE_5000: i64 = 839_100_122;
const RUNE_10000: i64 = 839_100_123;

struct RuneBank {
    seed: String,
    pending: HashMap<i64, (i32, bool)>,
    granted: HashSet<i64>,
    day1_at: Option<Instant>,
}

static RUNES: Mutex<RuneBank> = Mutex::new(RuneBank {
    seed: String::new(),
    pending: HashMap::new(),
    granted: HashSet::new(),
    day1_at: None,
});

fn rune_amount(item_id: i64) -> Option<i32> {
    match item_id {
        RUNE_1000 => Some(1000),
        RUNE_5000 => Some(5000),
        RUNE_10000 => Some(10000),
        _ => None,
    }
}

pub fn is_starting_rune(item_id: i64) -> bool {
    rune_amount(item_id).is_some()
}

fn bind_seed(bank: &mut RuneBank, seed: &str) {
    if bank.seed != seed {
        bank.seed = seed.to_string();
        bank.pending.clear();
        bank.granted.clear();
        bank.day1_at = None;
    }
}

pub fn note_starting_runes(seed: &str, index: i64, item_id: i64, expedition: bool) -> Option<String> {
    let amt = rune_amount(item_id)?;
    let mut bank = RUNES.lock().unwrap();
    bind_seed(&mut bank, seed);
    if bank.granted.contains(&index) || bank.pending.contains_key(&index) {
        return Some(format!("NRAP starting runes skip already {} ({item_id}) index={index}", crate::ap::item_name(item_id)));
    }
    if expedition {
        drop(bank);
        match add_runes(amt) {
            Ok(detail) => {
                RUNES.lock().unwrap().granted.insert(index);
                return Some(format!("NRAP starting runes +{amt} {} ({item_id}) index={index} {detail}", crate::ap::item_name(item_id)));
            }
            Err(e) => {
                RUNES.lock().unwrap().pending.insert(index, (amt, true));
                return Some(format!("NRAP starting runes retry +{amt} index={index} ({e})"));
            }
        }
    }
    bank.pending.insert(index, (amt, false));
    let total: i32 = bank.pending.values().map(|v| v.0).sum();
    Some(format!("NRAP starting runes banked +{amt} {} ({item_id}) index={index} pending={total}", crate::ap::item_name(item_id)))
}

pub fn tick_starting_runes(seed: &str, day1: bool) -> Option<String> {
    let mut bank = RUNES.lock().unwrap();
    bind_seed(&mut bank, seed);
    if bank.pending.is_empty() {
        bank.day1_at = None;
        return None;
    }
    let rush = bank.pending.values().any(|v| v.1);
    let hold = bank.pending.values().any(|v| !v.1);
    if rush {
        let total: i32 = bank.pending.values().filter(|v| v.1).map(|v| v.0).sum();
        let keys: Vec<i64> = bank.pending.iter().filter(|(_, v)| v.1).map(|(k, _)| *k).collect();
        drop(bank);
        match add_runes(total) {
            Ok(detail) => {
                let mut bank = RUNES.lock().unwrap();
                for k in keys { bank.pending.remove(&k); bank.granted.insert(k); }
                return Some(format!("NRAP starting runes +{total} field retry {detail}"));
            }
            Err(_) => return None,
        }
    }
    if !hold || !day1 {
        if !day1 { bank.day1_at = None; }
        return None;
    }
    let started = bank.day1_at.get_or_insert_with(Instant::now);
    if started.elapsed() < Duration::from_secs(5) {
        return None;
    }
    let total: i32 = bank.pending.values().map(|v| v.0).sum();
    let n = bank.pending.len();
    let saved = bank.pending.clone();
    bank.granted.extend(bank.pending.keys().copied());
    bank.pending.clear();
    bank.day1_at = None;
    drop(bank);
    match add_runes(total) {
        Ok(detail) => Some(format!("NRAP starting runes payout +{total} from {n} after day 1 {detail}")),
        Err(e) => {
            let mut bank = RUNES.lock().unwrap();
            bank.pending = saved;
            for k in bank.pending.keys() { bank.granted.remove(k); }
            bank.day1_at = Some(Instant::now());
            Some(format!("NRAP starting runes payout failed +{total} ({e})"))
        }
    }
}
