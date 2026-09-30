//! In-process MapItemMan item drop. Do not call until *map_slot is non-null (Hold).

#![cfg(windows)]

use crate::aob::{self, ModuleSpan};
use std::ffi::c_void;
use std::sync::Mutex;

const MAPITEMMAN_AOB: &str = "48 8B C8 E8 ?? ?? ?? ?? 0F 28 00 66 0F 7F 44 24 50";
const MAPITEMMAN_OFF: usize = 0x11;
const ITEMDROP_CALL_AOB: &str = "41 0F B6 E9 41 0F B6 F8 48 8B DA 48 8B F1 33 C0 48 89 44 24 30";
const ITEMDROP_FUNC_OFF: isize = -0x27;
const TLS_FETCHER_AOB: &str = "8D 41 0F 03 C2 83 E0 F0 41 89 00 8B 0D";

const MEM_COMMIT: u32 = 0x1000;
const MEM_RESERVE: u32 = 0x2000;
const PAGE_READWRITE: u32 = 0x04;
const ITEM_DATA_SIZE: usize = 0x50;

#[link(name = "kernel32")]
extern "system" {
    fn VirtualAlloc(addr: *mut c_void, size: usize, typ: u32, prot: u32) -> *mut c_void;
    fn TlsSetValue(index: u32, value: *mut c_void) -> i32;
}

type DropFn = unsafe extern "C" fn(map_item_man: usize, item_data: usize, a3: u32, a4: u32);

struct DropState {
    map_slot: usize,
    drop_fn: usize,
    tls_fetcher: usize,
    item_data: usize,
    fake_tls: usize,
}

static STATE: Mutex<Option<DropState>> = Mutex::new(None);
static PENDING: Mutex<Vec<(i32, u8)>> = Mutex::new(Vec::new());

fn rip_slot(span: ModuleSpan, match_off: usize, extra: usize) -> Option<usize> {
    let hay = span.slice();
    let at = match_off + extra;
    if at + 7 > hay.len() {
        return None;
    }
    let disp = i32::from_le_bytes(hay[at + 3..at + 7].try_into().ok()?);
    Some(at + 7 + span.base + disp as isize as usize)
}

fn alloc(size: usize) -> Option<usize> {
    let p = unsafe { VirtualAlloc(std::ptr::null_mut(), size, MEM_COMMIT | MEM_RESERVE, PAGE_READWRITE) };
    if p.is_null() {
        None
    } else {
        Some(p as usize)
    }
}

fn tls_index(fetcher: usize) -> Option<u32> {
    if fetcher < 0x10000 {
        return None;
    }
    unsafe {
        let p = fetcher + 0xD;
        let eax = std::ptr::read_unaligned(p as *const i32) as isize;
        let idx = std::ptr::read_unaligned((p as isize + eax + 4) as *const u32);
        Some(idx)
    }
}

fn map_item_man(st: &DropState) -> usize {
    unsafe { std::ptr::read_unaligned(st.map_slot as *const usize) }
}

pub fn init() -> Result<String, String> {
    let span = ModuleSpan::nightreign().ok_or_else(|| "no module".to_string())?;
    let man_rel = aob::find_pattern(span.slice(), MAPITEMMAN_AOB)
        .ok_or_else(|| "MAPITEMMAN AOB not found".to_string())?;
    let drop_rel = aob::find_pattern(span.slice(), ITEMDROP_CALL_AOB)
        .ok_or_else(|| "ITEMDROP AOB not found".to_string())?;
    let tls_rel = aob::find_pattern(span.slice(), TLS_FETCHER_AOB)
        .ok_or_else(|| "TLS fetcher AOB not found".to_string())?;
    let map_slot = rip_slot(span, man_rel, MAPITEMMAN_OFF)
        .ok_or_else(|| "MapItemMan slot rip failed".to_string())?;
    let drop_fn = (span.base as isize + drop_rel as isize + ITEMDROP_FUNC_OFF) as usize;
    let tls_fetcher = span.base + tls_rel;
    let item_data = alloc(ITEM_DATA_SIZE).ok_or_else(|| "ItemData alloc failed".to_string())?;
    let fake_tls = alloc(0x100).ok_or_else(|| "tls context alloc failed".to_string())?;
    *STATE.lock().unwrap() = Some(DropState {
        map_slot,
        drop_fn,
        tls_fetcher,
        item_data,
        fake_tls,
    });
    Ok(format!(
        "NRAP drop ready map_slot=0x{map_slot:X} fn=0x{drop_fn:X} tls=0x{tls_fetcher:X}"
    ))
}

fn drop_now(item_id: i32, qty: u8) -> Result<String, String> {
    let st = STATE.lock().unwrap();
    let st = st.as_ref().ok_or_else(|| "drop not initialized".to_string())?;
    let man = map_item_man(st);
    if man < 0x10000 {
        return Err(format!("MapItemMan null (slot 0x{:X})", st.map_slot));
    }
    unsafe {
        let buf = st.item_data as *mut u8;
        std::ptr::write_bytes(buf, 0xFF, ITEM_DATA_SIZE);
        std::ptr::write_unaligned(st.item_data as *mut i32, item_id);
        std::ptr::write_unaligned((st.item_data + 4) as *mut i32, -1);
        std::ptr::write_unaligned((st.item_data + 8) as *mut i32, -1);
        std::ptr::write_unaligned((st.item_data + 0x14) as *mut i32, -1);
        std::ptr::write_unaligned((st.item_data + 0x18) as *mut i32, -1);
        std::ptr::write(buf.add(0x48), qty);
    }
    if let Some(idx) = tls_index(st.tls_fetcher) {
        unsafe {
            TlsSetValue(idx, st.fake_tls as *mut c_void);
        }
    }
    let f: DropFn = unsafe { std::mem::transmute(st.drop_fn) };
    unsafe {
        f(man, st.item_data, 0, 1);
    }
    Ok(format!(
        "NRAP drop item={item_id} qty={qty} man=0x{man:X} fn=0x{:X}",
        st.drop_fn
    ))
}

fn queue(item_id: i32, qty: u8) {
    let mut q = PENDING.lock().unwrap();
    q.push((item_id, qty));
}

pub fn apply_item(item_id: i64, drop_goods_id: i32) -> Option<String> {
    if item_id != 839_100_100 {
        return None;
    }
    if drop_goods_id == 0 {
        return Some("NRAP drop skipped (grant.drop_item_id = 0)".into());
    }
    match drop_now(drop_goods_id, 1) {
        Ok(msg) => Some(msg),
        Err(e) => {
            queue(drop_goods_id, 1);
            Some(format!("NRAP drop queued item={drop_goods_id} ({e})"))
        }
    }
}

pub fn retry_pending() -> Option<String> {
    let mut q = PENDING.lock().unwrap();
    if q.is_empty() {
        return None;
    }
    let (id, qty) = q[0];
    match drop_now(id, qty) {
        Ok(msg) => {
            q.remove(0);
            Some(msg)
        }
        Err(_) => None,
    }
}

pub fn drop_item_id_from_toml(text: &str) -> i32 {
    let mut in_grant = false;
    for line in text.lines() {
        let line = line.trim();
        if line.starts_with('[') {
            in_grant = line == "[grant]";
            continue;
        }
        if !in_grant || line.starts_with('#') {
            continue;
        }
        if let Some((k, v)) = line.split_once('=') {
            if k.trim() == "drop_item_id" {
                return v.trim().parse().unwrap_or(0);
            }
        }
    }
    0
}

pub fn debug_drop_from_toml(text: &str) -> Option<i32> {
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
            if k.trim() == "drop_item" {
                let n: i32 = v.trim().parse().ok()?;
                if n == 0 {
                    return None;
                }
                return Some(n);
            }
        }
    }
    None
}

pub fn debug_drop(item_id: i32) -> String {
    match drop_now(item_id, 1) {
        Ok(msg) => msg,
        Err(e) => {
            queue(item_id, 1);
            format!("NRAP debug drop queued item={item_id} ({e})")
        }
    }
}
