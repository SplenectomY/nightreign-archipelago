//! Pattern scan inside nightreign.exe.

#![cfg(windows)]

use std::ffi::c_void;

pub type HMODULE = *mut c_void;

#[repr(C)]
struct ModuleInfo {
    lp_base_of_dll: *mut c_void,
    size_of_image: u32,
    entry_point: *mut c_void,
}

#[link(name = "kernel32")]
extern "system" {
    fn GetModuleHandleA(name: *const u8) -> HMODULE;
    fn GetCurrentProcess() -> *mut c_void;
}

#[link(name = "psapi")]
extern "system" {
    fn GetModuleInformation(
        process: *mut c_void,
        module: HMODULE,
        info: *mut ModuleInfo,
        cb: u32,
    ) -> i32;
}

#[derive(Clone, Copy)]
pub struct ModuleSpan {
    pub base: usize,
    pub size: usize,
}

impl ModuleSpan {
    pub fn nightreign() -> Option<Self> {
        unsafe {
            let module = GetModuleHandleA(std::ptr::null());
            if module.is_null() {
                return None;
            }
            let mut info = ModuleInfo {
                lp_base_of_dll: std::ptr::null_mut(),
                size_of_image: 0,
                entry_point: std::ptr::null_mut(),
            };
            if GetModuleInformation(
                GetCurrentProcess(),
                module,
                &mut info,
                std::mem::size_of::<ModuleInfo>() as u32,
            ) == 0
            {
                return None;
            }
            Some(Self {
                base: info.lp_base_of_dll as usize,
                size: info.size_of_image as usize,
            })
        }
    }

    pub fn slice(&self) -> &[u8] {
        unsafe { std::slice::from_raw_parts(self.base as *const u8, self.size) }
    }
}

/// `??` / `?` is a wildcard nibble-pair.
pub fn find_pattern(hay: &[u8], pat: &str) -> Option<usize> {
    let mut bytes: Vec<Option<u8>> = Vec::new();
    for tok in pat.split_whitespace() {
        if tok == "??" || tok == "?" {
            bytes.push(None);
        } else {
            bytes.push(Some(u8::from_str_radix(tok, 16).ok()?));
        }
    }
    if bytes.is_empty() || hay.len() < bytes.len() {
        return None;
    }
    'outer: for i in 0..=hay.len() - bytes.len() {
        for (j, b) in bytes.iter().enumerate() {
            if let Some(want) = *b {
                if hay[i + j] != want {
                    continue 'outer;
                }
            }
        }
        return Some(i);
    }
    None
}

/// Resolve `lea/mov/cmp rip+disp32` at `instr + offset_to_disp`.
pub fn rip_rel(span: ModuleSpan, instr_off: usize, disp_at: usize, next_at: usize) -> Option<usize> {
    let hay = span.slice();
    if instr_off + next_at > hay.len() {
        return None;
    }
    let disp = i32::from_le_bytes(hay[instr_off + disp_at..instr_off + disp_at + 4].try_into().ok()?);
    Some((span.base + instr_off + next_at).wrapping_add(disp as isize as usize))
}

pub fn find_ascii(hay: &[u8], needle: &[u8]) -> Option<usize> {
    hay.windows(needle.len()).position(|w| w == needle)
}
