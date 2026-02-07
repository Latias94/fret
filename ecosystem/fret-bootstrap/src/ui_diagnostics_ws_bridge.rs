use fret_diag_protocol::DiagTransportMessageV1;

#[cfg(feature = "diagnostics-ws")]
use fret_diag_ws::client::{ClientKindV1, DevtoolsWsClient, DevtoolsWsClientConfig};

#[derive(Debug, Default)]
pub(crate) struct UiDiagnosticsWsBridge {
    #[cfg(feature = "diagnostics-ws")]
    client: Option<DevtoolsWsClient>,
}

impl UiDiagnosticsWsBridge {
    #[cfg(feature = "diagnostics-ws")]
    pub(crate) fn ensure_connected(
        &mut self,
        ws_url: Option<&str>,
        token: Option<&str>,
    ) -> Option<&DevtoolsWsClient> {
        if self.client.is_some() {
            return self.client.as_ref();
        }

        let (ws_url, token) = (ws_url?, token?);
        if ws_url.trim().is_empty() || token.trim().is_empty() {
            return None;
        }

        let mut cfg = DevtoolsWsClientConfig::with_defaults(ws_url.to_string(), token.to_string());
        cfg.client_kind = if cfg!(target_arch = "wasm32") {
            ClientKindV1::WebApp
        } else {
            ClientKindV1::NativeApp
        };
        cfg.capabilities = vec![
            "inspect".to_string(),
            "pick".to_string(),
            "scripts".to_string(),
            "bundles".to_string(),
        ];

        #[cfg(target_arch = "wasm32")]
        let client = DevtoolsWsClient::connect_wasm(cfg).ok()?;
        #[cfg(not(target_arch = "wasm32"))]
        let client = DevtoolsWsClient::connect_native(cfg).ok()?;

        self.client = Some(client);
        self.client.as_ref()
    }

    #[cfg(not(feature = "diagnostics-ws"))]
    pub(crate) fn ensure_connected(
        &mut self,
        _ws_url: Option<&str>,
        _token: Option<&str>,
    ) -> Option<()> {
        None
    }

    #[cfg(feature = "diagnostics-ws")]
    pub(crate) fn drain_inbox(
        &mut self,
        ws_url: Option<&str>,
        token: Option<&str>,
        out: &mut Vec<DiagTransportMessageV1>,
    ) {
        let Some(client) = self.ensure_connected(ws_url, token) else {
            return;
        };

        while let Some(msg) = client.try_recv() {
            out.push(msg);
        }
    }

    #[cfg(not(feature = "diagnostics-ws"))]
    pub(crate) fn drain_inbox(
        &mut self,
        _ws_url: Option<&str>,
        _token: Option<&str>,
        _out: &mut Vec<DiagTransportMessageV1>,
    ) {
    }

    #[cfg(feature = "diagnostics-ws")]
    pub(crate) fn send(
        &mut self,
        ws_url: Option<&str>,
        token: Option<&str>,
        msg: DiagTransportMessageV1,
    ) {
        let Some(client) = self.ensure_connected(ws_url, token) else {
            return;
        };
        client.send(msg);
    }

    #[cfg(not(feature = "diagnostics-ws"))]
    pub(crate) fn send(
        &mut self,
        _ws_url: Option<&str>,
        _token: Option<&str>,
        _msg: DiagTransportMessageV1,
    ) {
    }
}
