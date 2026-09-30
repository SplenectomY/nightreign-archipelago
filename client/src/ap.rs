//! Blocking AP client. Own thread. Flag thread only pushes location ids.

use std::net::TcpStream;
use std::sync::mpsc::{Receiver, RecvTimeoutError};
use std::time::Duration;
use tungstenite::protocol::WebSocket;
use tungstenite::{client::client as ws_client, Message};

const GAME: &str = "Elden Ring Nightreign";

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

fn connect_and_handshake(cfg: &ApConfig) -> Result<Socket, String> {
    let addr = cfg.addr();
    let stream = TcpStream::connect(&addr).map_err(|e| format!("tcp {addr}: {e}"))?;
    stream
        .set_read_timeout(Some(Duration::from_secs(8)))
        .ok();
    stream.set_nodelay(true).ok();
    let url = format!("ws://{addr}");
    let (mut socket, _) =
        ws_client(&url, stream).map_err(|e| format!("ws handshake {url}: {e}"))?;
    let _ = socket.read();
    let connect = format!(
        "[{{\"cmd\":\"Connect\",\"password\":\"{}\",\"game\":\"{}\",\"name\":\"{}\",\"uuid\":\"\",\"version\":{{\"major\":0,\"minor\":5,\"build\":1,\"class\":\"Version\"}},\"items_handling\":0,\"tags\":[\"AP\"],\"slot_data\":false}}]",
        escape(&cfg.password),
        GAME,
        escape(&cfg.slot)
    );
    socket
        .send(Message::Text(connect))
        .map_err(|e| format!("Connect send: {e}"))?;
    let reply = socket.read().map_err(|e| format!("Connect read: {e}"))?;
    let text = match reply {
        Message::Text(t) => t,
        other => return Err(format!("Connect reply not text: {other:?}")),
    };
    if text.contains("ConnectionRefused") {
        return Err(format!("ConnectionRefused: {text}"));
    }
    if !(text.contains("Connected") || text.contains("RoomInfo")) {
        return Err(format!("unexpected handshake: {text}"));
    }
    socket
        .get_ref()
        .set_read_timeout(Some(Duration::from_millis(50)))
        .ok();
    Ok(socket)
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
    let pkt = format!("[{\"cmd\":\"LocationChecks\",\"locations\":[{list}]}]");
    socket
        .send(Message::Text(pkt))
        .map_err(|e| format!("LocationChecks: {e}"))
}

pub fn run(cfg: ApConfig, rx: Receiver<i64>, log: impl Fn(&str)) {
    log(&format!("NRAP AP targeting {} slot {}", cfg.host, cfg.slot));
    let mut pending: Vec<i64> = Vec::new();
    loop {
        match connect_and_handshake(&cfg) {
            Ok(mut socket) => {
                log("NRAP AP connected");
                if let Err(e) = send_checks(&mut socket, &pending) {
                    log(&format!("NRAP AP resend failed: {e}"));
                } else if !pending.is_empty() {
                    log(&format!("NRAP AP resent {} cached checks", pending.len()));
                }
                loop {
                    match rx.recv_timeout(Duration::from_millis(400)) {
                        Ok(id) => {
                            if !pending.contains(&id) {
                                pending.push(id);
                            }
                            if let Err(e) = send_checks(&mut socket, &[id]) {
                                log(&format!("NRAP AP send {id} failed: {e}"));
                                break;
                            }
                            log(&format!("NRAP AP LocationChecks {id}"));
                        }
                        Err(RecvTimeoutError::Timeout) => match socket.read() {
                            Ok(Message::Close(_)) => {
                                log("NRAP AP server closed");
                                break;
                            }
                            Ok(_) | Err(_) => {}
                        },
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
