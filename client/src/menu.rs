//! Expedition menu ownership. The selectability AOB fires once for the menu
//! panel, not once per Nightlord, so no code patch is installed.

#![cfg(windows)]

static OWNED: std::sync::atomic::AtomicU32 = std::sync::atomic::AtomicU32::new(0);

const ADEL: u32 = 1 << 0;
const GNOSTER: u32 = 1 << 1;
const MARIS: u32 = 1 << 2;
const LIBRA: u32 = 1 << 3;
const FULGHOR: u32 = 1 << 4;
const CALIGO: u32 = 1 << 5;
const HEOLSTOR: u32 = 1 << 6;
const HARMONIA: u32 = 1 << 7;
const STRAGHESS: u32 = 1 << 8;

pub fn drain_logs() -> Vec<String> {
    Vec::new()
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
    OWNED.fetch_or(bit, std::sync::atomic::Ordering::Relaxed);
}

pub fn init() -> Result<String, String> {
    Ok("NRAP menu probe disabled (site is the menu panel, not a Nightlord row)".into())
}
