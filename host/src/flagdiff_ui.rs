//! Flagdiff range and catalog editor. Writes flagdiff_options next to nrap.exe.

use std::ffi::c_void;
use std::fs;
use std::path::{Path, PathBuf};

type HWND = *mut c_void;
type HINSTANCE = *mut c_void;

const WS_OVERLAPPEDWINDOW: u32 = 0x00CF0000;
const WS_VISIBLE: u32 = 0x10000000;
const WS_CHILD: u32 = 0x40000000;
const WS_VSCROLL: u32 = 0x00200000;
const WS_TABSTOP: u32 = 0x00010000;
const WS_EX_CLIENTEDGE: u32 = 0x00000200;
const LVS_REPORT: u32 = 0x0001;
const LVS_SHOWSELALWAYS: u32 = 0x0008;
const WM_COMMAND: u32 = 0x0111;
const WM_NOTIFY: u32 = 0x004E;
const WM_DESTROY: u32 = 0x0002;
const WM_KEYDOWN: u32 = 0x0100;
const EN_KILLFOCUS: u16 = 0x0200;
const LVCF_WIDTH: u32 = 0x0002;
const LVCF_TEXT: u32 = 0x0004;
const LVIF_TEXT: u32 = 0x0001;
const LVM_FIRST: u32 = 0x1000;
const LVM_INSERTCOLUMNW: u32 = LVM_FIRST + 97;
const LVM_INSERTITEMW: u32 = LVM_FIRST + 77;
const LVM_SETITEMW: u32 = LVM_FIRST + 76;
const LVM_DELETEITEM: u32 = LVM_FIRST + 8;
const LVM_GETITEMCOUNT: u32 = LVM_FIRST + 4;
const LVM_SETEXTENDEDLISTVIEWSTYLE: u32 = LVM_FIRST + 54;
const LVM_GETITEMRECT: u32 = LVM_FIRST + 14;
const LVM_GETCOLUMNWIDTH: u32 = LVM_FIRST + 29;
const LVN_ITEMCHANGING: isize = -100;
const LVS_EX_FULLROWSELECT: usize = 0x20;
const LVS_EX_DOUBLEBUFFER: usize = 0x10000;
const LVS_EX_CHECKBOXES: usize = 0x4;
const LVM_SETITEMSTATE: u32 = LVM_FIRST + 43;
const LVM_GETITEMSTATE: u32 = LVM_FIRST + 44;
const NM_CLICK: isize = -2;
const LVN_ITEMCHANGED: isize = -101;

struct Row { id: u32, on: bool, desc: String }

struct Win {
    dir: PathBuf,
    min: HWND,
    max: HWND,
    list: HWND,
    add_id: HWND,
    add_desc: HWND,
    rows: Vec<Row>,
    edit: HWND,
    edit_item: i32,
}

static mut WIN: *mut Win = std::ptr::null_mut();
static mut EDIT_OLD: usize = 0;

#[link(name = "user32")]
extern "system" {
    fn RegisterClassW(class: *const WndClass) -> u16;
    fn CreateWindowExW(ex: u32, class: *const u16, title: *const u16, style: u32, x: i32, y: i32, w: i32, h: i32, parent: HWND, menu: isize, instance: HINSTANCE, param: *mut c_void) -> HWND;
    fn DefWindowProcW(hwnd: HWND, msg: u32, w: usize, l: isize) -> isize;
    fn SendMessageW(hwnd: HWND, msg: u32, w: usize, l: isize) -> isize;
    fn GetKeyState(key: i32) -> i16;
    fn SetWindowTextW(hwnd: HWND, text: *const u16) -> i32;
    fn GetWindowTextW(hwnd: HWND, buf: *mut u16, max: i32) -> i32;
    fn DestroyWindow(hwnd: HWND) -> i32;
    fn MessageBoxW(hwnd: HWND, text: *const u16, title: *const u16, flags: u32) -> i32;
    fn SetFocus(hwnd: HWND) -> HWND;
    fn CallWindowProcW(prev: usize, hwnd: HWND, msg: u32, w: usize, l: isize) -> isize;
    fn SetWindowLongPtrW(hwnd: HWND, index: i32, value: usize) -> usize;
    fn GetWindowRect(hwnd: HWND, rect: *mut i32) -> i32;
    fn ScreenToClient(hwnd: HWND, pt: *mut i32) -> i32;
}
#[link(name = "comctl32")]
extern "system" {
    fn InitCommonControlsEx(icc: *const [u32; 2]) -> i32;
}

#[repr(C)]
struct WndClass {
    style: u32,
    wnd_proc: Option<unsafe extern "system" fn(HWND, u32, usize, isize) -> isize>,
    cls_extra: i32,
    wnd_extra: i32,
    instance: HINSTANCE,
    icon: *mut c_void,
    cursor: *mut c_void,
    background: *mut c_void,
    menu_name: *const u16,
    class_name: *const u16,
}

#[repr(C)]
struct LvCol { mask: u32, fmt: i32, cx: i32, text: *mut u16, text_max: i32, sub: i32, image: i32, order: i32, min: i32, def: i32, ideal: i32 }
#[repr(C)]
struct LvItem { mask: u32, item: i32, sub: i32, state: u32, state_mask: u32, text: *mut u16, text_max: i32, image: i32, param: isize, indent: i32, group: i32, cols: u32, pcols: *mut i32, fmt: *mut i32, igroup: i32 }

fn wide(s: &str) -> Vec<u16> { s.encode_utf16().chain(std::iter::once(0)).collect() }

fn text_of(hwnd: HWND) -> String {
    let mut buf = vec![0u16; 512];
    let n = unsafe { GetWindowTextW(hwnd, buf.as_mut_ptr(), buf.len() as i32) };
    String::from_utf16_lossy(&buf[..n.max(0) as usize])
}

fn set_text(hwnd: HWND, s: &str) {
    let w = wide(s);
    unsafe { SetWindowTextW(hwnd, w.as_ptr()); }
}

fn load(dir: &Path) -> (u32, u32, Vec<Row>) {
    let path = dir.join("flagdiff_options");
    let text = fs::read_to_string(&path).unwrap_or_else(|_| include_str!("../../data/flagdiff_options").to_string());
    if !path.exists() {
        let _ = fs::write(&path, &text);
    }
    let mut min = 1u32;
    let mut max = 10000u32;
    let mut rows = Vec::new();
    for line in text.lines() {
        let line = line.trim();
        if line.is_empty() || line.starts_with('#') { continue; }
        let mut parts = line.splitn(3, '\t');
        let a = parts.next().unwrap_or("");
        let b = parts.next().unwrap_or("");
        let c = parts.next().unwrap_or("");
        if a == "min" { min = b.parse().unwrap_or(1); continue; }
        if a == "max" { max = b.parse().unwrap_or(10000); continue; }
        if let Ok(id) = a.parse::<u32>() {
            rows.push(Row { id, on: b != "0", desc: c.to_string() });
        }
    }
    rows.sort_by_key(|r| r.id);
    (min, max, rows)
}

fn save(win: &Win) {
    let min = text_of(win.min).parse::<u32>().unwrap_or(1);
    let max = text_of(win.max).parse::<u32>().unwrap_or(10000);
    let mut out = String::from("# NRAP flagdiff options. Tab-separated. min/max are inclusive.\n# A row with on=0 is excluded. Flags with no row are included when inside the range.\n");
    out.push_str(&format!("min\t{min}\nmax\t{max}\n"));
    for (i, row) in win.rows.iter().enumerate() {
        let on = checked(win.list, i as i32);
        let desc = row.desc.replace(['\t', '\n', '\r'], " ");
        out.push_str(&format!("{}\t{}\t{desc}\n", row.id, if on { "1" } else { "0" }));
    }
    let path = win.dir.join("flagdiff_options");
    let _ = fs::write(&path, out);
}

fn checked(list: HWND, item: i32) -> bool {
    let state = unsafe { SendMessageW(list, LVM_GETITEMSTATE, item as usize, 0xF000) } as u32;
    (state >> 12) == 2
}

fn set_check(list: HWND, item: i32, on: bool) {
    let mut it = LvItem { mask: 8, item, sub: 0, state: if on { 2 } else { 1 } << 12, state_mask: 0xF000, text: std::ptr::null_mut(), text_max: 0, image: 0, param: 0, indent: 0, group: 0, cols: 0, pcols: std::ptr::null_mut(), fmt: std::ptr::null_mut(), igroup: 0 };
    unsafe { SendMessageW(list, LVM_SETITEMSTATE, item as usize, &mut it as *mut _ as isize); }
}

fn set_cell(list: HWND, item: i32, sub: i32, text: &str) {
    let mut w = wide(text);
    let it = LvItem { mask: LVIF_TEXT, item, sub, state: 0, state_mask: 0, text: w.as_mut_ptr(), text_max: w.len() as i32, image: 0, param: 0, indent: 0, group: 0, cols: 0, pcols: std::ptr::null_mut(), fmt: std::ptr::null_mut(), igroup: 0 };
    unsafe { SendMessageW(list, LVM_SETITEMW, 0, &it as *const _ as isize); }
}

fn fill(list: HWND, rows: &[Row]) {
    for (i, row) in rows.iter().enumerate() {
        let mut id = wide(&row.id.to_string());
        let it = LvItem { mask: LVIF_TEXT, item: i as i32, sub: 0, state: 0, state_mask: 0, text: id.as_mut_ptr(), text_max: id.len() as i32, image: 0, param: 0, indent: 0, group: 0, cols: 0, pcols: std::ptr::null_mut(), fmt: std::ptr::null_mut(), igroup: 0 };
        unsafe { SendMessageW(list, LVM_INSERTITEMW, 0, &it as *const _ as isize); }
        set_cell(list, i as i32, 1, &row.desc);
        set_check(list, i as i32, row.on);
    }
}

fn desc_rect(list: HWND, item: i32) -> (i32, i32, i32, i32) {
    let mut row = [0i32, 0, 0, 0];
    unsafe { SendMessageW(list, LVM_GETITEMRECT, item as usize, row.as_mut_ptr() as isize); }
    let flag_w = unsafe { SendMessageW(list, LVM_GETCOLUMNWIDTH, 0, 0) } as i32;
    let desc_w = unsafe { SendMessageW(list, LVM_GETCOLUMNWIDTH, 1, 0) } as i32;
    (flag_w.max(0), row[1], desc_w.max(80), (row[3] - row[1]).max(18))
}

fn shift_down() -> bool {
    let state = unsafe { GetKeyState(0x10) };
    state < 0
}

fn begin_edit(item: i32) {
    unsafe {
        if WIN.is_null() || item < 0 || shift_down() { return; }
        let win = &mut *WIN;
        if !win.edit.is_null() { return; }
        let (x, y, w, h) = desc_rect(win.list, item);
        let edit = CreateWindowExW(WS_EX_CLIENTEDGE, wide("EDIT").as_ptr(), wide("").as_ptr(), WS_CHILD | WS_VISIBLE | WS_TABSTOP, x, y, w, h, win.list, 1, std::ptr::null_mut(), std::ptr::null_mut());
        set_text(edit, &win.rows[item as usize].desc);
        EDIT_OLD = SetWindowLongPtrW(edit, -4, edit_proc as usize);
        win.edit = edit;
        win.edit_item = item;
        SetFocus(edit);
    }
}

fn commit_edit() {
    unsafe {
        if WIN.is_null() { return; }
        let win = &mut *WIN;
        if win.edit.is_null() { return; }
        let item = win.edit_item;
        let desc = text_of(win.edit).replace(['\t', '\n', '\r'], " ");
        if item >= 0 && (item as usize) < win.rows.len() {
            win.rows[item as usize].desc = desc.clone();
            set_cell(win.list, item, 1, &desc);
        }
        DestroyWindow(win.edit);
        win.edit = std::ptr::null_mut();
        save(win);
    }
}

unsafe extern "system" fn edit_proc(hwnd: HWND, msg: u32, w: usize, l: isize) -> isize {
    if msg == WM_KEYDOWN && w == 0x0D { commit_edit(); return 0; }
    if msg == WM_KEYDOWN && w == 0x1B {
        if !WIN.is_null() { DestroyWindow((*WIN).edit); (*WIN).edit = std::ptr::null_mut(); }
        return 0;
    }
    CallWindowProcW(EDIT_OLD, hwnd, msg, w, l)
}

unsafe extern "system" fn proc(hwnd: HWND, msg: u32, w: usize, l: isize) -> isize {
    match msg {
        WM_COMMAND => {
            let id = (w & 0xffff) as isize;
            let note = (w >> 16) as u16;
            if id == 3 { add_row(hwnd); }
            if id == 4 { if !WIN.is_null() { commit_edit(); save(&*WIN); } DestroyWindow(hwnd); }
            let _ = note;
            0
        }
        WM_NOTIFY => {
            let code = std::ptr::read_unaligned((l as *const u8).add(16) as *const i32) as isize;
            if !WIN.is_null() && (code == NM_CLICK || code == LVN_ITEMCHANGED || code == LVN_ITEMCHANGING) {
                let item = std::ptr::read_unaligned((l as *const u8).add(24) as *const i32);
                let sub = std::ptr::read_unaligned((l as *const u8).add(28) as *const i32);
                let new_state = std::ptr::read_unaligned((l as *const u8).add(32) as *const u32);
                let editing = !(*WIN).edit.is_null();
                if editing && code == LVN_ITEMCHANGING && item != (*WIN).edit_item && new_state & 2 != 0 {
                    return 1;
                }
                if editing && code == NM_CLICK {
                    SetFocus((*WIN).edit);
                    return 1;
                }
                if code == NM_CLICK && sub == 1 && !shift_down() { begin_edit(item); }
                if code == LVN_ITEMCHANGED { save(&*WIN); }
            }
            0
        }
        WM_DESTROY => {
            if !WIN.is_null() { save(&*WIN); let _ = Box::from_raw(WIN); WIN = std::ptr::null_mut(); }
            0
        }
        _ => DefWindowProcW(hwnd, msg, w, l),
    }
}

fn add_row(owner: HWND) {
    unsafe {
        if WIN.is_null() { return; }
        let win = &mut *WIN;
        let id: u32 = match text_of(win.add_id).trim().parse() {
            Ok(v) => v,
            Err(_) => { MessageBoxW(owner, wide("Flag ID must be a number.").as_ptr(), wide("Flagdiff").as_ptr(), 0x30); return; }
        };
        if win.rows.iter().any(|r| r.id == id) {
            MessageBoxW(owner, wide(&format!("Flag {id} already exists.")).as_ptr(), wide("Flagdiff").as_ptr(), 0x30);
            return;
        }
        let desc = text_of(win.add_desc).replace(['\t', '\n', '\r'], " ");
        let at = win.rows.iter().position(|r| r.id > id).unwrap_or(win.rows.len());
        win.rows.insert(at, Row { id, on: true, desc: desc.clone() });
        // Rebuild so the inserted row stays sorted. 5k items is a one-shot cost.
        SendMessageW(win.list, 0x1009, 0, 0); // LVM_DELETEALLITEMS
        fill(win.list, &win.rows);
        set_text(win.add_id, "");
        set_text(win.add_desc, "");
        save(win);
    }
}

pub fn open(dir: PathBuf) {
    unsafe {
        if !WIN.is_null() { return; }
        InitCommonControlsEx(&[8, 1]);
        let (min, max, rows) = load(&dir);
        let class = wide("NRAPFlagdiff");
        let wc = WndClass { style: 0, wnd_proc: Some(proc), cls_extra: 0, wnd_extra: 0, instance: std::ptr::null_mut(), icon: std::ptr::null_mut(), cursor: std::ptr::null_mut(), background: 16 as *mut c_void, menu_name: std::ptr::null(), class_name: class.as_ptr() };
        RegisterClassW(&wc);
        let win = CreateWindowExW(0, class.as_ptr(), wide("Flagdiff options").as_ptr(), WS_OVERLAPPEDWINDOW | WS_VISIBLE, 180, 60, 920, 760, std::ptr::null_mut(), 0, std::ptr::null_mut(), std::ptr::null_mut());
        CreateWindowExW(0, wide("STATIC").as_ptr(), wide("Range min").as_ptr(), WS_CHILD | WS_VISIBLE, 16, 16, 80, 20, win, 0, std::ptr::null_mut(), std::ptr::null_mut());
        let min_e = CreateWindowExW(WS_EX_CLIENTEDGE, wide("EDIT").as_ptr(), wide("").as_ptr(), WS_CHILD | WS_VISIBLE | WS_TABSTOP, 100, 12, 100, 24, win, 10, std::ptr::null_mut(), std::ptr::null_mut());
        CreateWindowExW(0, wide("STATIC").as_ptr(), wide("Range max").as_ptr(), WS_CHILD | WS_VISIBLE, 220, 16, 80, 20, win, 0, std::ptr::null_mut(), std::ptr::null_mut());
        let max_e = CreateWindowExW(WS_EX_CLIENTEDGE, wide("EDIT").as_ptr(), wide("").as_ptr(), WS_CHILD | WS_VISIBLE | WS_TABSTOP, 304, 12, 120, 24, win, 11, std::ptr::null_mut(), std::ptr::null_mut());
        set_text(min_e, &min.to_string());
        set_text(max_e, &max.to_string());
        CreateWindowExW(0, wide("STATIC").as_ptr(), wide("Unchecked rows are excluded. Flags not listed are included when inside the range.").as_ptr(), WS_CHILD | WS_VISIBLE, 440, 16, 450, 20, win, 0, std::ptr::null_mut(), std::ptr::null_mut());
        let list = CreateWindowExW(WS_EX_CLIENTEDGE, wide("SysListView32").as_ptr(), wide("").as_ptr(), WS_CHILD | WS_VISIBLE | WS_TABSTOP | WS_VSCROLL | LVS_REPORT | LVS_SHOWSELALWAYS, 16, 48, 872, 560, win, 2, std::ptr::null_mut(), std::ptr::null_mut());
        SendMessageW(list, LVM_SETEXTENDEDLISTVIEWSTYLE, 0, (LVS_EX_FULLROWSELECT | LVS_EX_DOUBLEBUFFER | LVS_EX_CHECKBOXES) as isize);
        for (i, (title, width)) in ["Flag", "Description"].iter().zip([90, 760]).enumerate() {
            let mut t = wide(title);
            let col = LvCol { mask: LVCF_TEXT | LVCF_WIDTH, fmt: 0, cx: width, text: t.as_mut_ptr(), text_max: t.len() as i32, sub: i as i32, image: 0, order: 0, min: 0, def: 0, ideal: 0 };
            SendMessageW(list, LVM_INSERTCOLUMNW, i, &col as *const _ as isize);
        }
        // ID lives in column 0 text beside the checkbox. Description is column 1.
        fill(list, &rows);
        CreateWindowExW(0, wide("STATIC").as_ptr(), wide("New flag ID").as_ptr(), WS_CHILD | WS_VISIBLE, 16, 622, 90, 20, win, 0, std::ptr::null_mut(), std::ptr::null_mut());
        let add_id = CreateWindowExW(WS_EX_CLIENTEDGE, wide("EDIT").as_ptr(), wide("").as_ptr(), WS_CHILD | WS_VISIBLE | WS_TABSTOP, 110, 618, 100, 24, win, 12, std::ptr::null_mut(), std::ptr::null_mut());
        CreateWindowExW(0, wide("STATIC").as_ptr(), wide("Description").as_ptr(), WS_CHILD | WS_VISIBLE, 224, 622, 90, 20, win, 0, std::ptr::null_mut(), std::ptr::null_mut());
        let add_desc = CreateWindowExW(WS_EX_CLIENTEDGE, wide("EDIT").as_ptr(), wide("").as_ptr(), WS_CHILD | WS_VISIBLE | WS_TABSTOP, 318, 618, 420, 24, win, 13, std::ptr::null_mut(), std::ptr::null_mut());
        CreateWindowExW(0, wide("BUTTON").as_ptr(), wide("Add flag").as_ptr(), WS_CHILD | WS_VISIBLE | WS_TABSTOP, 752, 616, 136, 28, win, 3, std::ptr::null_mut(), std::ptr::null_mut());
        CreateWindowExW(0, wide("BUTTON").as_ptr(), wide("Close").as_ptr(), WS_CHILD | WS_VISIBLE | WS_TABSTOP, 360, 668, 200, 32, win, 4, std::ptr::null_mut(), std::ptr::null_mut());
        WIN = Box::into_raw(Box::new(Win { dir, min: min_e, max: max_e, list, add_id, add_desc, rows, edit: std::ptr::null_mut(), edit_item: -1 }));
    }
}
