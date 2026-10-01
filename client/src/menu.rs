//! Expedition menu ownership. Both the Rust detour and the passthrough cave crashed
//! on opening the board, so no code patch is installed.

#![cfg(windows)]

use std::sync::Mutex;

const ADEL: u32 = 1 << 0;
const GNOSTER: u32 = 1 << 1;
const MARIS: u32 = 1 << 2;
const LIBRA: u32 = 1 << 3;
const FULGHOR: u32 = 1 << 4;
const CALIGO: u32 = 1 << 5;
const HEOLSTOR: u32 = 1 << 6;
const HARMONIA: u32 = 1 << 7;
const STRAGHESS: u32 = 1 << 8;

static OWNED: Mutex<u32> = Mutex::new(0);
static LOGS: Mutex<Vec<String>> = Mutex::new(Vec::new());

fn log(msg: impl Into<String>) {
    if let Ok(mut q) = LOGS.lock() {
        if q.len() < 40 {
            q.push(msg.into());
        }
    }
}

pub fn drain_logs() -> Vec<String> {
    LOGS.lock().map(|mut q| std::mem::take(&mut *q)).unwrap_or_default()
}

pub fn grant(item_id: i64) {
    let bit = match item_id {
        839_100_002 => ADEL,
        839_100_003 => GNOSTER,
        839_100_004 => MARIS,
        839_100_005 => LIBRA,
        839_100_006 => FULGHOR,
        839_100_007 => CALIGO,
        839_100_008 => HEOLSTOR,
        839_100_009 => HARMONIA,
        839_100_010 => STRAGHESS,
        _ => return,
    };
    if let Ok(mut owned) = OWNED.lock() {
        *owned |= bit;
        log(format!("NRAP menu owned bit=0x{bit:X} mask=0x{:X}", *owned));
    }
}

pub fn init() -> Result<String, String> {
    Ok("NRAP menu probe disabled (passthrough cave crashed on the expedition board)".into())
}
