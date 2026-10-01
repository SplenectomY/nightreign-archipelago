//! Blocking AP client. Own thread. Flag thread only pushes location ids.

use std::net::TcpStream;
use std::sync::mpsc::{Receiver, RecvTimeoutError};
use std::time::Duration;
use tungstenite::protocol::WebSocket;
use tungstenite::{client::client as ws_client, Message};

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

type Socket = WebSocket<TcpStream>;

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

fn preview(text: &str) -> String {
    let t = text.replace('\n', " ");
    if t.len() <= 240 {
        t
    } else {
        format!("{}…", &t[..240])
    }
}

fn handle_server_text(text: &str, log: &impl Fn(&str), next_index: &mut i64, drop_goods: i32) {
    log(&format!("NRAP AP rx {}", preview(text)));
    if text.contains("ConnectionRefused") {
        log(&format!("NRAP AP server refused: {text}"));
        return;
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
        if let Some(msg) = crate::flag_write::apply_item(id) {
            log(&msg);
        }
        if let Some(msg) = crate::hero::want(id) {
            log(&msg);
        }
        if let Some(msg) = crate::drop::apply_item(id, drop_goods) {
            log(&msg);
        }
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
        Ok(Message::Close(_)) => Err("server closed".into()),
        Err(_) => Ok(None),
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

fn connect_and_handshake(
    cfg: &ApConfig,
    log: &impl Fn(&str),
    next_index: &mut i64,
    drop_goods: i32,
) -> Result<Socket, String> {
    let addr = cfg.addr();
    let stream = TcpStream::connect(&addr).map_err(|e| format!("tcp {addr}: {e}"))?;
    stream
        .set_read_timeout(Some(Duration::from_secs(8)))
        .ok();
    stream.set_nodelay(true).ok();
    let url = format!("ws://{addr}");
    let (mut socket, _) =
        ws_client(&url, stream).map_err(|e| format!("ws handshake {url}: {e}"))?;
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
    socket
        .get_ref()
        .set_read_timeout(Some(Duration::from_millis(80)))
        .ok();
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

pub fn run(cfg: ApConfig, rx: Receiver<i64>, drop_goods: i32, log: impl Fn(&str)) {
    log(&format!("NRAP AP targeting {} slot {}", cfg.host, cfg.slot));
    let mut pending: Vec<i64> = Vec::new();
    loop {
        let mut next_index = 0i64;
        match connect_and_handshake(&cfg, &log, &mut next_index, drop_goods) {
            Ok(mut socket) => {
                log("NRAP AP connected");
                for _ in 0..20 {
                    if drain_server(&mut socket, &log, &mut next_index, drop_goods).is_err() {
                        log("NRAP AP server closed");
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
                            if drain_server(&mut socket, &log, &mut next_index, drop_goods).is_err()
                            {
                                log("NRAP AP server closed");
                                break;
                            }
                            match send_goal(&mut socket) {
                                Ok(true) => log("NRAP AP goal sent"),
                                Ok(false) => {}
                                Err(e) => log(&format!("NRAP AP goal failed: {e}")),
                            }
                        }
                        Err(RecvTimeoutError::Timeout) => {
                            if drain_server(&mut socket, &log, &mut next_index, drop_goods).is_err()
                            {
                                log("NRAP AP server closed");
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
