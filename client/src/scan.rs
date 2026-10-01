#![allow(dead_code, unused)]
//! In-process 4-byte value scan. Parked: requires [scan] enable = true.

#![cfg(windows)]

use std::fs;
use std::path::PathBuf;

#[repr(C)]
struct Mbi {
    base: usize,
    alloc_base: usize,
    alloc_protect: u32,
    _pad: u32,
    region_size: usize,
    state: u32,
    protect: u32,
    ty: u32,
}

#[link(name = "kernel32")]
extern "system" {
    fn VirtualQuery(addr: *const u8, info: *mut Mbi, len: usize) -> usize;
}

const MEM_COMMIT: u32 = 0x1000;
const PAGE_GUARD: u32 = 0x100;
const PAGE_NOACCESS: u32 = 0x01;
const READABLE: u32 = 0x02 | 0x04 | 0x08 | 0x20 | 0x40 | 0x80;
const MAX_HITS: usize = 64;
const MAX_BYTES: usize = 256 * 1024 * 1024;

fn page_readable(p: usize) -> bool {
    if p < 0x10000 {
        return false;
    }
    unsafe {
        let mut mbi = std::mem::zeroed::<Mbi>();
        if VirtualQuery(p as *const u8, &mut mbi, std::mem::size_of::<Mbi>()) == 0 {
            return false;
        }
        mbi.state == MEM_COMMIT
            && mbi.protect & PAGE_GUARD == 0
            && mbi.protect & PAGE_NOACCESS == 0
            && mbi.protect & READABLE != 0
    }
}

fn scan_section_key(text: &str, key: &str) -> Option<String> {
    let mut in_scan = false;
    for line in text.lines() {
        let line = line.trim();
        if line.starts_with('[') {
            in_scan = line == "[scan]";
            continue;
        }
        if !in_scan || line.starts_with('#') {
            continue;
        }
        if let Some((k, v)) = line.split_once('=') {
            if k.trim() == key {
                return Some(v.trim().trim_matches('"').to_string());
            }
        }
    }
    None
}

pub fn enabled(text: &str) -> bool {
    matches!(
        scan_section_key(text, "enable").as_deref(),
        Some("1" | "true" | "True")
    )
}

pub fn murk_from_toml(text: &str) -> Option<i32> {
    scan_section_key(text, "murk")?.parse().ok()
}

pub fn grant_now_from_toml(text: &str) -> bool {
    matches!(
        scan_section_key(text, "grant_now").as_deref(),
        Some("1" | "true" | "True")
    )
}

fn readable_regions() -> Vec<(usize, usize)> {
    let mut out = Vec::new();
    let mut addr = 0x10000usize;
    let mut scanned = 0usize;
    while addr < 0x0000_7FFF_FFFF_0000 && scanned < MAX_BYTES {
        let mut mbi = unsafe { std::mem::zeroed::<Mbi>() };
        let n = unsafe { VirtualQuery(addr as *const u8, &mut mbi, std::mem::size_of::<Mbi>()) };
        if n == 0 {
            break;
        }
        let size = mbi.region_size.max(0x1000);
        if page_readable(mbi.base) && size <= 32 * 1024 * 1024 {
            out.push((mbi.base, size));
            scanned = scanned.saturating_add(size);
        }
        addr = mbi.base.saturating_add(size);
        if addr <= mbi.base {
            break;
        }
    }
    out
}

fn collect(value: i32, only: Option<&[usize]>) -> Vec<usize> {
    let mut hits = Vec::new();
    if let Some(list) = only {
        for &p in list {
            if !page_readable(p) {
                continue;
            }
            let v = unsafe { std::ptr::read_unaligned(p as *const i32) };
            if v == value {
                hits.push(p);
            }
        }
        return hits;
    }
    for (base, size) in readable_regions() {
        if size < 4 || !page_readable(base) {
            continue;
        }
        let slice = unsafe { std::slice::from_raw_parts(base as *const u8, size) };
        let mut i = 0;
        while i + 4 <= slice.len() {
            let v = i32::from_le_bytes(slice[i..i + 4].try_into().unwrap());
            if v == value {
                hits.push(base + i);
                if hits.len() >= MAX_HITS {
                    return hits;
                }
            }
            i += 4;
        }
    }
    hits
}

fn load_cands(path: &PathBuf) -> Vec<usize> {
    fs::read_to_string(path)
        .ok()
        .map(|s| {
            s.lines()
                .filter_map(|l| {
                    let t = l.trim().trim_start_matches("0x");
                    usize::from_str_radix(t, 16).ok()
                })
                .collect()
        })
        .unwrap_or_default()
}

fn save_cands(path: &PathBuf, hits: &[usize]) {
    let body = hits
        .iter()
        .map(|p| format!("{p:X}"))
        .collect::<Vec<_>>()
        .join("\n");
    let _ = fs::write(path, body);
}

pub fn run(dir: Option<&PathBuf>, value: i32) -> String {
    let path = dir.map(|d| d.join("murk_cands.txt"));
    let prior = path.as_ref().map(load_cands).unwrap_or_default();
    let hits = if prior.is_empty() {
        collect(value, None)
    } else {
        collect(value, Some(&prior))
    };
    if let Some(path) = path.as_ref() {
        save_cands(path, &hits);
    }
    let shown = hits
        .iter()
        .take(16)
        .map(|p| format!("{p:X}"))
        .collect::<Vec<_>>()
        .join(", ");
    format!(
        "NRAP murk scan value={value} hits={} prior={} {}",
        hits.len(),
        prior.len(),
        if shown.is_empty() {
            String::from("(none)")
        } else {
            shown
        }
    )
}
