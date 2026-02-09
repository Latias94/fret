use std::collections::VecDeque;
use std::sync::{Arc, Mutex};

use fret_diag_protocol::{
    DevtoolsSessionDescriptorV1, DevtoolsSessionListV1, DiagTransportMessageV1,
};

mod fs;
mod ws;

pub use fs::FsDiagTransportConfig;
pub use ws::WsDiagTransportConfig;

pub use fret_diag_ws::client::ClientKindV1;
pub use fret_diag_ws::client::DevtoolsWsClientConfig;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DiagTransportKind {
    WebSocket,
    FileSystem,
}

pub trait DiagTransport: Send + Sync {
    fn kind(&self) -> DiagTransportKind;
    fn send(&self, msg: DiagTransportMessageV1);
    fn try_recv(&self) -> Option<DiagTransportMessageV1>;
    fn set_default_session_id(&self, session_id: Option<String>);
}

#[derive(Clone)]
pub struct ToolingDiagClient {
    transport: Arc<dyn DiagTransport>,
}

impl std::fmt::Debug for ToolingDiagClient {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ToolingDiagClient")
            .field("kind", &self.transport.kind())
            .finish_non_exhaustive()
    }
}

impl ToolingDiagClient {
    pub fn connect_ws(cfg: WsDiagTransportConfig) -> Result<Self, String> {
        Ok(Self {
            transport: Arc::new(ws::WsDiagTransport::connect(cfg)?),
        })
    }

    pub fn connect_fs(cfg: FsDiagTransportConfig) -> Result<Self, String> {
        Ok(Self {
            transport: Arc::new(fs::FsDiagTransport::new(cfg)),
        })
    }

    pub fn kind(&self) -> DiagTransportKind {
        self.transport.kind()
    }

    pub fn send(&self, msg: DiagTransportMessageV1) {
        self.transport.send(msg);
    }

    pub fn send_type_payload(&self, ty: impl Into<String>, payload: serde_json::Value) {
        self.send(DiagTransportMessageV1 {
            schema_version: 1,
            r#type: ty.into(),
            session_id: None,
            request_id: None,
            payload,
        });
    }

    pub fn try_recv(&self) -> Option<DiagTransportMessageV1> {
        self.transport.try_recv()
    }

    pub fn set_default_session_id(&self, session_id: Option<String>) {
        self.transport.set_default_session_id(session_id);
    }
}

#[derive(Debug, Clone)]
pub struct DiagInbox {
    q: Arc<Mutex<VecDeque<DiagTransportMessageV1>>>,
}

impl Default for DiagInbox {
    fn default() -> Self {
        Self {
            q: Arc::new(Mutex::new(VecDeque::new())),
        }
    }
}

impl DiagInbox {
    pub fn push(&self, msg: DiagTransportMessageV1) {
        if let Ok(mut q) = self.q.lock() {
            q.push_back(msg);
        }
    }

    pub fn pop(&self) -> Option<DiagTransportMessageV1> {
        self.q.lock().ok().and_then(|mut q| q.pop_front())
    }
}

pub fn fs_single_session_list(session_id: &str) -> DiagTransportMessageV1 {
    DiagTransportMessageV1 {
        schema_version: 1,
        r#type: "session.list".to_string(),
        session_id: None,
        request_id: None,
        payload: serde_json::to_value(DevtoolsSessionListV1 {
            sessions: vec![DevtoolsSessionDescriptorV1 {
                session_id: session_id.to_string(),
                client_kind: "filesystem".to_string(),
                client_version: "unknown".to_string(),
                capabilities: vec![
                    "inspect".to_string(),
                    "pick".to_string(),
                    "scripts".to_string(),
                    "bundles".to_string(),
                ],
            }],
        })
        .unwrap_or_else(|_| serde_json::json!({ "sessions": [] })),
    }
}
