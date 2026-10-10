//! Blocking AP client. Own thread. Flag thread only pushes location ids.

use std::net::TcpStream;
use std::sync::atomic::{AtomicBool, AtomicI64, AtomicU32, AtomicU64, Ordering};
use std::time::{SystemTime, UNIX_EPOCH};
use std::sync::Mutex;
use std::sync::mpsc::{Receiver, RecvTimeoutError};
use std::time::Duration;
use tungstenite::protocol::WebSocket;
use tungstenite::stream::MaybeTlsStream;
use tungstenite::{connect as ws_connect, Message};

const GAME: &str = "Elden Ring Nightreign";
const ITEMS_HANDLING: u8 = 0b111;

pub static RECONNECT: AtomicBool = AtomicBool::new(false);
static BACKOFF: AtomicU64 = AtomicU64::new(1);
static SHOP_HINTS: Mutex<Vec<i64>> = Mutex::new(Vec::new());
static DEATH_LINK: AtomicBool = AtomicBool::new(false);
static DEATH_MODE: AtomicU32 = AtomicU32::new(0);
static DEATH_PERCENT: AtomicU32 = AtomicU32::new(50);
static DEATH_CHANCE: AtomicU32 = AtomicU32::new(50);
static LOCAL_DEATH: AtomicBool = AtomicBool::new(false);
static INCOMING_DEATH: AtomicBool = AtomicBool::new(false);
static LAST_SENT: AtomicU64 = AtomicU64::new(0);
static SUPPRESS_UNTIL: AtomicU64 = AtomicU64::new(0);
static SLOT_NAME: Mutex<String> = Mutex::new(String::new());

pub fn note_local_death() {
    if !DEATH_LINK.load(Ordering::SeqCst) {
        return;
    }
    let now = SystemTime::now().duration_since(UNIX_EPOCH).map(|d| d.as_secs()).unwrap_or(0);
    if now < SUPPRESS_UNTIL.load(Ordering::SeqCst) {
        return;
    }
    LOCAL_DEATH.store(true, Ordering::SeqCst);
}

pub fn suppress_local_death(secs: u64) {
    let now = SystemTime::now().duration_since(UNIX_EPOCH).map(|d| d.as_secs()).unwrap_or(0);
    SUPPRESS_UNTIL.store(now.saturating_add(secs), Ordering::SeqCst);
}

pub fn death_mode() -> u32 { DEATH_MODE.load(Ordering::SeqCst) }
pub fn death_percent() -> u32 { DEATH_PERCENT.load(Ordering::SeqCst).clamp(1, 100) }
pub fn death_chance() -> u32 { DEATH_CHANCE.load(Ordering::SeqCst).clamp(1, 100) }

pub fn death_pending() -> bool {
    INCOMING_DEATH.load(Ordering::SeqCst)
}

pub fn take_death() -> bool {
    INCOMING_DEATH.swap(false, Ordering::SeqCst)
}

pub fn force_local_death() {
    LOCAL_DEATH.store(true, Ordering::SeqCst);
}

pub fn force_incoming_death() {
    INCOMING_DEATH.store(true, Ordering::SeqCst);
}


pub fn request_reconnect() {
    RECONNECT.store(true, Ordering::SeqCst);
}

#[derive(Clone)]
pub struct ApConfig {
    pub host: String,
    pub slot: String,
    pub password: String,
}

impl ApConfig {
    pub fn from_toml(text: &str) -> Self {
        let mut host = "127.0.0.1:38281".to_string();
        let mut slot = "Player1".to_string();
        let mut password = String::new();
        let mut in_ap = false;
        for line in text.lines() {
            let line = line.trim();
            if line.starts_with('[') {
                in_ap = line == "[ap]";
                continue;
            }
            if !in_ap {
                continue;
            }
            if let Some((k, v)) = line.split_once('=') {
                let v = v.trim().trim_matches('"').to_string();
                match k.trim() {
                    "host" => host = v,
                    "slot" => slot = v,
                    "password" => password = v,
                    _ => {}
                }
            }
        }
        Self {
            host,
            slot,
            password,
        }
    }

    fn addr(&self) -> String {
        self.host
            .trim_start_matches("ws://")
            .trim_start_matches("wss://")
            .to_string()
    }
}

type Socket = WebSocket<MaybeTlsStream<TcpStream>>;

fn escape(s: &str) -> String {
    s.replace('\\', "\\\\").replace('"', "\\\"")
}

static ITEM_NAMES: Mutex<Vec<(i64, String)>> = Mutex::new(Vec::new());
static LOCATION_NAMES: Mutex<Vec<(i64, String)>> = Mutex::new(Vec::new());
static PACKAGE_GAMES: Mutex<Vec<String>> = Mutex::new(Vec::new());

pub fn item_name(id: i64) -> String {
    if let Some(name) = crate::names::item_label(id) {
        return name.to_string();
    }
    ITEM_NAMES.lock().unwrap().iter().find(|(item, _)| *item == id).map(|(_, name)| name.clone()).unwrap_or_else(|| format!("item {id}"))
}

fn location_name(id: i64) -> String {
    if let Some(name) = crate::names::location_label(id) {
        return name.to_string();
    }
    LOCATION_NAMES.lock().unwrap().iter().find(|(loc, _)| *loc == id).map(|(_, name)| name.clone()).unwrap_or_else(|| id.to_string())
}

fn remember_name(map: &Mutex<Vec<(i64, String)>>, id: i64, name: &str) {
    if name.is_empty() || id == 0 {
        return;
    }
    let mut names = map.lock().unwrap();
    if !names.iter().any(|(have, _)| *have == id) {
        names.push((id, name.to_string()));
    }
}

fn ingest_package(text: &str) {
    for (key, map) in [("item_name_to_id", &ITEM_NAMES), ("location_name_to_id", &LOCATION_NAMES)] {
        let mut from = 0usize;
        let needle = format!("\"{key}\":");
        while let Some(rel) = text[from..].find(&needle) {
            let start = from + rel + needle.len();
            let Some(brace) = text[start..].find('{') else { break };
            let mut depth = 0i32;
            let mut end = start + brace;
            for (i, c) in text[end..].char_indices() {
                if c == '{' { depth += 1; }
                if c == '}' {
                    depth -= 1;
                    if depth == 0 {
                        end += i;
                        break;
                    }
                }
            }
            let body = &text[start + brace..=end.min(text.len() - 1)];
            let mut at = 0usize;
            while let Some((name, next)) = json_string_at(&body[at..]) {
                let after = &body[at + next..];
                let Some(colon) = after.find(':') else { break };
                let num: String = after[colon + 1..].trim_start().chars().take_while(|c| c.is_ascii_digit() || *c == '-').collect();
                if let Ok(id) = num.parse::<i64>() {
                    remember_name(map, id, &name);
                }
                at += next;
            }
            from = end + 1;
        }
    }
}

fn note_games(games: &[String]) {
    let mut have = PACKAGE_GAMES.lock().unwrap();
    for game in games {
        if !game.is_empty() && !have.iter().any(|have| have == game) {
            have.push(game.clone());
        }
    }
}

fn games_in_package(text: &str) -> Vec<String> {
    let Some(at) = text.find("\"games\":") else { return Vec::new() };
    let rest = &text[at..];
    let Some(start) = rest.find('{') else { return Vec::new() };
    let chars: Vec<char> = rest[start + 1..].chars().collect();
    let mut games = Vec::new();
    let mut depth = 1i32;
    let mut i = 0usize;
    while i < chars.len() && depth > 0 {
        let c = chars[i];
        if c == '{' { depth += 1; i += 1; continue; }
        if c == '}' { depth -= 1; i += 1; continue; }
        if c == '"' && depth == 1 {
            let mut name = String::new();
            i += 1;
            while i < chars.len() {
                let ch = chars[i];
                i += 1;
                if ch == '\\' && i < chars.len() { name.push(chars[i]); i += 1; continue; }
                if ch == '"' { break; }
                name.push(ch);
            }
            while i < chars.len() && chars[i].is_whitespace() { i += 1; }
            if i < chars.len() && chars[i] == ':' && !name.is_empty() && !games.contains(&name) {
                games.push(name);
            }
            continue;
        }
        i += 1;
    }
    games
}

fn package_games(text: &str) -> Vec<String> {
    let Some(at) = text.find("datapackage_checksums") else { return Vec::new() };
    let rest = &text[at..at.saturating_add(4000).min(text.len())];
    let Some(start) = rest.find('{') else { return Vec::new() };
    let mut games = Vec::new();
    let mut from = start + 1;
    while let Some(q) = rest[from..].find('"') {
        let name_at = from + q + 1;
        let Some(q2) = rest[name_at..].find('"') else { break };
        let name = json_unescape(&rest[name_at..name_at + q2]);
        if !name.is_empty() && !games.contains(&name) {
            games.push(name);
        }
        let after = &rest[name_at + q2 + 1..];
        let Some(end) = after.find(',') .or_else(|| after.find('}')) else { break };
        if after[..end].contains('}') && !after[..end].contains(',') {
            break;
        }
        from = name_at + q2 + 1 + end;
    }
    games
}

fn datapackage_request(games: &[String]) -> String {
    let list = games.iter().map(|g| format!("\"{}\"", escape(g))).collect::<Vec<_>>().join(",");
    format!("[{{\"cmd\":\"GetDataPackage\",\"games\":[{list}]}}]")
}

fn package_dir() -> Option<std::path::PathBuf> {
    let dir = crate::flag_write::cache_dir()?.join("datapackages");
    let _ = std::fs::create_dir_all(&dir);
    if let Some(root) = crate::flag_write::cache_dir() {
        if let Ok(entries) = std::fs::read_dir(&root) {
            for entry in entries.flatten() {
                let name = entry.file_name().to_string_lossy().to_string();
                if name.starts_with("datapackage") {
                    let _ = std::fs::remove_file(entry.path());
                }
            }
        }
    }
    Some(dir)
}

fn names_path() -> Option<std::path::PathBuf> {
    let seed = crate::flag_write::cache_seed();
    if seed.is_empty() { return None; }
    package_dir().map(|dir| dir.join(format!("datapackage_v2_{seed}.txt")))
}

fn load_names() -> usize {
    let Some(path) = names_path() else { return 0 };
    let text = std::fs::read_to_string(path).unwrap_or_default();
    if text.is_empty() { return 0; }
    for line in text.lines() {
        if let Some(rest) = line.strip_prefix("#games\t") {
            note_games(&rest.split('\t').map(|s| s.to_string()).filter(|s| !s.is_empty()).collect::<Vec<_>>());
            continue;
        }
        let mut parts = line.splitn(3, '\t');
        let kind = parts.next().unwrap_or("");
        let id = parts.next().and_then(|s| s.parse::<i64>().ok());
        let name = parts.next().unwrap_or("");
        if let Some(id) = id {
            if kind == "i" { remember_name(&ITEM_NAMES, id, name); }
            if kind == "l" { remember_name(&LOCATION_NAMES, id, name); }
        }
    }
    ITEM_NAMES.lock().unwrap().len()
}

fn save_names() {
    let Some(path) = names_path() else { return };
    let mut body = String::new();
    let games = PACKAGE_GAMES.lock().unwrap().clone();
    if !games.is_empty() {
        body.push_str("#games");
        for game in &games {
            body.push('\t');
            body.push_str(game);
        }
        body.push('\n');
    }
    for (id, name) in ITEM_NAMES.lock().unwrap().iter() {
        body.push_str(&format!("i\t{id}\t{name}\n"));
    }
    for (id, name) in LOCATION_NAMES.lock().unwrap().iter() {
        body.push_str(&format!("l\t{id}\t{name}\n"));
    }
    let _ = std::fs::write(path, body);
}

fn parse_seed(text: &str) -> Option<String> {
    let key = "\"seed_name\":\"";
    let at = text.find(key)?;
    let rest = &text[at+key.len()..];
    let end = rest.find('"')?;
    Some(rest[..end].to_string())
}

fn parse_i64_after(hay: &str, key: &str) -> Option<i64> {
    let needle = format!("\"{key}\":");
    let i = hay.find(&needle)?;
    let rest = hay[i + needle.len()..].trim_start();
    let num: String = rest
        .chars()
        .take_while(|c| c.is_ascii_digit() || *c == '-')
        .collect();
    num.parse().ok()
}

static GOAL: AtomicBool = AtomicBool::new(false);
static TUTORIAL_MARGIT: AtomicBool = AtomicBool::new(true);
static ALWAYS_WYLDER_TUTORIAL: AtomicBool = AtomicBool::new(true);

pub fn always_wylder_tutorial() -> bool {
    ALWAYS_WYLDER_TUTORIAL.load(Ordering::SeqCst)
}

pub fn tutorial_margit() -> bool {
    TUTORIAL_MARGIT.load(Ordering::SeqCst)
}
static GOAL_SENT: AtomicBool = AtomicBool::new(false);

// 1 murk = 1,000,000 J. Withdraw only. Factorio is 1:1, Satisfactory is 1:1e3, Stardew gold is 1:1e7.
const MURK_JOULES: i64 = 1_000_000;
static ENERGY_POOL: AtomicI64 = AtomicI64::new(0);
static WITHDRAW: Mutex<Vec<i64>> = Mutex::new(Vec::new());
static WITHDRAW_PENDING: AtomicI64 = AtomicI64::new(0);

pub fn energy_pool_joules() -> i64 {
    ENERGY_POOL.load(Ordering::SeqCst)
}

pub fn queue_withdraw(murk: i64) {
    if murk > 0 {
        WITHDRAW.lock().unwrap().push(murk);
    }
}

fn pop_withdraw() -> Option<i64> {
    let mut q = WITHDRAW.lock().unwrap();
    if q.is_empty() { None } else { Some(q.remove(0)) }
}

fn in_hold() -> bool {
    crate::flag_write::in_session()
        && !crate::flag_write::in_expedition()
        && !crate::flag_write::in_tutorial()
}

fn send_energy_watch(socket: &mut Socket) -> Result<(), String> {
    socket
        .send(Message::Text("[{\"cmd\":\"SetNotify\",\"keys\":[\"EnergyLink\"]}]".into()))
        .map_err(|e| format!("SetNotify: {e}"))?;
    socket
        .send(Message::Text("[{\"cmd\":\"Get\",\"keys\":[\"EnergyLink\"]}]".into()))
        .map_err(|e| format!("Get EnergyLink: {e}"))
}

fn send_energy_deplete(socket: &mut Socket, joules: i64) -> Result<(), String> {
    let pkt = format!(
        "[{{\"cmd\":\"Set\",\"key\":\"EnergyLink\",\"want_reply\":true,\"operations\":[{{\"operation\":\"deplete\",\"value\":{joules}}}]}}]"
    );
    socket.send(Message::Text(pkt.into())).map_err(|e| format!("EnergyLink deplete: {e}"))
}

fn note_energy(text: &str, log: &impl Fn(&str)) {
    if !text.contains("EnergyLink") {
        return;
    }
    if text.contains("\"cmd\":\"Retrieved\"") || text.contains("\"cmd\": \"Retrieved\"") {
        if let Some(v) = parse_i64_after(text, "EnergyLink") {
            ENERGY_POOL.store(v.max(0), Ordering::SeqCst);
            log(&format!("NRAP energy pool {v} J ({} murk)", v / MURK_JOULES));
        }
        return;
    }
    if !(text.contains("\"cmd\":\"SetReply\"") || text.contains("\"cmd\": \"SetReply\"")) {
        return;
    }
    let value = parse_i64_after(text, "value");
    let original = parse_i64_after(text, "original_value");
    if let Some(v) = value {
        ENERGY_POOL.store(v.max(0), Ordering::SeqCst);
    }
    let pending = WITHDRAW_PENDING.swap(0, Ordering::SeqCst);
    if pending <= 0 {
        return;
    }
    let gained = match (original, value) {
        (Some(before), Some(after)) => (before - after).max(0),
        _ => 0,
    };
    let murk = (gained / MURK_JOULES) as i32;
    if murk <= 0 {
        log(&format!("NRAP withdraw empty pool={value:?} gained={gained} J"));
        return;
    }
    match crate::grant::grant_murk(murk) {
        Ok(detail) => log(&format!("NRAP withdraw +{murk} murk ({gained} J) {detail}")),
        Err(e) => log(&format!("NRAP withdraw +{murk} murk grant failed: {e}")),
    }
}

static COUNT_NEED: AtomicU32 = AtomicU32::new(0);

pub fn count_need() -> u32 {
    COUNT_NEED.load(Ordering::SeqCst)
}

static PLAYERS: Mutex<Vec<(i64, String)>> = Mutex::new(Vec::new());

fn player_name(id: i64) -> Option<String> {
    PLAYERS.lock().unwrap().iter().find(|(slot, _)| *slot == id).map(|(_, name)| name.clone())
}

fn object_i64(obj: &str, key: &str) -> Option<i64> {
    parse_i64_after(obj, key)
}

fn json_string_at(s: &str) -> Option<(String, usize)> {
    let bytes = s.as_bytes();
    let start = bytes.iter().position(|b| *b == b'"')?;
    let mut i = start + 1;
    let mut raw = String::new();
    while i < bytes.len() {
        if bytes[i] == b'\\' {
            raw.push('\\');
            i += 1;
            if i < bytes.len() {
                raw.push(bytes[i] as char);
                i += 1;
            }
            continue;
        }
        if bytes[i] == b'"' {
            return Some((json_unescape(&raw), i + 1));
        }
        raw.push(bytes[i] as char);
        i += 1;
    }
    None
}

fn object_str(obj: &str, key: &str) -> Option<String> {
    let needle = format!("\"{key}\":");
    let at = obj.find(&needle)?;
    json_string_at(&obj[at + needle.len()..]).map(|(s, _)| s)
}

fn remember_players(text: &str) {
    if !text.contains("Connected") {
        return;
    }
    if text.contains("\"shop_checks\":\"none\"") || text.contains("\"shop_checks\": \"none\"") {
        crate::flag_write::stock_shop(true);
    }
    if text.contains("\"tutorial_margit\":false") || text.contains("\"tutorial_margit\": false") {
        TUTORIAL_MARGIT.store(false, Ordering::SeqCst);
    }
    if text.contains("\"always_wylder_in_tutorial\":false") || text.contains("\"always_wylder_in_tutorial\": false") {
        ALWAYS_WYLDER_TUTORIAL.store(false, Ordering::SeqCst);
    }
    if text.contains("\"goal\":\"count\"") || text.contains("\"goal\": \"count\"") {
        if let Some(n) = parse_i64_after(text, "nightlord_count") {
            COUNT_NEED.store(n.max(1) as u32, Ordering::SeqCst);
        }
    }
    let mut players = Vec::new();
    if let Some(at) = text.find("\"players\":[") {
        let body = &text[at + 11..];
        let mut from = 0usize;
        while let Some(rel) = body[from..].find('{') {
            let start = from + rel;
            let Some(end_rel) = body[start..].find('}') else { break };
            let obj = &body[start..start + end_rel];
            if let Some(slot) = object_i64(obj, "slot") {
                let name = object_str(obj, "alias").or_else(|| object_str(obj, "name")).unwrap_or_else(|| format!("Player {slot}"));
                players.push((slot, name));
            }
            from = start + end_rel + 1;
            if players.len() > 32 { break; }
        }
    }
    if !players.is_empty() {
        *PLAYERS.lock().unwrap() = players;
    }
}

fn preview(text: &str) -> String {
    let t = text.replace('\n', " ");
    if t.len() <= 240 {
        t
    } else {
        format!("{}…", &t[..240])
    }
}

fn json_unescape(s: &str) -> String {
    let mut out = String::new();
    let mut chars = s.chars().peekable();
    while let Some(c) = chars.next() {
        if c == '\\' {
            match chars.next() {
                Some('n') => out.push('\n'),
                Some('r') => out.push('\r'),
                Some('t') => out.push('\t'),
                Some('"') => out.push('"'),
                Some('\\') => out.push('\\'),
                Some(other) => out.push(other),
                None => {}
            }
        } else {
            out.push(c);
        }
    }
    out
}

fn part_name(raw: &str, kind: &str) -> String {
    let Some(id) = raw.parse::<i64>().ok() else {
        return raw.to_string();
    };
    match kind {
        "item_id" => item_name(id),
        "location_id" => location_name(id),
        "player_id" => player_name(id).unwrap_or_else(|| raw.to_string()),
        _ => raw.to_string(),
    }
}

fn printjson_text(text: &str) -> Option<String> {
    if !text.contains("PrintJSON") {
        return None;
    }
    let Some(at) = text.find("\"data\":[") else { return None };
    let body = &text[at + 8..];
    let mut parts = Vec::new();
    let mut from = 0usize;
    while let Some(rel) = body[from..].find('{') {
        let start = from + rel;
        let Some(end_rel) = body[start..].find('}') else { break };
        let obj = &body[start..=start + end_rel];
        if let Some(raw) = object_str(obj, "text") {
            let kind = object_str(obj, "type").unwrap_or_default();
            parts.push(part_name(&raw, &kind));
        }
        from = start + end_rel + 1;
        if parts.len() > 24 || body[from..].trim_start().starts_with(']') {
            break;
        }
    }
    if parts.is_empty() {
        return None;
    }
    Some(parts.join(""))
}

fn json_bool(text: &str, key: &str) -> Option<bool> {
    let needle = format!("\"{key}\":");
    let at = text.find(&needle)?;
    let rest = text[at + needle.len()..].trim_start();
    if rest.starts_with("true") { Some(true) } else if rest.starts_with("false") { Some(false) } else { None }
}

fn json_u32(text: &str, key: &str) -> Option<u32> {
    let needle = format!("\"{key}\":");
    let at = text.find(&needle)?;
    let rest = text[at + needle.len()..].trim_start();
    rest.split(|c: char| !c.is_ascii_digit()).next()?.parse().ok()
}

fn json_str(text: &str, key: &str) -> Option<String> {
    let needle = format!("\"{key}\":");
    let at = text.find(&needle)?;
    let rest = text[at + needle.len()..].trim_start();
    let rest = rest.strip_prefix('"')?;
    let end = rest.find('"')?;
    Some(rest[..end].to_string())
}

fn apply_slot_data(text: &str, log: &impl Fn(&str)) {
    if !text.contains("slot_data") {
        return;
    }
    if let (Some(in_pool), Some(count)) = (json_bool(text, "heolstor_in_pool"), json_u32(text, "heolstor_unlock_count")) {
        crate::flag_write::configure_heolstor(in_pool, count);
    }
    if let Some(on) = json_bool(text, "death_link") {
        DEATH_LINK.store(on, Ordering::SeqCst);
        let mode = json_str(text, "death_link_mode").unwrap_or_else(|| json_u32(text, "death_link_mode").map(|n| n.to_string()).unwrap_or_else(|| "instant".into()));
        let mode_id = match mode.as_str() { "percent" | "1" => 1, "dice" | "2" => 2, _ => 0 };
        DEATH_MODE.store(mode_id, Ordering::SeqCst);
        if let Some(pct) = json_u32(text, "death_link_percent") { DEATH_PERCENT.store(pct.clamp(1, 100), Ordering::SeqCst); }
        if let Some(chance) = json_u32(text, "death_link_chance") { DEATH_CHANCE.store(chance.clamp(1, 100), Ordering::SeqCst); }
        log(&format!(
            "NRAP death link {} mode={mode} percent={} chance={}",
            if on { "on" } else { "off" },
            death_percent(),
            death_chance()
        ));
    }
    if let (Some(purse), Some(bundle), Some(coffer), Some(chest), Some(hoard)) = (
        json_u32(text, "murk_purse"),
        json_u32(text, "murk_bundle"),
        json_u32(text, "murk_coffer"),
        json_u32(text, "murk_chest"),
        json_u32(text, "murk_hoard"),
    ) {
        log(&crate::grant::configure(purse as i32, bundle as i32, coffer as i32, chest as i32, hoard as i32));
    }
}

fn handle_server_text(text: &str, log: &impl Fn(&str), next_index: &mut i64, drop_goods: i32) {
    note_energy(text, log);
    if text.contains("\"type\":\"ItemSend\"") || text.contains("\"type\": \"ItemSend\"") {
        if let Some(msg) = printjson_text(text) {
            let team = parse_i64_after(text, "team").unwrap_or(0) + 1;
            log(&format!("NRAP AP | (Team #{team}) {msg}"));
        }
    } else if let Some(msg) = printjson_text(text) {
        for line in msg.split('\n') {
            log(&format!("NRAP AP | {line}"));
        }
    } else {
        log(&format!("NRAP AP rx {}", preview(text)));
    }
    if text.contains("ConnectionRefused") {
        log(&format!("NRAP AP server refused: {text}"));
        return;
    }
    if text.contains("\"cmd\":\"Bounce\"") && text.contains("DeathLink") && DEATH_LINK.load(Ordering::SeqCst) {
        let source = object_str(text, "source").unwrap_or_default();
        let now = SystemTime::now().duration_since(UNIX_EPOCH).map(|d| d.as_secs()).unwrap_or(0);
        let mine = SLOT_NAME.lock().unwrap().clone();
        if !source.is_empty() && source != mine && now.saturating_sub(LAST_SENT.load(Ordering::SeqCst)) > 3 {
            INCOMING_DEATH.store(true, Ordering::SeqCst);
            log(&format!("NRAP death link from {source}"));
        }
    }
    remember_players(text);
    if text.contains("RoomInfo") {
        if let Some(seed) = parse_seed(&text) {
            log(&crate::flag_write::bind_seed(&seed));
        }
    }
    if text.contains("DataPackage") {
        note_games(&games_in_package(text));
        let before = ITEM_NAMES.lock().unwrap().len();
        ingest_package(text);
        let after = ITEM_NAMES.lock().unwrap().len();
        if after > before {
            save_names();
            log(&format!("NRAP AP datapackage items {} locations {} cached", after, LOCATION_NAMES.lock().unwrap().len()));
        }
    }
    apply_slot_data(text, log);
    if !text.contains("ReceivedItems") {
        return;
    }
    let index = parse_i64_after(text, "index").unwrap_or(0);
    let mut from = 0usize;
    let mut count = 0i64;
    while let Some(rel) = text[from..].find("\"item\":") {
        let at = from + rel;
        let Some(id) = parse_i64_after(&text[at..], "item") else {
            break;
        };
        let ap_index = index + count;
        log(&format!(
            "NRAP received {} ({id}) index {ap_index}",
            item_name(id)
        ));
        if crate::grant::is_starting_rune(id) {
            if let Some(msg) = crate::grant::note_starting_runes(
                &crate::flag_write::cache_seed(),
                ap_index,
                id,
                crate::flag_write::in_expedition(),
            ) {
                log(&msg);
            }
            count += 1;
            from = at + 8;
            continue;
        }
        if crate::flag_write::enqueue_item(ap_index, id) {
            log(&format!("NRAP grant queued {} ({id}) index {ap_index}", item_name(id)));
        } else {
            log(&format!("NRAP grant skip already {} ({id}) index {ap_index}", item_name(id)));
        }
        if let Some(loc) = shop_purchase(id) {
            queue_shop_hint(loc);
        }
        let _ = drop_goods;
        if id == 839_100_900 {
            GOAL.store(true, Ordering::SeqCst);
            log("NRAP goal met");
        }
        count += 1;
        from = at + 8;
    }
    if count == 0 {
        log(&format!("NRAP ReceivedItems index={index} (no item ids parsed)"));
    }
    *next_index = index + count.max(1);
}

fn read_text(socket: &mut Socket) -> Result<Option<String>, String> {
    match socket.read() {
        Ok(Message::Text(t)) => Ok(Some(t.to_string())),
        Ok(Message::Ping(p)) => {
            let _ = socket.send(Message::Pong(p));
            Ok(None)
        }
        Ok(Message::Pong(_)) | Ok(Message::Frame(_)) => Ok(None),
        Ok(Message::Binary(_)) => Ok(None),
        Ok(Message::Close(frame)) => Err(format!("close frame {frame:?}")),
        Err(e) => {
            let text = e.to_string();
            if text.contains("10060") || text.contains("10035") || text.contains("timed out") || text.contains("WouldBlock") {
                Ok(None)
            } else {
                Err(format!("read: {e}"))
            }
        }
    }
}

fn drain_server(
    socket: &mut Socket,
    log: &impl Fn(&str),
    next_index: &mut i64,
    drop_goods: i32,
) -> Result<(), String> {
    loop {
        match read_text(socket)? {
            Some(t) => handle_server_text(&t, log, next_index, drop_goods),
            None => return Ok(()),
        }
    }
}


fn tune(socket: &mut Socket, timeout: Duration) {
    let tcp = match socket.get_ref() {
        MaybeTlsStream::Plain(stream) => stream,
        MaybeTlsStream::NativeTls(stream) => stream.get_ref(),
        #[allow(unreachable_patterns)]
        _ => return,
    };
    tcp.set_read_timeout(Some(timeout)).ok();
    tcp.set_nodelay(true).ok();
}

fn connect_and_handshake(
    cfg: &ApConfig,
    log: &impl Fn(&str),
    next_index: &mut i64,
    drop_goods: i32,
) -> Result<Socket, String> {
    let mut addr = cfg.addr();
    if addr.starts_with("0.0.0.0") {
        addr = addr.replacen("0.0.0.0", "127.0.0.1", 1);
        log("NRAP AP 0.0.0.0 is a bind address; connecting to 127.0.0.1");
    }
    let host = addr.split(':').next().unwrap_or(&addr);
    let local = host == "localhost"
        || host == "127.0.0.1"
        || host.starts_with("127.")
        || host.starts_with("10.")
        || host.starts_with("192.168.")
        || host.starts_with("172.16.")
        || host.starts_with("172.17.")
        || host.starts_with("172.18.")
        || host.starts_with("172.19.")
        || host.starts_with("172.2")
        || host.starts_with("172.30.")
        || host.starts_with("172.31.")
        || host == "::1";
    let url = if local { format!("ws://{addr}") } else { format!("wss://{addr}") };
    let (mut socket, _) = ws_connect(&url).map_err(|e| format!("ws handshake {url}: {e}"))?;
    tune(&mut socket, Duration::from_secs(8));
    if let Ok(Some(room)) = read_text(&mut socket) {
        handle_server_text(&room, log, next_index, drop_goods);
        if let Some(dir) = package_dir() {
            let raw = dir.join("preflight_package.txt");
            if let Ok(text) = std::fs::read_to_string(&raw) {
                note_games(&games_in_package(&text));
                ingest_package(&text);
                save_names();
                let _ = std::fs::remove_file(&raw);
            }
        }
        let cached = load_names();
        let have = PACKAGE_GAMES.lock().unwrap().clone();
        let missing: Vec<String> = package_games(&room).into_iter().filter(|g| !have.iter().any(|h| h == g)).collect();
        if cached > 0 {
            log(&format!("NRAP AP datapackage cache loaded {cached}"));
        }
        if !missing.is_empty() {
            let _ = socket.send(Message::Text(datapackage_request(&missing).into()));
            log(&format!("NRAP AP requested datapackage for {}", missing.join(", ")));
        }
    }
    let connect = format!(
        "[{{\"cmd\":\"Connect\",\"password\":\"{}\",\"game\":\"{}\",\"name\":\"{}\",\"uuid\":\"\",\"version\":{{\"major\":0,\"minor\":5,\"build\":1,\"class\":\"Version\"}},\"items_handling\":{},\"tags\":[\"AP\",\"DeathLink\"],\"slot_data\":true}}]",
        escape(&cfg.password),
        GAME,
        escape(&cfg.slot),
        ITEMS_HANDLING
    );
    socket
        .send(Message::Text(connect.into()))
        .map_err(|e| format!("Connect send: {e}"))?;
    let mut reply = read_text(&mut socket)?.ok_or_else(|| "no Connect reply".to_string())?;
    if reply.contains("DataPackage") {
        handle_server_text(&reply, log, next_index, drop_goods);
        reply = read_text(&mut socket)?.ok_or_else(|| "no Connect reply".to_string())?;
    }
    if reply.contains("ConnectionRefused") {
        if let Some(reason) = hard_refusal(&reply) {
            return Err(format!("hold:{reason}: {reply}"));
        }
        return Err(format!("ConnectionRefused: {reply}"));
    }
    if !(reply.contains("Connected") || reply.contains("RoomInfo")) {
        return Err(format!("unexpected handshake: {reply}"));
    }
    handle_server_text(&reply, log, next_index, drop_goods);
    tune(&mut socket, Duration::from_millis(80));
    let _ = socket.send(Message::Text("[{\"cmd\":\"Sync\"}]".into()));
    if let Err(e) = send_energy_watch(&mut socket) {
        log(&format!("NRAP energy watch failed: {e}"));
    }
    Ok(socket)
}

fn hard_refusal(text: &str) -> Option<&'static str> {
    for reason in ["InvalidSlot", "InvalidPassword", "InvalidGame", "IncompatibleVersion"] {
        if text.contains(reason) {
            return Some(reason);
        }
    }
    None
}

fn flush_withdraw(socket: &mut Socket, log: &impl Fn(&str)) {
    if WITHDRAW_PENDING.load(Ordering::SeqCst) > 0 {
        return;
    }
    let Some(murk) = pop_withdraw() else { return };
    if !in_hold() {
        log("NRAP withdraw ignored, not in the Hold");
        return;
    }
    let joules = murk.saturating_mul(MURK_JOULES);
    match send_energy_deplete(socket, joules) {
        Ok(()) => {
            WITHDRAW_PENDING.store(joules, Ordering::SeqCst);
            log(&format!("NRAP withdraw requested {murk} murk ({joules} J)"));
        }
        Err(e) => log(&format!("NRAP withdraw failed: {e}")),
    }
}

fn send_goal(socket: &mut Socket) -> Result<bool, String> {
    if !GOAL.load(Ordering::SeqCst) || GOAL_SENT.load(Ordering::SeqCst) {
        return Ok(false);
    }
    socket
        .send(Message::Text("[{\"cmd\":\"StatusUpdate\",\"status\":30}]".into()))
        .map_err(|e| format!("StatusUpdate: {e}"))?;
    GOAL_SENT.store(true, Ordering::SeqCst);
    Ok(true)
}

fn send_say(socket: &mut Socket, text: &str) -> Result<(), String> {
    let text = text.replace('\\', "\\\\").replace('"', "\\\"");
    let pkt = format!("[{{\"cmd\":\"Say\",\"text\":\"{text}\"}}]");
    socket
        .send(Message::Text(pkt.into()))
        .map_err(|e| format!("Say: {e}"))
}

fn shop_purchase(item_id: i64) -> Option<i64> {
    if (839_100_600..=839_100_683).contains(&item_id) {
        return Some(item_id - 100_000);
    }
    if (839_100_800..=839_100_839).contains(&item_id) {
        return Some(item_id - 100_000 + 100);
    }
    None
}

fn send_scouts(socket: &mut Socket, ids: &[i64]) -> Result<(), String> {
    if ids.is_empty() {
        return Ok(());
    }
    let list = ids.iter().map(|id| id.to_string()).collect::<Vec<_>>().join(",");
    let pkt = format!("[{{\"cmd\":\"LocationScouts\",\"locations\":[{list}],\"create_as_hint\":2}}]");
    socket
        .send(Message::Text(pkt.into()))
        .map_err(|e| format!("LocationScouts: {e}"))
}


fn hint_file() -> Option<std::path::PathBuf> {
    let seed = crate::flag_write::cache_seed();
    if seed.is_empty() {
        return None;
    }
    crate::flag_write::cache_dir().map(|dir| dir.join(format!("shop_hints_{seed}.txt")))
}

fn read_hint_file() -> Vec<i64> {
    let Some(path) = hint_file() else { return Vec::new() };
    std::fs::read_to_string(path).unwrap_or_default().lines().filter_map(|line| line.trim().parse().ok()).collect()
}

fn write_hint_file(ids: &[i64]) {
    let Some(path) = hint_file() else { return };
    if ids.is_empty() {
        let _ = std::fs::remove_file(path);
        return;
    }
    let body = ids.iter().map(|id| id.to_string()).collect::<Vec<_>>().join("\n");
    let _ = std::fs::write(path, body + "\n");
}

fn queue_shop_hint(loc: i64) {
    let mut hints = SHOP_HINTS.lock().unwrap();
    for id in read_hint_file() {
        if !hints.contains(&id) {
            hints.push(id);
        }
    }
    if !hints.contains(&loc) {
        hints.push(loc);
    }
    write_hint_file(&hints);
}

fn flush_shop_hints(socket: &mut Socket, log: &impl Fn(&str)) {
    if !crate::flag_write::in_session() || crate::flag_write::in_expedition() {
        return;
    }
    let ids = {
        let mut hints = SHOP_HINTS.lock().unwrap();
        for id in read_hint_file() {
            if !hints.contains(&id) {
                hints.push(id);
            }
        }
        if hints.is_empty() {
            return;
        }
        hints.clone()
    };
    match send_scouts(socket, &ids) {
        Ok(()) => {
            SHOP_HINTS.lock().unwrap().retain(|id| !ids.contains(id));
            write_hint_file(&SHOP_HINTS.lock().unwrap());
            log(&format!(
                "NRAP AP shop hint {}",
                ids.iter().map(|id| id.to_string()).collect::<Vec<_>>().join(",")
            ));
        }
        Err(e) => log(&format!("NRAP AP shop hint failed: {e}")),
    }
}

fn send_checks(socket: &mut Socket, ids: &[i64]) -> Result<(), String> {
    if ids.is_empty() {
        return Ok(());
    }
    let list = ids
        .iter()
        .map(|id| id.to_string())
        .collect::<Vec<_>>()
        .join(",");
    let pkt = format!("[{{\"cmd\":\"LocationChecks\",\"locations\":[{list}]}}]");
    socket
        .send(Message::Text(pkt.into()))
        .map_err(|e| format!("LocationChecks: {e}"))
}


fn send_death(socket: &mut Socket, slot: &str) -> Result<(), String> {
    let now = SystemTime::now().duration_since(UNIX_EPOCH).map(|d| d.as_secs()).unwrap_or(0);
    LAST_SENT.store(now, Ordering::SeqCst);
    let cause = format!("{slot} was slain.");
    let pkt = format!(
        "[{{\"cmd\":\"Bounce\",\"tags\":[\"DeathLink\"],\"data\":{{\"time\":{now}.0,\"source\":\"{}\",\"cause\":\"{}\"}}}}]",
        escape(slot),
        escape(&cause)
    );
    socket.send(Message::Text(pkt.into())).map_err(|e| format!("DeathLink: {e}"))?;
    Ok(())
}

pub fn run(mut cfg: ApConfig, rx: Receiver<i64>, say_rx: Receiver<String>, drop_goods: i32, config_path: std::path::PathBuf, log: impl Fn(&str)) {
    *SLOT_NAME.lock().unwrap() = cfg.slot.clone();
    log(&format!("NRAP AP targeting {} slot {}", cfg.host, cfg.slot));
    let mut pending: Vec<i64> = Vec::new();
    let mut hold = false;
    loop {
        let trigger = config_path.parent().map(|p| p.join("reconnect.trigger"));
        let triggered = RECONNECT.swap(false, Ordering::SeqCst)
            || trigger.as_ref().is_some_and(|p| p.is_file());
        if hold && !triggered {
            match rx.recv_timeout(Duration::from_millis(250)) {
                Ok(id) => {
                    if !pending.contains(&id) {
                        pending.push(id);
                        log(&format!("NRAP AP queued {id} (offline)"));
                    }
                }
                Err(RecvTimeoutError::Timeout) => {}
                Err(RecvTimeoutError::Disconnected) => return,
            }
            continue;
        }
        if triggered {
            hold = false;
            if let Some(path) = &trigger { let _ = std::fs::remove_file(path); }
            if let Ok(text) = std::fs::read_to_string(&config_path) {
                cfg = ApConfig::from_toml(&text);
            }
            *SLOT_NAME.lock().unwrap() = cfg.slot.clone();
            log(&format!("NRAP AP reconnecting to {} slot {}", cfg.host, cfg.slot));
        }
        let mut next_index = 0i64;
        match connect_and_handshake(&cfg, &log, &mut next_index, drop_goods) {
            Ok(mut socket) => {
                log("NRAP AP connected");
                for _ in 0..20 {
                    if let Err(e) = drain_server(&mut socket, &log, &mut next_index, drop_goods) {
                        log(&format!("NRAP AP socket ended: {e}"));
                        break;
                    }
                    flush_shop_hints(&mut socket, &log);
                    std::thread::sleep(Duration::from_millis(50));
                }
                if let Err(e) = send_checks(&mut socket, &pending) {
                    log(&format!("NRAP AP resend failed: {e}"));
                } else if !pending.is_empty() {
                    log(&format!("NRAP AP resent {} cached checks", pending.len()));
                }
                loop {
                    while let Ok(line) = say_rx.try_recv() {
                        match send_say(&mut socket, &line) {
                            Ok(()) => log(&format!("NRAP AP say {line}")),
                            Err(e) => log(&format!("NRAP AP say failed: {e}")),
                        }
                    }
                    let trigger = config_path.parent().map(|p| p.join("reconnect.trigger"));
                    if RECONNECT.load(Ordering::SeqCst) || trigger.as_ref().is_some_and(|p| p.is_file()) {
                        log("NRAP AP disconnecting for reconnect");
                        let _ = socket.close(None);
                        break;
                    }
                    match rx.recv_timeout(Duration::from_millis(250)) {
                        Ok(id) => {
                            if !pending.contains(&id) {
                                pending.push(id);
                            }
                            if let Err(e) = send_checks(&mut socket, &[id]) {
                                log(&format!("NRAP AP send {id} failed: {e}"));
                                break;
                            }
                            log(&format!("NRAP AP LocationChecks {id}"));
                            if let Err(e) = drain_server(&mut socket, &log, &mut next_index, drop_goods) {
                                log(&format!("NRAP AP socket ended: {e}"));
                                break;
                            }
                            flush_shop_hints(&mut socket, &log);
                            if LOCAL_DEATH.swap(false, Ordering::SeqCst) {
                                match send_death(&mut socket, &cfg.slot) {
                                    Ok(()) => log("NRAP death link sent"),
                                    Err(e) => log(&format!("NRAP death link send failed: {e}")),
                                }
                            }
                            flush_withdraw(&mut socket, &log);
                            match send_goal(&mut socket) {
                                Ok(true) => log("NRAP AP goal sent"),
                                Ok(false) => {}
                                Err(e) => log(&format!("NRAP AP goal failed: {e}")),
                            }
                        }
                        Err(RecvTimeoutError::Timeout) => {
                            if let Err(e) = drain_server(&mut socket, &log, &mut next_index, drop_goods) {
                                log(&format!("NRAP AP socket ended: {e}"));
                                break;
                            }
                            flush_shop_hints(&mut socket, &log);
                            if LOCAL_DEATH.swap(false, Ordering::SeqCst) {
                                match send_death(&mut socket, &cfg.slot) {
                                    Ok(()) => log("NRAP death link sent"),
                                    Err(e) => log(&format!("NRAP death link send failed: {e}")),
                                }
                            }
                            flush_withdraw(&mut socket, &log);
                            match send_goal(&mut socket) {
                                Ok(true) => log("NRAP AP goal sent"),
                                Ok(false) => {}
                                Err(e) => log(&format!("NRAP AP goal failed: {e}")),
                            }
                        }
                        Err(RecvTimeoutError::Disconnected) => return,
                    }
                }
            }
            Err(e) => {
                let low = e.to_ascii_lowercase();
                let fatal = e.contains("InvalidSlot") || e.starts_with("hold:") || low.contains("no such host") || low.contains("name or service not known") || e.contains("os error 11001");
                if fatal {
                    hold = true;
                    log(&format!("NRAP AP reconnect cancelled: {e}"));
                } else if let Some(reason) = e.strip_prefix("hold:") {
                    hold = true;
                    log(&format!("NRAP AP disconnected: {reason}. Press Reconnect after fixing the slot."));
                } else {
                    log(&format!("NRAP AP not connected: {e}"));
                }
                let wait = if RECONNECT.load(Ordering::SeqCst) {
                    Duration::from_millis(200)
                } else if fatal {
                    Duration::from_secs(3600)
                } else if low.contains("timed out") || e.contains("os error 10060") {
                    let step = BACKOFF.fetch_update(Ordering::SeqCst, Ordering::SeqCst, |n| Some((n.saturating_mul(2)).clamp(1, 30))).unwrap_or(1);
                    log(&format!("NRAP AP timed out, retry in {step}s"));
                    Duration::from_secs(step)
                } else {
                    BACKOFF.store(1, Ordering::SeqCst);
                    Duration::from_secs(5)
                };
                let wait_until = std::time::Instant::now() + wait;
                while std::time::Instant::now() < wait_until {
                    match rx.recv_timeout(Duration::from_millis(250)) {
                        Ok(id) => {
                            if !pending.contains(&id) {
                                pending.push(id);
                                log(&format!("NRAP AP queued {id} (offline)"));
                            }
                        }
                        Err(RecvTimeoutError::Timeout) => {}
                        Err(RecvTimeoutError::Disconnected) => return,
                    }
                }
                continue;
            }
        }
        let wait_until = std::time::Instant::now() + Duration::from_secs(1);
        while std::time::Instant::now() < wait_until {
            match rx.recv_timeout(Duration::from_millis(250)) {
                Ok(id) => {
                    if !pending.contains(&id) {
                        pending.push(id);
                        log(&format!("NRAP AP queued {id} (offline)"));
                    }
                }
                Err(RecvTimeoutError::Timeout) => {}
                Err(RecvTimeoutError::Disconnected) => return,
            }
        }
    }
}
