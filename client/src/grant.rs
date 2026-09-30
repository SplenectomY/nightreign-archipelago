//! Apply Murk Bundle to addresses discovered by the value scan.

#![cfg(windows)]

use std::fs;
use std::path::PathBuf;
use std::sync::atomic::{AtomicI32, AtomicUsize, Ordering};
use std::sync::Mutex;

const MURK_BUNDLE_ID: i64 = 839_100_100;
const DEFAULT_BUNDLE: i32 = 1000;

static BUNDLE_AMOUNT: AtomicI32 = AtomicI32::new(DEFAULT_BUNDLE);
static PENDING_MURK: AtomicI32 = AtomicI32::new(0);
static NEXT_GRANT: AtomicUsize = AtomicUsize::new(0);
static STATE_PATH: Mutex<Option<PathBuf>> = Mutex::new(None);
static CAND_PATH: Mutex<Option<PathBuf>> = Mutex::new(None);

fn load_next(path: &PathBuf) -> usize {
    fs::read_to_string(path)
        .ok()
        .and_then(|s| s.trim().parse().ok())
        .unwrap_or(0)
}

fn save_next(path: &PathBuf, n: usize) {
    let _ = fs::write(path, n.to_string());
}

fn load_addrs() -> Vec<usize> {
    let guard = CAND_PATH.lock().unwrap();
    let Some(path) = guard.as_ref() else {
        return Vec::new();
    };
    fs::read_to_string(path)
        .ok()
        .map(|s| {
            s.lines()
                .filter_map(|l| usize::from_str_radix(l.trim().trim_start_matches("0x"), 16).ok())
                .collect()
        })
        .unwrap_or_default()
}

fn add_murk(amount: i32) -> Result<i32, String> {
    let addrs = load_addrs();
    if addrs.is_empty() {
        return Err("no murk_cands.txt addresses".into());
    }
    let mut last = 0i32;
    let mut wrote = 0usize;
    for p in addrs {
        if p < 0x10000 {
            continue;
        }
        unsafe {
            let old = std::ptr::read_unaligned(p as *const i32);
            let new = old.saturating_add(amount);
            std::ptr::write_unaligned(p as *mut i32, new);
            last = new;
            wrote += 1;
        }
    }
    if wrote == 0 {
        return Err("wallet addresses not writable".into());
    }
    Ok(last)
}

pub fn init(dir: Option<&PathBuf>, bundle_amount: i32) -> Result<String, String> {
    BUNDLE_AMOUNT.store(bundle_amount.max(1), Ordering::SeqCst);
    if let Some(dir) = dir {
        let path = dir.join("granted_index.txt");
        NEXT_GRANT.store(load_next(&path), Ordering::SeqCst);
        *STATE_PATH.lock().unwrap() = Some(path);
        *CAND_PATH.lock().unwrap() = Some(dir.join("murk_cands.txt"));
    }
    let n = load_addrs().len();
    Ok(format!(
        "NRAP murk grant ready cands={n} bundle={}",
        BUNDLE_AMOUNT.load(Ordering::SeqCst)
    ))
}

pub fn apply_received(item_id: i64, index: i64) -> Option<String> {
    let next = NEXT_GRANT.load(Ordering::SeqCst) as i64;
    if index < next {
        return None;
    }
    NEXT_GRANT.store((index + 1) as usize, Ordering::SeqCst);
    if let Some(path) = STATE_PATH.lock().unwrap().as_ref() {
        save_next(path, (index + 1) as usize);
    }
    if item_id != MURK_BUNDLE_ID {
        return Some(format!("NRAP grant skipped {item_id} (no handler)"));
    }
    let amt = BUNDLE_AMOUNT.load(Ordering::SeqCst);
    match add_murk(amt) {
        Ok(new) => Some(format!("NRAP granted Murk +{amt} wallet={new}")),
        Err(e) => {
            PENDING_MURK.fetch_add(amt, Ordering::SeqCst);
            Some(format!("NRAP grant queued +{amt} Murk ({e})"))
        }
    }
}

pub fn retry_pending() -> Option<String> {
    let amt = PENDING_MURK.load(Ordering::SeqCst);
    if amt <= 0 {
        return None;
    }
    match add_murk(amt) {
        Ok(new) => {
            PENDING_MURK.store(0, Ordering::SeqCst);
            Some(format!("NRAP granted queued Murk +{amt} wallet={new}"))
        }
        Err(_) => None,
    }
}
