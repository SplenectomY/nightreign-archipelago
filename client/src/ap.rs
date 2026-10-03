//! Blocking AP client. Own thread. Flag thread only pushes location ids.

use std::net::TcpStream;
use std::sync::atomic::{AtomicBool, AtomicU32, Ordering};
use std::sync::Mutex;
use std::sync::mpsc::{Receiver, RecvTimeoutError};
use std::time::Duration;
use tungstenite::protocol::WebSocket;
use tungstenite::stream::MaybeTlsStream;
use tungstenite::{connect as ws_connect, Message};

const GAME: &str = "Elden Ring Nightreign";
const ITEMS_HANDLING: u8 = 0b111;

pub static RECONNECT: AtomicBool = AtomicBool::new(false);
static SHOP_HINTS: Mutex<Vec<i64>> = Mutex::new(Vec::new());

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
            while let Some(q) = body[at..].find('"') {
                let name_at = at + q + 1;
                let Some(q2) = body[name_at..].find('"') else { break };
                let name = json_unescape(&body[name_at..name_at + q2]);
                let after = &body[name_at + q2 + 1..];
                let Some(colon) = after.find(':') else { break };
                let num: String = after[colon + 1..].trim_start().chars().take_while(|c| c.is_ascii_digit() || *c == '-').collect();
                if let Ok(id) = num.parse::<i64>() {
                    remember_name(map, id, &name);
                }
                at = name_at + q2 + 1;
            }
            from = end + 1;
        }
    }
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

pub fn tutorial_margit() -> bool {
    TUTORIAL_MARGIT.load(Ordering::SeqCst)
}
static GOAL_SENT: AtomicBool = AtomicBool::new(false);
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

fn object_str(obj: &str, key: &str) -> Option<String> {
    let needle = format!("\"{key}\":");
    let at = obj.find(&needle)?;
    let rest = &obj[at + needle.len()..];
    let start = rest.find('"')? + 1;
    let end = rest[start..].find('"')?;
    Some(json_unescape(&rest[start..start + end]))
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
    let id = raw.parse::<i64>().unwrap_or(i64::MIN);
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
    let mut parts = Vec::new();
    let mut from = 0usize;
    while let Some(rel) = text[from..].find("\"text\":\"") {
        let at = from + rel + 8;
        let mut end = at;
        let bytes = text.as_bytes();
        while end < bytes.len() {
            if bytes[end] == b'\\' {
                end += 2;
                continue;
            }
            if bytes[end] == b'"' {
                break;
            }
            end += 1;
        }
        if end > bytes.len() {
            break;
        }
        let raw = json_unescape(&text[at..end]);
        let tail = &text[end..end.saturating_add(80).min(text.len())];
        let kind = if tail.contains("\"type\":\"item_id\"") {
            "item_id"
        } else if tail.contains("\"type\":\"location_id\"") {
            "location_id"
        } else if tail.contains("\"type\":\"player_id\"") {
            "player_id"
        } else {
            ""
        };
        parts.push(part_name(&raw, kind));
        from = end + 1;
    }
    if parts.is_empty() {
        return None;
    }
    Some(parts.join(""))
}

fn handle_server_text(text: &str, log: &impl Fn(&str), next_index: &mut i64, drop_goods: i32) {
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
    remember_players(text);
    if text.contains("RoomInfo") {
        if let Some(seed) = parse_seed(&text) {
            log(&crate::flag_write::bind_seed(&seed));
        }
    }
    if text.contains("DataPackage") {
        let before = ITEM_NAMES.lock().unwrap().len();
        ingest_package(text);
        let after = ITEM_NAMES.lock().unwrap().len();
        if after > before {
            log(&format!("NRAP AP datapackage items {} locations {}", after, LOCATION_NAMES.lock().unwrap().len()));
        }
    }
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
        if crate::flag_write::enqueue_item(ap_index, id) {
            log(&format!("NRAP grant queued {id} index {ap_index}"));
        } else if let Some(msg) = crate::flag_write::apply_now(id) {
            log(&format!("NRAP grant replay {id} index {ap_index}; {msg}"));
        } else {
            log(&format!("NRAP grant skip already {id} index {ap_index}"));
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
        let games = package_games(&room);
        if !games.is_empty() {
            let _ = socket.send(Message::Text(datapackage_request(&games).into()));
            log(&format!("NRAP AP requested datapackage for {}", games.join(", ")));
        }
    }
    let connect = format!(
        "[{{\"cmd\":\"Connect\",\"password\":\"{}\",\"game\":\"{}\",\"name\":\"{}\",\"uuid\":\"\",\"version\":{{\"major\":0,\"minor\":5,\"build\":1,\"class\":\"Version\"}},\"items_handling\":{},\"tags\":[\"AP\"],\"slot_data\":true}}]",
        escape(&cfg.password),
        GAME,
        escape(&cfg.slot),
        ITEMS_HANDLING
    );
    socket
        .send(Message::Text(connect.into()))
        .map_err(|e| format!("Connect send: {e}"))?;
    let reply = read_text(&mut socket)?.ok_or_else(|| "no Connect reply".to_string())?;
    if reply.contains("ConnectionRefused") {
        return Err(format!("ConnectionRefused: {reply}"));
    }
    if !(reply.contains("Connected") || reply.contains("RoomInfo")) {
        return Err(format!("unexpected handshake: {reply}"));
    }
    handle_server_text(&reply, log, next_index, drop_goods);
    let games = package_games(&reply);
    if !games.is_empty() {
        let _ = socket.send(Message::Text(datapackage_request(&games).into()));
        log(&format!("NRAP AP requested datapackage for {}", games.join(", ")));
    }
    tune(&mut socket, Duration::from_millis(80));
    let _ = socket.send(Message::Text("[{\"cmd\":\"Sync\"}]".into()));
    Ok(socket)
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

pub fn run(mut cfg: ApConfig, rx: Receiver<i64>, say_rx: Receiver<String>, drop_goods: i32, config_path: std::path::PathBuf, log: impl Fn(&str)) {
    log(&format!("NRAP AP targeting {} slot {}", cfg.host, cfg.slot));
    let mut pending: Vec<i64> = Vec::new();
    loop {
        let trigger = config_path.parent().map(|p| p.join("reconnect.trigger"));
        let triggered = RECONNECT.swap(false, Ordering::SeqCst)
            || trigger.as_ref().is_some_and(|p| p.is_file());
        if triggered {
            if let Some(path) = &trigger { let _ = std::fs::remove_file(path); }
            if let Ok(text) = std::fs::read_to_string(&config_path) {
                cfg = ApConfig::from_toml(&text);
            }
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
                log(&format!("NRAP AP not connected: {e}"));
            }
        }
        let wait = if RECONNECT.load(Ordering::SeqCst) { Duration::from_millis(200) } else { Duration::from_secs(5) };
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
    }
}
