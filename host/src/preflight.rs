use std::net::TcpStream;
use std::path::PathBuf;
use std::sync::Arc;
use std::time::Duration;
use rustls::ClientConfig;
use tungstenite::stream::MaybeTlsStream;
use tungstenite::{connect, Message};
use tungstenite::client::IntoClientRequest;
use tungstenite::Connector;
use webpki_roots::TLS_SERVER_ROOTS;

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

fn tls_error(err: &str) -> bool {
    let low = err.to_ascii_lowercase();
    low.contains("tls") || low.contains("certificate") || low.contains("schannel") || low.contains("2146893048") || low.contains("invalid token") || low.contains("webpki")
}

fn rustls_config(insecure: bool) -> Arc<ClientConfig> {
    if insecure {
        let config = ClientConfig::builder()
            .dangerous()
            .with_custom_certificate_verifier(Arc::new(NoVerifier))
            .with_no_client_auth();
        Arc::new(config)
    } else {
        let mut roots = rustls::RootCertStore::empty();
        roots.extend(TLS_SERVER_ROOTS.iter().cloned());
        let config = ClientConfig::builder()
            .with_root_certificates(roots)
            .with_no_client_auth();
        Arc::new(config)
    }
}

#[derive(Debug)]
struct NoVerifier;

impl rustls::client::danger::ServerCertVerifier for NoVerifier {
    fn verify_server_cert(
        &self,
        _end_entity: &rustls::pki_types::CertificateDer<'_>,
        _intermediates: &[rustls::pki_types::CertificateDer<'_>],
        _server_name: &rustls::pki_types::ServerName<'_>,
        _ocsp: &[u8],
        _now: rustls::pki_types::UnixTime,
    ) -> Result<rustls::client::danger::ServerCertVerified, rustls::Error> {
        Ok(rustls::client::danger::ServerCertVerified::assertion())
    }
    fn verify_tls12_signature(
        &self,
        _message: &[u8],
        _cert: &rustls::pki_types::CertificateDer<'_>,
        _dss: &rustls::DigitallySignedStruct,
    ) -> Result<rustls::client::danger::HandshakeSignatureValid, rustls::Error> {
        Ok(rustls::client::danger::HandshakeSignatureValid::assertion())
    }
    fn verify_tls13_signature(
        &self,
        _message: &[u8],
        _cert: &rustls::pki_types::CertificateDer<'_>,
        _dss: &rustls::DigitallySignedStruct,
    ) -> Result<rustls::client::danger::HandshakeSignatureValid, rustls::Error> {
        Ok(rustls::client::danger::HandshakeSignatureValid::assertion())
    }
    fn supported_verify_schemes(&self) -> Vec<rustls::SignatureScheme> {
        vec![
            rustls::SignatureScheme::RSA_PKCS1_SHA256,
            rustls::SignatureScheme::RSA_PKCS1_SHA384,
            rustls::SignatureScheme::RSA_PKCS1_SHA512,
            rustls::SignatureScheme::ECDSA_NISTP256_SHA256,
            rustls::SignatureScheme::ECDSA_NISTP384_SHA384,
            rustls::SignatureScheme::ECDSA_NISTP521_SHA512,
            rustls::SignatureScheme::RSA_PSS_SHA256,
            rustls::SignatureScheme::RSA_PSS_SHA384,
            rustls::SignatureScheme::RSA_PSS_SHA512,
            rustls::SignatureScheme::ED25519,
        ]
    }
}

fn connect_rustls(url: &str, insecure: bool) -> Result<(tungstenite::WebSocket<MaybeTlsStream<TcpStream>>, tungstenite::handshake::client::Response), String> {
    let request = url.into_client_request().map_err(|e| e.to_string())?;
    let uri = request.uri();
    let host = uri.host().ok_or_else(|| "no host".to_string())?;
    let port = uri.port_u16().unwrap_or(443);
    let addr = format!("{host}:{port}");
    let stream = TcpStream::connect(&addr).map_err(|e| e.to_string())?;
    let connector = Connector::Rustls(rustls_config(insecure));
    tungstenite::client_tls_with_config(request, stream, None, Some(connector)).map_err(|e| e.to_string())
}

pub fn connect_and_cache(host: &str, slot: &str, password: &str, dir: &PathBuf) -> Result<(), Gate> {
    let ver = env!("CARGO_PKG_VERSION");
    let host = host.trim();
    let force_plain = host.starts_with("ws://");
    let addr = host.trim_start_matches("wss://").trim_start_matches("ws://").trim_end_matches('/');
    let addr = addr.replace(['\u{ff1a}', '\u{2236}'], ":");
    let local = addr.starts_with("127.") || addr.starts_with("localhost") || addr.starts_with("0.0.0.0") || addr.starts_with("[::1]");
    let addr = if addr.contains(':') { addr.to_string() } else { format!("{addr}:38281") };
    let url = if local || force_plain { format!("ws://{addr}") } else { format!("wss://{addr}") };

    let flags = std::fs::read_to_string(dir.join("flags.toml")).unwrap_or_default();
    let ignore = flags.lines().any(|l| l.trim() == "ignore_tls_errors = true");
    let fallback = !flags.lines().any(|l| l.trim() == "tls_fallback = false"); // on by default

    let (mut socket, _) = if url.starts_with("ws://") {
        connect(&url).map_err(|e| classify(&format!("{ver} {e}")))?
    } else if ignore {
        // Pure rustls with no cert verification — never touches Schannel.
        connect_rustls(&url, true).map_err(|e| classify(&format!("{ver} rustls-insecure: {e}")))?
    } else {
        // Prefer rustls + webpki-roots (bundled Mozilla CAs).
        match connect_rustls(&url, false) {
            Ok(pair) => pair,
            Err(e) => {
                let msg = e.to_string();
                if fallback && tls_error(&msg) {
                    connect_rustls(&url, true).map_err(|e2| classify(&format!("{ver} rustls: {msg}; insecure: {e2}")))?
                } else {
                    return Err(classify(&format!("{ver} rustls: {msg}")));
                }
            }
        }
    };

    let tcp = match socket.get_ref() {
        MaybeTlsStream::Plain(stream) => stream,
        MaybeTlsStream::NativeTls(stream) => stream.get_ref(),
        MaybeTlsStream::Rustls(stream) => stream.get_ref(),
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
    let mut seed = String::new();
    for _ in 0..12 {
        let msg = socket.read().map_err(|e| classify(&e.to_string()))?;
        let text = match msg {
            Message::Text(t) => t.to_string(),
            Message::Binary(b) => String::from_utf8_lossy(&b).to_string(),
            Message::Ping(p) => { let _ = socket.send(Message::Pong(p)); continue; }
            Message::Pong(_) | Message::Frame(_) => continue,
            Message::Close(_) => break,
        };
        if text.contains("InvalidSlot") {
            return Err(Gate::InvalidSlot);
        }
        if seed.is_empty() {
            if let Some(found) = seed_name(&text) {
                seed = found;
            }
        }
        if text.contains("\"cmd\":\"Connected\"") || text.contains("\"cmd\": \"Connected\"") {
            connected = true;
            let _ = socket.send(Message::Text("[{\"cmd\":\"GetDataPackage\",\"games\":[\"Elden Ring Nightreign\"]}]".into()));
        }
        if text.contains("DataPackage") {
            package = text;
            if connected && !seed.is_empty() {
                break;
            }
        }
    }
    if !connected {
        return Err(Gate::Failed("server did not accept the slot".into()));
    }
    if seed.is_empty() {
        return Err(Gate::Failed("server did not send a seed name".into()));
    }
    std::fs::write(dir.join("launch.seed"), &seed).map_err(|e| Gate::Failed(e.to_string()))?;
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

fn seed_name(text: &str) -> Option<String> {
    for key in ["\"seed_name\":\"", "\"seed_name\": \"", "\"seed\":\"", "\"seed\": \""] {
        if let Some(at) = text.find(key) {
            let rest = &text[at + key.len()..];
            if let Some(end) = rest.find('"') {
                if end > 0 {
                    return Some(rest[..end].to_string());
                }
            }
        }
    }
    None
}
