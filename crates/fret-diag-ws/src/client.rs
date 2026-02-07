use fret_diag_protocol::DiagTransportMessageV1;
use serde_json::Value;
use std::collections::VecDeque;
use std::sync::{Arc, Mutex};
use std::time::Duration;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ClientKindV1 {
    NativeApp,
    WebApp,
    Tooling,
}

impl ClientKindV1 {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::NativeApp => "native_app",
            Self::WebApp => "web_app",
            Self::Tooling => "tooling",
        }
    }
}

#[derive(Debug, Clone)]
pub struct DevtoolsWsClientConfig {
    pub ws_url: String,
    pub token: String,
    pub client_kind: ClientKindV1,
    pub client_version: String,
    pub capabilities: Vec<String>,
    pub read_timeout: Duration,
}

impl DevtoolsWsClientConfig {
    pub fn with_defaults(ws_url: impl Into<String>, token: impl Into<String>) -> Self {
        Self {
            ws_url: ws_url.into(),
            token: token.into(),
            client_kind: ClientKindV1::NativeApp,
            client_version: env!("CARGO_PKG_VERSION").to_string(),
            capabilities: Vec::new(),
            read_timeout: Duration::from_millis(5),
        }
    }
}

#[derive(Debug, Clone)]
pub struct DevtoolsWsClient {
    state: Arc<State>,
}

#[derive(Debug)]
struct State {
    outbox: Mutex<VecDeque<DiagTransportMessageV1>>,
    inbox: Mutex<VecDeque<DiagTransportMessageV1>>,
}

impl DevtoolsWsClient {
    pub fn connect_native(cfg: DevtoolsWsClientConfig) -> Result<Self, String> {
        Self::connect_native_inner(cfg)
    }

    pub fn try_recv(&self) -> Option<DiagTransportMessageV1> {
        self.state.inbox.lock().ok()?.pop_front()
    }

    pub fn send(&self, msg: DiagTransportMessageV1) {
        if let Ok(mut outbox) = self.state.outbox.lock() {
            outbox.push_back(msg);
        }
    }

    pub fn send_type_payload(&self, ty: impl Into<String>, payload: Value) {
        self.send(DiagTransportMessageV1 {
            schema_version: 1,
            r#type: ty.into(),
            session_id: None,
            request_id: None,
            payload,
        });
    }

    #[cfg(feature = "client-native")]
    fn connect_native_inner(cfg: DevtoolsWsClientConfig) -> Result<Self, String> {
        use tungstenite::Message;

        let state = Arc::new(State {
            outbox: Mutex::new(VecDeque::new()),
            inbox: Mutex::new(VecDeque::new()),
        });

        let state_thread = Arc::clone(&state);
        std::thread::spawn(move || {
            let mut backoff = Duration::from_millis(200);

            loop {
                let ws_url =
                    append_token_query_param(&cfg.ws_url, &cfg.token, "fret_devtools_token");

                let connect_result = tungstenite::connect(ws_url.as_str());
                let (mut ws, _resp) = match connect_result {
                    Ok(ok) => ok,
                    Err(_) => {
                        std::thread::sleep(backoff);
                        backoff = (backoff * 2).min(Duration::from_secs(5));
                        continue;
                    }
                };

                backoff = Duration::from_millis(200);

                {
                    use tungstenite::stream::MaybeTlsStream;
                    match ws.get_mut() {
                        MaybeTlsStream::Plain(stream) => {
                            let _ = stream.set_read_timeout(Some(cfg.read_timeout));
                            let _ = stream.set_write_timeout(Some(Duration::from_millis(50)));
                        }
                        _ => {}
                    }
                }

                let hello = DiagTransportMessageV1 {
                    schema_version: 1,
                    r#type: "hello".to_string(),
                    session_id: None,
                    request_id: Some(1),
                    payload: serde_json::json!({
                        "client_kind": cfg.client_kind.as_str(),
                        "client_version": cfg.client_version,
                        "capabilities": cfg.capabilities,
                    }),
                };
                let _ = ws.send(Message::Text(
                    serde_json::to_string(&hello).unwrap_or_default().into(),
                ));

                loop {
                    // Flush outbound messages first.
                    if let Ok(mut outbox) = state_thread.outbox.lock() {
                        while let Some(msg) = outbox.pop_front() {
                            let Ok(text) = serde_json::to_string(&msg) else {
                                continue;
                            };
                            if ws.send(Message::Text(text.into())).is_err() {
                                break;
                            }
                        }
                    }

                    match ws.read() {
                        Ok(Message::Text(text)) => {
                            if let Ok(msg) = serde_json::from_str::<DiagTransportMessageV1>(&text) {
                                if let Ok(mut inbox) = state_thread.inbox.lock() {
                                    inbox.push_back(msg);
                                }
                            }
                        }
                        Ok(Message::Binary(_)) => {}
                        Ok(Message::Ping(_)) => {}
                        Ok(Message::Pong(_)) => {}
                        Ok(Message::Frame(_)) => {}
                        Ok(Message::Close(_)) => break,
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
            }
        });

        Ok(Self { state })
    }
}

#[cfg(feature = "client-native")]
fn append_token_query_param(url: &str, token: &str, key: &str) -> String {
    use url::Url;
    let Ok(mut url) = Url::parse(url) else {
        return url.to_string();
    };
    let pairs = url.query_pairs().collect::<Vec<_>>();
    if !pairs.iter().any(|(k, _)| k.as_ref() == key) {
        url.query_pairs_mut().append_pair(key, token);
    }
    url.to_string()
}
