//! Blocking AP client. Own thread. Flag thread only pushes location ids.

use std::net::TcpStream;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Mutex;
use std::sync::mpsc::{Receiver, RecvTimeoutError};
use std::time::Duration;
use tungstenite::protocol::WebSocket;
use tungstenite::stream::MaybeTlsStream;
use tungstenite::{connect as ws_connect, Message};

const GAME: &str = "Elden Ring Nightreign";
const ITEMS_HANDLING: u8 = 0b111;

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

fn item_name(id: i64) -> &'static str {
    match id {
        839_100_001 => "Expedition Unlock - Tricephalos",
        839_100_008 => "Expedition Unlock - Heolstor",
        839_100_009 => "Expedition Unlock - Harmonia",
        839_100_010 => "Expedition Unlock - Straghess",
        839_100_100 => "Murk Bundle",
        839_100_900 => "Victory",
        _ => "unknown item",
    }
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
static GOAL_SENT: AtomicBool = AtomicBool::new(false);
static PLAYERS: Mutex<Vec<(i64, String)>> = Mutex::new(Vec::new());

fn player_name(id: i64) -> Option<String> {
    PLAYERS.lock().unwrap().iter().find(|(slot, _)| *slot == id).map(|(_, name)| name.clone())
}

fn remember_players(text: &str) {
    if !text.contains("Connected") {
        return;
    }
    let mut players = Vec::new();
    let mut from = 0usize;
    while let Some(rel) = text[from..].find("\"slot\":") {
        let at = from + rel;
        let Some(slot) = parse_i64_after(&text[at..], "slot") else { break };
        let name = text[at..].find("\"alias\":\"").or_else(|| text[at..].find("\"name\":\"")).map(|n| {
            let key_at = at + n;
            let start = text[key_at..].find("\":\"").map(|i| key_at + i + 3).unwrap_or(key_at);
            let end = text[start..].find('"').map(|i| start + i).unwrap_or(start);
            text[start..end].to_string()
        }).unwrap_or_else(|| format!("Player {slot}"));
        players.push((slot, name));
        from = at + 8;
        if players.len() > 32 { break; }
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
        "item_id" => crate::names::item_label(id).unwrap_or(raw).to_string(),
        "location_id" => crate::names::location_label(id).unwrap_or_else(|| raw.to_string()),
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
        } else {
            crate::flag_write::remember_unlock(id);
            log(&format!("NRAP grant skip already {id} index {ap_index}"));
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
    let addr = cfg.addr();
    let local = addr.starts_with("127.") || addr.starts_with("localhost");
    let url = if local { format!("ws://{addr}") } else { format!("wss://{addr}") };
    let (mut socket, _) = ws_connect(&url).map_err(|e| format!("ws handshake {url}: {e}"))?;
    tune(&mut socket, Duration::from_secs(8));
    if let Ok(Some(room)) = read_text(&mut socket) {
        handle_server_text(&room, log, next_index, drop_goods);
    }
    let connect = format!(
        "[{{\"cmd\":\"Connect\",\"password\":\"{}\",\"game\":\"{}\",\"name\":\"{}\",\"uuid\":\"\",\"version\":{{\"major\":0,\"minor\":5,\"build\":1,\"class\":\"Version\"}},\"items_handling\":{},\"tags\":[\"AP\"],\"slot_data\":false}}]",
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

pub fn run(cfg: ApConfig, rx: Receiver<i64>, say_rx: Receiver<String>, drop_goods: i32, log: impl Fn(&str)) {
    log(&format!("NRAP AP targeting {} slot {}", cfg.host, cfg.slot));
    let mut pending: Vec<i64> = Vec::new();
    loop {
        let mut next_index = 0i64;
        match connect_and_handshake(&cfg, &log, &mut next_index, drop_goods) {
            Ok(mut socket) => {
                log("NRAP AP connected");
                for _ in 0..20 {
                    if let Err(e) = drain_server(&mut socket, &log, &mut next_index, drop_goods) {
                        log(&format!("NRAP AP socket ended: {e}"));
                        break;
                    }
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
        let wait_until = std::time::Instant::now() + Duration::from_secs(5);
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
