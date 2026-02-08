use fret_diag_protocol::DiagTransportMessageV1;
use fret_diag_ws::client::{DevtoolsWsClient, DevtoolsWsClientConfig};

use super::{DiagTransport, DiagTransportKind};

pub struct WsDiagTransport {
    client: DevtoolsWsClient,
}

impl WsDiagTransport {
    pub fn connect(cfg: WsDiagTransportConfig) -> Result<Self, String> {
        let client = DevtoolsWsClient::connect_native(cfg.client_cfg)?;
        Ok(Self { client })
    }
}

impl DiagTransport for WsDiagTransport {
    fn kind(&self) -> DiagTransportKind {
        DiagTransportKind::WebSocket
    }

    fn send(&self, msg: DiagTransportMessageV1) {
        self.client.send(msg);
    }

    fn try_recv(&self) -> Option<DiagTransportMessageV1> {
        self.client.try_recv()
    }

    fn set_default_session_id(&self, session_id: Option<String>) {
        self.client.set_default_session_id(session_id);
    }
}

#[derive(Clone)]
pub struct WsDiagTransportConfig {
    pub(crate) client_cfg: DevtoolsWsClientConfig,
}

impl WsDiagTransportConfig {
    pub fn native(client_cfg: DevtoolsWsClientConfig) -> Self {
        Self { client_cfg }
    }
}
