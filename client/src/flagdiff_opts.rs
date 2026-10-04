//! flagdiff_options: range plus explicit exclusions. Missing ids stay included.

use std::collections::HashSet;
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::Mutex;
use std::time::SystemTime;

struct Filter {
    min: u32,
    max: u32,
    excluded: HashSet<u32>,
    stamp: Option<SystemTime>,
    path: PathBuf,
}

static FILTER: Mutex<Option<Filter>> = Mutex::new(None);

fn parse(text: &str) -> (u32, u32, HashSet<u32>) {
    let mut min = 1u32;
    let mut max = 10000u32;
    let mut excluded = HashSet::new();
    for line in text.lines() {
        let line = line.trim();
        if line.is_empty() || line.starts_with('#') {
            continue;
        }
        let mut parts = line.splitn(3, '\t');
        let key = parts.next().unwrap_or("");
        let second = parts.next().unwrap_or("");
        if key == "min" {
            min = second.parse().unwrap_or(1);
            continue;
        }
        if key == "max" {
            max = second.parse().unwrap_or(10000);
            continue;
        }
        let Ok(id) = key.parse::<u32>() else { continue };
        if second == "0" {
            excluded.insert(id);
        }
    }
    (min, max, excluded)
}

pub fn reload(dir: &Path) {
    let path = dir.join("flagdiff_options");
    let stamp = fs::metadata(&path).and_then(|m| m.modified()).ok();
    let mut slot = FILTER.lock().unwrap();
    if let Some(cur) = slot.as_ref() {
        if cur.path == path && cur.stamp == stamp {
            return;
        }
    }
    let text = fs::read_to_string(&path).unwrap_or_default();
    let (min, max, excluded) = if text.is_empty() { (1, 10000, HashSet::new()) } else { parse(&text) };
    *slot = Some(Filter { min, max, excluded, stamp, path });
}

pub fn allows(id: u32) -> bool {
    let slot = FILTER.lock().unwrap();
    let Some(f) = slot.as_ref() else { return (1..=10000).contains(&id) };
    id >= f.min && id <= f.max && !f.excluded.contains(&id)
}
