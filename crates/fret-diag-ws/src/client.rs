use fret_diag_protocol::DiagTransportMessageV1;
use serde_json::Value;
use std::collections::VecDeque;
use std::sync::{Arc, Mutex};
use std::time::Duration;

#[cfg(feature = "client-wasm")]
use wasm_bindgen::JsCast as _;
#[cfg(feature = "client-wasm")]
use wasm_bindgen::prelude::*;

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

#[derive(Clone)]
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

pub struct DevtoolsWsClient {
    state: Arc<State>,
    #[cfg(feature = "client-wasm")]
    wasm: Option<WasmInner>,
}

impl std::fmt::Debug for DevtoolsWsClient {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("DevtoolsWsClient").finish_non_exhaustive()
    }
}

#[derive(Debug)]
struct State {
    outbox: Mutex<VecDeque<DiagTransportMessageV1>>,
    inbox: Mutex<VecDeque<DiagTransportMessageV1>>,
    default_session_id: Mutex<Option<String>>,
}

#[cfg(feature = "client-wasm")]
struct WasmInner {
    ws: web_sys::WebSocket,
    _on_open: Closure<dyn FnMut(web_sys::Event)>,
    _on_message: Closure<dyn FnMut(web_sys::MessageEvent)>,
    _on_error: Closure<dyn FnMut(web_sys::ErrorEvent)>,
    _on_close: Closure<dyn FnMut(web_sys::CloseEvent)>,
}

impl DevtoolsWsClient {
    #[cfg(feature = "client-native")]
    pub fn connect_native(cfg: DevtoolsWsClientConfig) -> Result<Self, String> {
        Self::connect_native_inner(cfg)
    }

    #[cfg(feature = "client-wasm")]
    pub fn connect_wasm(cfg: DevtoolsWsClientConfig) -> Result<Self, String> {
        let state = Arc::new(State {
            outbox: Mutex::new(VecDeque::new()),
            inbox: Mutex::new(VecDeque::new()),
            default_session_id: Mutex::new(None),
        });

        let ws_url =
            append_token_query_param_simple(&cfg.ws_url, &cfg.token, "fret_devtools_token");
        let ws = web_sys::WebSocket::new(&ws_url).map_err(|_| "WebSocket::new failed")?;

        let state_open = Arc::clone(&state);
        let cfg_open = cfg.clone();
        let ws_open = ws.clone();
        let on_open = Closure::wrap(Box::new(move |_e: web_sys::Event| {
            let hello = DiagTransportMessageV1 {
                schema_version: 1,
                r#type: "hello".to_string(),
                session_id: None,
                request_id: Some(1),
                payload: serde_json::json!({
                    "client_kind": cfg_open.client_kind.as_str(),
                    "client_version": cfg_open.client_version,
                    "capabilities": cfg_open.capabilities,
                }),
            };
            if let Ok(text) = serde_json::to_string(&hello) {
                let _ = ws_open.send_with_str(&text);
            }

            flush_wasm_outbox(&ws_open, &state_open);
        }) as Box<dyn FnMut(web_sys::Event)>);
        ws.set_onopen(Some(on_open.as_ref().unchecked_ref()));

        let state_message = Arc::clone(&state);
        let on_message = Closure::wrap(Box::new(move |e: web_sys::MessageEvent| {
            let Some(text) = e.data().as_string() else {
                return;
            };
            let Ok(msg) = serde_json::from_str::<DiagTransportMessageV1>(&text) else {
                return;
            };
            maybe_capture_session_id(&state_message, &msg);
            if let Ok(mut inbox) = state_message.inbox.lock() {
                inbox.push_back(msg);
            }
        }) as Box<dyn FnMut(web_sys::MessageEvent)>);
        ws.set_onmessage(Some(on_message.as_ref().unchecked_ref()));

        let on_error = Closure::wrap(Box::new(move |_e: web_sys::ErrorEvent| {
            // Best-effort: errors are surfaced via close/reconnect at higher layers.
        }) as Box<dyn FnMut(web_sys::ErrorEvent)>);
        ws.set_onerror(Some(on_error.as_ref().unchecked_ref()));

        let on_close = Closure::wrap(Box::new(move |_e: web_sys::CloseEvent| {
            // Best-effort: callers can re-create the client to reconnect.
        }) as Box<dyn FnMut(web_sys::CloseEvent)>);
        ws.set_onclose(Some(on_close.as_ref().unchecked_ref()));

        Ok(Self {
            state,
            wasm: Some(WasmInner {
                ws,
                _on_open: on_open,
                _on_message: on_message,
                _on_error: on_error,
                _on_close: on_close,
            }),
        })
    }

    pub fn try_recv(&self) -> Option<DiagTransportMessageV1> {
        self.state.inbox.lock().ok()?.pop_front()
    }

    pub fn default_session_id(&self) -> Option<String> {
        self.state
            .default_session_id
            .lock()
            .ok()
            .and_then(|v| v.clone())
    }

    pub fn set_default_session_id(&self, session_id: Option<String>) {
        if let Ok(mut v) = self.state.default_session_id.lock() {
            *v = session_id;
        }
    }

    pub fn send(&self, msg: DiagTransportMessageV1) {
        if let Ok(mut outbox) = self.state.outbox.lock() {
            let mut msg = msg;
            if msg.session_id.is_none()
                && msg.r#type != "hello"
                && let Ok(default) = self.state.default_session_id.lock()
                && let Some(s) = default.as_deref()
            {
                msg.session_id = Some(s.to_string());
            }
            outbox.push_back(msg);
        }

        #[cfg(feature = "client-wasm")]
        if let Some(wasm) = self.wasm.as_ref() {
            flush_wasm_outbox(&wasm.ws, &self.state);
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
            default_session_id: Mutex::new(None),
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
                    if let MaybeTlsStream::Plain(stream) = ws.get_mut() {
                        let _ = stream.set_read_timeout(Some(cfg.read_timeout));
                        let _ = stream.set_write_timeout(Some(Duration::from_millis(50)));
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
                                maybe_capture_session_id(&state_thread, &msg);
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

        Ok(Self {
            state,
            #[cfg(feature = "client-wasm")]
            wasm: None,
        })
    }
}

fn maybe_capture_session_id(state: &Arc<State>, msg: &DiagTransportMessageV1) {
    if msg.r#type != "hello_ack" {
        return;
    }
    let Some(session_id) = msg.session_id.as_deref() else {
        return;
    };
    let Ok(mut default) = state.default_session_id.lock() else {
        return;
    };
    if default.is_none() {
        *default = Some(session_id.to_string());
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

#[cfg(feature = "client-wasm")]
fn append_token_query_param_simple(url: &str, token: &str, key: &str) -> String {
    if url.contains(&format!("{key}=")) {
        return url.to_string();
    }
    if url.contains('?') {
        format!("{url}&{key}={token}")
    } else {
        format!("{url}?{key}={token}")
    }
}

#[cfg(feature = "client-wasm")]
fn flush_wasm_outbox(ws: &web_sys::WebSocket, state: &Arc<State>) {
    if ws.ready_state() != web_sys::WebSocket::OPEN {
        return;
    }

    let Ok(mut outbox) = state.outbox.lock() else {
        return;
    };
    while let Some(msg) = outbox.pop_front() {
        let Ok(text) = serde_json::to_string(&msg) else {
            continue;
        };
        let _ = ws.send_with_str(&text);
    }
}

#[cfg(feature = "client-wasm")]
pub fn devtools_ws_config_from_window_query() -> (Option<String>, Option<String>) {
    let Some(window) = web_sys::window() else {
        return (None, None);
    };

    let location = window.location();
    let search = location.search().unwrap_or_default();
    let hash = location.hash().unwrap_or_default();

    fn read_from_params(params: &web_sys::UrlSearchParams) -> (Option<String>, Option<String>) {
        let ws_url = params.get("fret_devtools_ws");
        let token = params.get("fret_devtools_token");
        (ws_url, token)
    }

    fn parse_query_params(query: &str) -> Option<web_sys::UrlSearchParams> {
        let query = query.trim();
        if query.is_empty() {
            return None;
        }
        let query = query.trim_start_matches('?');
        web_sys::UrlSearchParams::new_with_str(query).ok()
    }

    fn parse_hash_query_params(hash: &str) -> Option<web_sys::UrlSearchParams> {
        let hash = hash.trim();
        if hash.is_empty() {
            return None;
        }

        let hash = hash.trim_start_matches('#');
        let query = hash.split_once('?').map(|(_, q)| q).unwrap_or(hash);
        parse_query_params(query)
    }

    let mut ws_url = None;
    let mut token = None;

    if let Some(params) = parse_query_params(&search) {
        let (u, t) = read_from_params(&params);
        ws_url = ws_url.or(u);
        token = token.or(t);
    }

    if let Some(params) = parse_hash_query_params(&hash) {
        let (u, t) = read_from_params(&params);
        ws_url = ws_url.or(u);
        token = token.or(t);
    }

    if ws_url.is_none() || token.is_none() {
        let ws_global =
            js_sys::Reflect::get(window.as_ref(), &JsValue::from_str("__FRET_DEVTOOLS_WS"))
                .ok()
                .and_then(|v| v.as_string());
        let token_global =
            js_sys::Reflect::get(window.as_ref(), &JsValue::from_str("__FRET_DEVTOOLS_TOKEN"))
                .ok()
                .and_then(|v| v.as_string());

        ws_url = ws_url.or(ws_global);
        token = token.or(token_global);
    }

    (ws_url, token)
}
