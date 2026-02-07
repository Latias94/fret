#![cfg(feature = "server-native")]

use fret_diag_protocol::DiagTransportMessageV1;
use std::collections::HashMap;
use std::net::{SocketAddr, TcpListener, TcpStream};
use std::sync::{Arc, Mutex};
use std::time::Duration;
use tungstenite::handshake::server::{Request, Response};
use tungstenite::{Message, WebSocket};

#[derive(Debug, Clone)]
pub struct DevtoolsWsServerConfig {
    pub bind: SocketAddr,
    pub token: String,
}

#[derive(Debug)]
pub struct DevtoolsWsServer {
    cfg: DevtoolsWsServerConfig,
}

impl DevtoolsWsServer {
    pub fn new(cfg: DevtoolsWsServerConfig) -> Self {
        Self { cfg }
    }

    pub fn run(&self) -> Result<(), String> {
        let listener = TcpListener::bind(self.cfg.bind).map_err(|e| e.to_string())?;
        let hub = Arc::new(Hub::default());

        for stream in listener.incoming() {
            let stream = match stream {
                Ok(stream) => stream,
                Err(_) => continue,
            };
            let hub = Arc::clone(&hub);
            let token = self.cfg.token.clone();

            std::thread::spawn(move || {
                let _ = handle_conn(stream, hub, token);
            });
        }

        Ok(())
    }
}

#[derive(Default, Debug)]
struct Hub {
    next_id: Mutex<u64>,
    peers: Mutex<HashMap<u64, Peer>>,
}

#[derive(Debug)]
struct Peer {
    send: std::sync::mpsc::Sender<DiagTransportMessageV1>,
}

fn handle_conn(stream: TcpStream, hub: Arc<Hub>, token: String) -> Result<(), String> {
    let mut ws = accept_with_token(stream, &token)?;

    let (tx, rx) = std::sync::mpsc::channel::<DiagTransportMessageV1>();

    let id = {
        let mut next = hub.next_id.lock().map_err(|_| "lock poisoned")?;
        *next = next.saturating_add(1);
        *next
    };

    hub.peers
        .lock()
        .map_err(|_| "lock poisoned")?
        .insert(id, Peer { send: tx });

    let _ = ws
        .get_mut()
        .set_read_timeout(Some(Duration::from_millis(5)));
    let _ = ws
        .get_mut()
        .set_write_timeout(Some(Duration::from_millis(50)));

    loop {
        // Flush outbound first.
        while let Ok(msg) = rx.try_recv() {
            let Ok(text) = serde_json::to_string(&msg) else {
                continue;
            };
            if ws.send(Message::Text(text.into())).is_err() {
                break;
            }
        }

        match ws.read() {
            Ok(Message::Text(text)) => {
                if let Ok(msg) = serde_json::from_str::<DiagTransportMessageV1>(&text) {
                    broadcast(&hub, id, msg);
                }
            }
            Ok(Message::Close(_)) => break,
            Ok(_) => {}
            Err(err) => {
                let is_timeout = matches!(
                    err,
                    tungstenite::Error::Io(ref io) if io.kind() == std::io::ErrorKind::TimedOut
                );
                if is_timeout {
                    continue;
                }
                break;
            }
        }
    }

    let _ = hub.peers.lock().map_err(|_| "lock poisoned")?.remove(&id);
    Ok(())
}

fn broadcast(hub: &Hub, from: u64, msg: DiagTransportMessageV1) {
    let peers = match hub.peers.lock() {
        Ok(peers) => peers,
        Err(_) => return,
    };
    for (&id, peer) in peers.iter() {
        if id == from {
            continue;
        }
        let _ = peer.send.send(msg.clone());
    }
}

fn accept_with_token(
    stream: TcpStream,
    expected_token: &str,
) -> Result<WebSocket<TcpStream>, String> {
    let expected_token = expected_token.to_string();
    tungstenite::accept_hdr(stream, move |req: &Request, resp: Response| {
        if token_matches_request(req, &expected_token) {
            return Ok(resp);
        }

        let mut resp = resp.map(|_| Some("Unauthorized".to_string()));
        *resp.status_mut() = tungstenite::http::StatusCode::UNAUTHORIZED;
        Err(resp)
    })
    .map_err(|e| e.to_string())
}

fn token_matches_request(req: &Request, expected: &str) -> bool {
    let uri = req.uri();
    let Some(query) = uri.query() else {
        return false;
    };

    for pair in query.split('&') {
        let mut it = pair.splitn(2, '=');
        let k = it.next().unwrap_or_default();
        let v = it.next().unwrap_or_default();
        if k == "fret_devtools_token" && v == expected {
            return true;
        }
    }
    false
}
