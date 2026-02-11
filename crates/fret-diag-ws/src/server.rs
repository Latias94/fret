#![cfg(feature = "server-native")]

use fret_diag_protocol::{
    DevtoolsHelloAckV1, DevtoolsHelloV1, DevtoolsSessionAddedV1, DevtoolsSessionDescriptorV1,
    DevtoolsSessionListV1, DevtoolsSessionRemovedV1, DiagTransportMessageV1,
};
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
    sessions: Mutex<HashMap<String, SessionRecord>>,
}

#[derive(Debug)]
struct Peer {
    send: std::sync::mpsc::Sender<DiagTransportMessageV1>,
    kind: PeerKind,
    session_id: Option<String>,
    hello: Option<DevtoolsHelloV1>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum PeerKind {
    Unknown,
    App,
    Tooling,
}

#[derive(Debug, Clone)]
struct SessionRecord {
    desc: DevtoolsSessionDescriptorV1,
}

fn handle_conn(stream: TcpStream, hub: Arc<Hub>, token: String) -> Result<(), String> {
    let mut ws = accept_with_token(stream, &token)?;

    let (tx, rx) = std::sync::mpsc::channel::<DiagTransportMessageV1>();

    let id = {
        let mut next = hub.next_id.lock().map_err(|_| "lock poisoned")?;
        *next = next.saturating_add(1);
        *next
    };

    hub.peers.lock().map_err(|_| "lock poisoned")?.insert(
        id,
        Peer {
            send: tx,
            kind: PeerKind::Unknown,
            session_id: None,
            hello: None,
        },
    );

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
                    handle_incoming(&hub, id, msg);
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

    let removed = hub.peers.lock().map_err(|_| "lock poisoned")?.remove(&id);

    if let Some(peer) = removed {
        if peer.kind == PeerKind::App {
            if let Some(session_id) = peer.session_id {
                let removed_session = hub
                    .sessions
                    .lock()
                    .map_err(|_| "lock poisoned")?
                    .remove(&session_id);
                if removed_session.is_some() {
                    broadcast_to_tooling(
                        hub.as_ref(),
                        None,
                        DiagTransportMessageV1 {
                            schema_version: 1,
                            r#type: "session.removed".to_string(),
                            session_id: None,
                            request_id: None,
                            payload: serde_json::to_value(DevtoolsSessionRemovedV1 { session_id })
                                .unwrap_or_else(|_| serde_json::json!({})),
                        },
                    );
                }
            }
        }
    }
    Ok(())
}

fn handle_incoming(hub: &Hub, from: u64, mut msg: DiagTransportMessageV1) {
    if msg.r#type == "hello" {
        handle_hello(hub, from, msg);
        return;
    }

    let (peer_kind, peer_session_id) = {
        let peers = match hub.peers.lock() {
            Ok(peers) => peers,
            Err(_) => return,
        };
        let Some(peer) = peers.get(&from) else {
            return;
        };
        (peer.kind, peer.session_id.clone())
    };

    match peer_kind {
        PeerKind::Unknown => return,
        PeerKind::App => {
            let Some(session_id) = peer_session_id else {
                return;
            };
            match msg.session_id.as_deref() {
                None => msg.session_id = Some(session_id),
                Some(s) if s == session_id => {}
                Some(_) => return,
            }
        }
        PeerKind::Tooling => {
            if msg.session_id.is_none() {
                return;
            }
        }
    }

    route_message(hub, from, msg);
}

fn handle_hello(hub: &Hub, from: u64, msg: DiagTransportMessageV1) {
    let hello = serde_json::from_value::<DevtoolsHelloV1>(msg.payload.clone()).ok();
    let client_kind = hello
        .as_ref()
        .map(|h| h.client_kind.as_str())
        .unwrap_or("unknown");

    let is_tooling = client_kind == "tooling";
    let is_app = client_kind == "native_app" || client_kind == "web_app";

    if is_tooling {
        {
            let mut peers = match hub.peers.lock() {
                Ok(peers) => peers,
                Err(_) => return,
            };
            if let Some(peer) = peers.get_mut(&from) {
                peer.kind = PeerKind::Tooling;
                peer.hello = hello;
            }
        }

        send_to_peer(
            hub,
            from,
            DiagTransportMessageV1 {
                schema_version: 1,
                r#type: "hello_ack".to_string(),
                session_id: None,
                request_id: msg.request_id,
                payload: serde_json::to_value(DevtoolsHelloAckV1 {
                    server_version: env!("CARGO_PKG_VERSION").to_string(),
                    server_capabilities: vec!["sessions".to_string()],
                })
                .unwrap_or_else(|_| serde_json::json!({})),
            },
        );

        let sessions = hub
            .sessions
            .lock()
            .ok()
            .map(|m| m.values().map(|s| s.desc.clone()).collect::<Vec<_>>())
            .unwrap_or_default();
        send_to_peer(
            hub,
            from,
            DiagTransportMessageV1 {
                schema_version: 1,
                r#type: "session.list".to_string(),
                session_id: None,
                request_id: None,
                payload: serde_json::to_value(DevtoolsSessionListV1 { sessions })
                    .unwrap_or_else(|_| serde_json::json!({ "sessions": [] })),
            },
        );

        return;
    }

    if is_app {
        let assigned_session_id = {
            if let Some(want) = msg.session_id.as_deref() {
                let sessions = match hub.sessions.lock() {
                    Ok(sessions) => sessions,
                    Err(_) => return,
                };
                if !sessions.contains_key(want) {
                    Some(want.to_string())
                } else {
                    None
                }
            } else {
                None
            }
            .unwrap_or_else(|| format!("session-{from}"))
        };

        let desc = DevtoolsSessionDescriptorV1 {
            session_id: assigned_session_id.clone(),
            client_kind: client_kind.to_string(),
            client_version: hello
                .as_ref()
                .map(|h| h.client_version.clone())
                .unwrap_or_else(|| "unknown".to_string()),
            capabilities: hello
                .as_ref()
                .map(|h| h.capabilities.clone())
                .unwrap_or_default(),
        };

        {
            let mut peers = match hub.peers.lock() {
                Ok(peers) => peers,
                Err(_) => return,
            };
            if let Some(peer) = peers.get_mut(&from) {
                peer.kind = PeerKind::App;
                peer.session_id = Some(assigned_session_id.clone());
                peer.hello = hello;
            }
        }

        {
            let mut sessions = match hub.sessions.lock() {
                Ok(sessions) => sessions,
                Err(_) => return,
            };
            sessions.insert(
                assigned_session_id.clone(),
                SessionRecord { desc: desc.clone() },
            );
        }

        send_to_peer(
            hub,
            from,
            DiagTransportMessageV1 {
                schema_version: 1,
                r#type: "hello_ack".to_string(),
                session_id: Some(assigned_session_id.clone()),
                request_id: msg.request_id,
                payload: serde_json::to_value(DevtoolsHelloAckV1 {
                    server_version: env!("CARGO_PKG_VERSION").to_string(),
                    server_capabilities: vec![
                        "inspect".to_string(),
                        "pick".to_string(),
                        "scripts".to_string(),
                        "bundles".to_string(),
                        "sessions".to_string(),
                    ],
                })
                .unwrap_or_else(|_| serde_json::json!({})),
            },
        );

        broadcast_to_tooling(
            hub,
            Some(from),
            DiagTransportMessageV1 {
                schema_version: 1,
                r#type: "session.added".to_string(),
                session_id: None,
                request_id: None,
                payload: serde_json::to_value(DevtoolsSessionAddedV1 { session: desc })
                    .unwrap_or_else(|_| serde_json::json!({})),
            },
        );

        return;
    }

    // Unknown peer kind: acknowledge and do not route until a known kind is negotiated.
    send_to_peer(
        hub,
        from,
        DiagTransportMessageV1 {
            schema_version: 1,
            r#type: "hello_ack".to_string(),
            session_id: None,
            request_id: msg.request_id,
            payload: serde_json::to_value(DevtoolsHelloAckV1 {
                server_version: env!("CARGO_PKG_VERSION").to_string(),
                server_capabilities: vec![],
            })
            .unwrap_or_else(|_| serde_json::json!({})),
        },
    );
}

fn route_message(hub: &Hub, from: u64, msg: DiagTransportMessageV1) {
    let target_session = msg.session_id.clone();

    let peers = match hub.peers.lock() {
        Ok(peers) => peers,
        Err(_) => return,
    };
    for (&id, peer) in peers.iter() {
        if id == from {
            continue;
        }

        match peer.kind {
            PeerKind::Unknown => {}
            PeerKind::Tooling => {
                let _ = peer.send.send(msg.clone());
            }
            PeerKind::App => {
                if target_session.as_deref() == peer.session_id.as_deref() {
                    let _ = peer.send.send(msg.clone());
                }
            }
        }
    }
}

fn send_to_peer(hub: &Hub, id: u64, msg: DiagTransportMessageV1) {
    let peers = match hub.peers.lock() {
        Ok(peers) => peers,
        Err(_) => return,
    };
    let Some(peer) = peers.get(&id) else {
        return;
    };
    let _ = peer.send.send(msg);
}

fn broadcast_to_tooling(hub: &Hub, exclude: Option<u64>, msg: DiagTransportMessageV1) {
    let peers = match hub.peers.lock() {
        Ok(peers) => peers,
        Err(_) => return,
    };
    for (&id, peer) in peers.iter() {
        if exclude == Some(id) {
            continue;
        }
        if peer.kind != PeerKind::Tooling {
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
