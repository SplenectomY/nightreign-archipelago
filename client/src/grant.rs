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

pub fn set_skin(model: i32) -> Result<String, String> {
    if SLOT.load(Ordering::SeqCst) == 0 {
        let _ = init();
    }
    let player = player_data().ok_or_else(|| "GameDataMan+8 not live".to_string())?;
    let addr = player + 0x350;
    let before = unsafe { std::ptr::read_unaligned(addr as *const i32) };
    if before == model {
        return Ok(format!("NRAP skin already {model}"));
    }
    unsafe { std::ptr::write_unaligned(addr as *mut i32, model) };
    let after = unsafe { std::ptr::read_unaligned(addr as *const i32) };
    Ok(format!("NRAP skin {before}->{after} player=0x{player:X}"))
}


const EMPTY_GEAR: i32 = 110000;
const WYLDER_SWORD: i32 = 3_750_000;
const WYLDER_SHIELD: i32 = 30_750_000;
const GEAR_SLOTS: [usize; 6] = [0x30C, 0x314, 0x31C, 0x310, 0x318, 0x320];

pub fn swap_wylder_gear(session: u8) -> Result<String, String> {
    // Hexinton weapon list. Right hand replaces the greatsword, left replaces the shield.
        let (right, left) = match session {
        2 => (18_750_000, 32_750_000), // Guardian halberd, greatshield
        3 => (41_750_000, EMPTY_GEAR), // Ironeye bow
        4 => (1_750_000, EMPTY_GEAR), // Duchess dagger
        5 => (23_750_000, EMPTY_GEAR), // Raider greataxe
        6 => (21_750_000, EMPTY_GEAR), // Revenant claws
        7 => (33_750_000, EMPTY_GEAR), // Recluse staff
        8 => (9_750_000, EMPTY_GEAR), // Executor blade
        9 => (5_750_000, EMPTY_GEAR), // Scholar
        10 => (11_750_000, EMPTY_GEAR), // Undertaker
        _ => return Err("no starter gear".into()),
    };
    if SLOT.load(Ordering::SeqCst) == 0 {
        let _ = init();
    }
    let player = player_data().ok_or_else(|| "GameDataMan+8 not live".to_string())?;
    let mut changed = Vec::new();
    let mut sword_used = false;
    let mut shield_used = false;
    for off in GEAR_SLOTS {
        let addr = player + off;
        let cur = unsafe { std::ptr::read_unaligned(addr as *const i32) };
        let next = if cur == WYLDER_SWORD {
            let n = if sword_used { EMPTY_GEAR } else { right };
            sword_used = true;
            n
        } else if cur == WYLDER_SHIELD {
            let n = if shield_used { EMPTY_GEAR } else { left };
            shield_used = true;
            n
        } else {
            continue;
        };
        if next == cur {
            continue;
        }
        unsafe { std::ptr::write_unaligned(addr as *mut i32, next) };
        changed.push(format!("+{off:X}:{cur}->{next}"));
    }
    if changed.is_empty() {
        return Ok("NRAP gear already clear".into());
    }
    Ok(format!("NRAP gear {}", changed.join(" ")))
}

pub fn restore_wylder_gear() -> Result<String, String> {
    // Reverse of swap_wylder_gear: put Wylder sword + shield back into starter slots
    // when another character's starter is currently equipped.
    const OTHER_STARTERS: &[i32] = &[
        18_750_000, 32_750_000, // Guardian
        41_750_000, // Ironeye
        1_750_000, // Duchess
        23_750_000, // Raider
        21_750_000, // Revenant
        33_750_000, // Recluse staff
        9_750_000, // Executor
        5_750_000, // Scholar
        11_750_000, // Undertaker
    ];
    if SLOT.load(Ordering::SeqCst) == 0 {
        let _ = init();
    }
    let player = player_data().ok_or_else(|| "GameDataMan+8 not live".to_string())?;
    let mut changed = Vec::new();
    let mut sword_used = false;
    let mut shield_used = false;
    for off in GEAR_SLOTS {
        let addr = player + off;
        let cur = unsafe { std::ptr::read_unaligned(addr as *const i32) };
        if !OTHER_STARTERS.contains(&cur) {
            continue;
        }
        let next = if !sword_used {
            sword_used = true;
            WYLDER_SWORD
        } else if !shield_used {
            shield_used = true;
            WYLDER_SHIELD
        } else {
            EMPTY_GEAR
        };
        if next == cur {
            continue;
        }
        unsafe { std::ptr::write_unaligned(addr as *mut i32, next) };
        changed.push(format!("+{off:X}:{cur}->{next}"));
    }
    if changed.is_empty() {
        return Ok("NRAP gear already clear".into());
    }
    Ok(format!("NRAP gear {}", changed.join(" ")))
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

pub fn grant_murk(amount: i32) -> Result<String, String> {
    add_murk(amount)
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

use std::collections::HashMap;
use std::sync::Mutex;
use std::time::{Duration, Instant};

const RUNE_1000: i64 = 839_100_121;
const RUNE_5000: i64 = 839_100_122;
const RUNE_10000: i64 = 839_100_123;

struct RuneBank {
    seed: String,
    piles: HashMap<i64, i32>,
    paid_now: HashMap<i64, i32>,
    day1_at: Option<Instant>,
    saw_day1: bool,
    paid_this_rise: bool,
}

fn empty_bank() -> RuneBank {
    RuneBank {
        seed: String::new(),
        piles: HashMap::new(),
        paid_now: HashMap::new(),
        day1_at: None,
        saw_day1: false,
        paid_this_rise: false,
    }
}

static RUNES: std::sync::LazyLock<Mutex<RuneBank>> = std::sync::LazyLock::new(|| Mutex::new(empty_bank()));

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
        bank.piles.clear();
        bank.paid_now.clear();
        bank.day1_at = None;
        bank.saw_day1 = false;
        bank.paid_this_rise = false;
    }
}

pub fn note_starting_runes(seed: &str, index: i64, item_id: i64, expedition: bool) -> Option<String> {
    let amt = rune_amount(item_id)?;
    let mut bank = RUNES.lock().unwrap();
    bind_seed(&mut bank, seed);
    if bank.piles.contains_key(&index) {
        return Some(format!("NRAP starting runes skip already {} ({item_id}) index={index}", crate::ap::item_name(item_id)));
    }
    bank.piles.insert(index, amt);
    let total: i32 = bank.piles.values().sum();
    let name = crate::ap::item_name(item_id);
    if expedition {
        bank.paid_now.insert(index, amt);
        drop(bank);
        return Some(match add_runes(amt) {
            Ok(detail) => format!("NRAP starting runes +{amt} {name} ({item_id}) index={index} banked={total} {detail}"),
            Err(e) => format!("NRAP starting runes banked +{amt} {name} index={index} grant failed ({e})"),
        });
    }
    Some(format!("NRAP starting runes banked +{amt} {name} ({item_id}) index={index} total={total}"))
}

pub fn tick_starting_runes(seed: &str, day1: bool) -> Option<String> {
    let mut bank = RUNES.lock().unwrap();
    bind_seed(&mut bank, seed);
    if bank.piles.is_empty() {
        bank.day1_at = None;
        bank.saw_day1 = day1;
        bank.paid_this_rise = day1;
        return None;
    }
    if !day1 {
        bank.day1_at = None;
        bank.saw_day1 = false;
        bank.paid_this_rise = false;
        bank.paid_now.clear();
        return None;
    }
    if !bank.saw_day1 {
        bank.saw_day1 = true;
        bank.day1_at = Some(Instant::now());
        bank.paid_this_rise = false;
    }
    if bank.paid_this_rise {
        return None;
    }
    let started = bank.day1_at.get_or_insert_with(Instant::now);
    if started.elapsed() < Duration::from_secs(5) {
        return None;
    }
    let total: i32 = bank.piles.iter().filter(|(k, _)| !bank.paid_now.contains_key(k)).map(|(_, v)| *v).sum();
    let n = bank.piles.len() - bank.paid_now.len();
    drop(bank);
    if total <= 0 {
        RUNES.lock().unwrap().paid_this_rise = true;
        return None;
    }
    match add_runes(total) {
        Ok(detail) => {
            RUNES.lock().unwrap().paid_this_rise = true;
            Some(format!("NRAP starting runes payout +{total} from {n} after day 1 {detail}"))
        }
        Err(e) => Some(format!("NRAP starting runes payout failed +{total} ({e})")),
    }
}
