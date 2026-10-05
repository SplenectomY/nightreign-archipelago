use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::time::{SystemTime, UNIX_EPOCH};

const STEAM64_BASE: u64 = 7_656_119_796_026_5728;

pub fn stage(nrap_dir: &Path, new_seed: &str) -> Result<String, String> {
    let new_seed = new_seed.trim();
    if new_seed.is_empty() {
        return Err("Archipelago seed was missing".into());
    }
    let staged = nrap_dir.join("saves").join(".staged_seed");
    if staged.is_file() && fs::read_to_string(&staged).unwrap_or_default().trim() == new_seed {
        return Ok(format!("NRAP save already staged for seed {new_seed}"));
    }
    let marker = nrap_dir.join("current_save_seed.txt");
    let current = fs::read_to_string(&marker).unwrap_or_default();
    let current = current.trim();
    if marker.is_file() && current == new_seed {
        return Ok(format!("NRAP save already matches seed {new_seed}"));
    }
    let save_dir = nightreign_save_dir()?;
    let files = co2_files(&save_dir);
    if marker.is_file() {
        let dest = nrap_dir.join("saves").join(current);
        fs::create_dir_all(&dest).map_err(|e| e.to_string())?;
        let moved = move_saves(&files, &dest, false)?;
        let restored = restore_seed(nrap_dir, new_seed, &save_dir)?;
        let _ = fs::write(&staged, new_seed);
        Ok(format!(
            "NRAP save swapped {current} -> {new_seed} (moved {moved}, restored {restored}) from {}",
            save_dir.display()
        ))
    } else {
        let dest = nrap_dir.join("saves").join("old");
        fs::create_dir_all(&dest).map_err(|e| e.to_string())?;
        let moved = move_saves(&files, &dest, true)?;
        let _ = fs::write(&staged, new_seed);
        Ok(format!(
            "NRAP save archived {moved} file(s) to saves/old for new seed {new_seed} from {}",
            save_dir.display()
        ))
    }
}

pub fn commit(nrap_dir: &Path, new_seed: &str) -> Result<(), String> {
    fs::write(nrap_dir.join("current_save_seed.txt"), format!("{}\n", new_seed.trim())).map_err(|e| e.to_string())?;
    let _ = fs::remove_file(nrap_dir.join("saves").join(".staged_seed"));
    Ok(())
}

fn steam_ids() -> Vec<String> {
    let mut ids = Vec::new();
    let out = Command::new("reg")
        .args(["query", r"HKCU\Software\Valve\Steam\ActiveProcess", "/v", "ActiveUser"])
        .output();
    if let Ok(out) = out {
        let text = String::from_utf8_lossy(&out.stdout);
        if let Some(id) = text.split_whitespace().rev().find_map(|part| {
            if part.starts_with("0x") || part.starts_with("0X") {
                u64::from_str_radix(part.trim_start_matches("0x").trim_start_matches("0X"), 16).ok()
            } else {
                part.parse::<u64>().ok()
            }
        }) {
            if id > 0 {
                ids.push(id.to_string());
                ids.push((STEAM64_BASE + id).to_string());
            }
        }
    }
    ids
}

fn nightreign_save_dir() -> Result<PathBuf, String> {
    let appdata = std::env::var("APPDATA").map_err(|_| "APPDATA is not set".to_string())?;
    let ids = steam_ids();
    let mut found = Vec::new();
    for root in ["Nightreign", "ELDEN RING", "EldenRing"] {
        let base = PathBuf::from(&appdata).join(root);
        if !ids.is_empty() {
            for id in &ids {
                let dir = base.join(id);
                if dir.is_dir() {
                    found.push(dir);
                }
            }
        }
        if let Ok(read) = fs::read_dir(&base) {
            for entry in read.flatten() {
                let name = entry.file_name().to_string_lossy().to_string();
                if name.chars().all(|c| c.is_ascii_digit()) && entry.path().is_dir() {
                    found.push(entry.path());
                }
            }
        }
    }
    found.sort();
    found.dedup();
    found
        .into_iter()
        .max_by_key(|dir| co2_files(dir).len())
        .ok_or_else(|| "Nightreign save folder was not found under %APPDATA%\\Nightreign".to_string())
}

fn co2_files(dir: &Path) -> Vec<PathBuf> {
    let mut out = Vec::new();
    let Ok(read) = fs::read_dir(dir) else { return out };
    for entry in read.flatten() {
        let path = entry.path();
        let name = path.file_name().and_then(|n| n.to_str()).unwrap_or("").to_ascii_lowercase();
        if name.ends_with(".co2") || name.ends_with(".co2.bak") {
            out.push(path);
        }
    }
    out.sort();
    out
}

fn move_saves(files: &[PathBuf], dest: &Path, stamp: bool) -> Result<usize, String> {
    let stamp = if stamp { Some(timestamp()) } else { None };
    let mut n = 0;
    for src in files {
        let name = src.file_name().and_then(|n| n.to_str()).unwrap_or("save.co2");
        let named = match &stamp {
            Some(ts) => stamped(name, ts),
            None => name.to_string(),
        };
        let target = dest.join(named);
        if target.exists() {
            fs::remove_file(&target).map_err(|e| e.to_string())?;
        }
        fs::rename(src, &target).or_else(|_| {
            fs::copy(src, &target).map_err(|e| e.to_string())?;
            fs::remove_file(src).map_err(|e| e.to_string())
        })?;
        n += 1;
    }
    Ok(n)
}

fn restore_seed(nrap_dir: &Path, seed: &str, save_dir: &Path) -> Result<usize, String> {
    let src = nrap_dir.join("saves").join(seed);
    if !src.is_dir() {
        return Ok(0);
    }
    let mut n = 0;
    for file in co2_files(&src) {
        let name = file.file_name().and_then(|n| n.to_str()).unwrap_or("save.co2");
        let target = save_dir.join(name);
        if target.exists() {
            fs::remove_file(&target).map_err(|e| e.to_string())?;
        }
        fs::copy(&file, &target).map_err(|e| e.to_string())?;
        n += 1;
    }
    Ok(n)
}

fn stamped(name: &str, ts: &str) -> String {
    if let Some(dot) = name.find('.') {
        format!("{}-{ts}{}", &name[..dot], &name[dot..])
    } else {
        format!("{name}-{ts}")
    }
}

fn timestamp() -> String {
    let secs = SystemTime::now().duration_since(UNIX_EPOCH).map(|d| d.as_secs()).unwrap_or(0);
    let days = secs / 86400;
    let z = secs % 86400;
    let hour = z / 3600;
    let min = (z % 3600) / 60;
    let sec = z % 60;
    // Civil date from days since 1970-01-01, good enough for a filename.
    let mut y = 1970i32;
    let mut rem = days as i64;
    loop {
        let diy = if y % 4 == 0 && (y % 100 != 0 || y % 400 == 0) { 366 } else { 365 };
        if rem < diy { break; }
        rem -= diy;
        y += 1;
    }
    let md = [31, 28 + if y % 4 == 0 && (y % 100 != 0 || y % 400 == 0) { 1 } else { 0 }, 31, 30, 31, 30, 31, 31, 30, 31, 30, 31];
    let mut m = 1;
    for d in md {
        if rem < d { break; }
        rem -= d;
        m += 1;
    }
    format!("{y:04}-{m:02}-{day:02}-{hour:02}-{min:02}-{sec:02}", day = rem + 1)
}
