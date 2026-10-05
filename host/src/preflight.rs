use std::net::TcpStream;
use std::path::PathBuf;
use std::time::Duration;
use tungstenite::stream::MaybeTlsStream;
use tungstenite::{connect, Message};

pub enum Gate {
    InvalidSlot,
    HostNotFound,
    TimedOut,
    Failed(String),
}

fn classify(err: &str) -> Gate {
    let low = err.to_ascii_lowercase();
    if low.contains("timed out") || low.contains("10060") {
        Gate::TimedOut
    } else if low.contains("no such host") || low.contains("name or service") || low.contains("11001") || low.contains("failed to lookup") {
        Gate::HostNotFound
    } else {
        Gate::Failed(err.to_string())
    }
}

pub fn connect_and_cache(host: &str, slot: &str, password: &str, dir: &PathBuf) -> Result<(), Gate> {
    let addr = host.trim().trim_start_matches("wss://").trim_start_matches("ws://").trim_end_matches('/');
    let addr = addr.replace(['\u{ff1a}', '\u{2236}'], ":");
    let local = addr.starts_with("127.") || addr.starts_with("localhost") || addr.starts_with("0.0.0.0") || addr.starts_with("[::1]");
    let addr = if addr.contains(':') { addr.to_string() } else { format!("{addr}:38281") };
    let url = if local { format!("ws://{addr}") } else { format!("wss://{addr}") };
    let (mut socket, _) = connect(&url).map_err(|e| classify(&e.to_string()))?;
    let tcp = match socket.get_ref() {
        MaybeTlsStream::Plain(stream) => stream,
        MaybeTlsStream::NativeTls(stream) => stream.get_ref(),
        _ => return Err(Gate::Failed("unsupported socket".into())),
    };
    tcp.set_read_timeout(Some(Duration::from_secs(12))).ok();
    tcp.set_write_timeout(Some(Duration::from_secs(12))).ok();
    let connect_pkt = format!(
        "[{{\"cmd\":\"Connect\",\"password\":\"{}\",\"game\":\"Elden Ring Nightreign\",\"name\":\"{}\",\"uuid\":\"\",\"version\":{{\"major\":0,\"minor\":5,\"build\":1,\"class\":\"Version\"}},\"items_handling\":7,\"tags\":[\"AP\"],\"slot_data\":true}}]",
        password.replace('\\', "\\\\").replace('"', "\\\""),
        slot.replace('\\', "\\\\").replace('"', "\\\"")
    );
    socket.send(Message::Text(connect_pkt.into())).map_err(|e| classify(&e.to_string()))?;
    let mut package = String::new();
    let mut connected = false;
    for _ in 0..12 {
        let text = socket.read().map_err(|e| classify(&e.to_string()))?.to_string();
        if text.contains("InvalidSlot") {
            return Err(Gate::InvalidSlot);
        }
        if text.contains("\"cmd\":\"Connected\"") || text.contains("\"cmd\": \"Connected\"") {
            connected = true;
            let _ = socket.send(Message::Text("[{\"cmd\":\"GetDataPackage\",\"games\":[\"Elden Ring Nightreign\"]}]".into()));
        }
        if text.contains("DataPackage") {
            package = text;
            break;
        }
    }
    if !connected {
        return Err(Gate::Failed("server did not accept the slot".into()));
    }
    if package.is_empty() {
        return Err(Gate::Failed("data package was not downloaded".into()));
    }
    let folder = dir.join("datapackages");
    std::fs::create_dir_all(&folder).map_err(|e| Gate::Failed(e.to_string()))?;
    std::fs::write(folder.join("preflight_package.txt"), package).map_err(|e| Gate::Failed(e.to_string()))?;
    let _ = socket.close(None);
    Ok(())
}

pub fn game_open() -> bool {
    std::process::Command::new("tasklist")
        .args(["/FI", "IMAGENAME eq nightreign.exe", "/NH"])
        .output()
        .map(|out| String::from_utf8_lossy(&out.stdout).to_ascii_lowercase().contains("nightreign.exe"))
        .unwrap_or(false)
}
