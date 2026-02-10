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
        screenshots_enabled: bool,
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
        let mut caps = vec![
            // Backwards-compatible (legacy, un-namespaced) control plane capabilities.
            "inspect".to_string(),
            "pick".to_string(),
            "scripts".to_string(),
            "bundles".to_string(),
            // Namespaced control plane capabilities (recommended).
            "devtools.inspect".to_string(),
            "devtools.pick".to_string(),
            "devtools.scripts".to_string(),
            "devtools.bundles".to_string(),
            // Runner/diagnostics capabilities (used for fail-fast gating).
            "diag.script_v2".to_string(),
            "diag.text_ime_trace".to_string(),
            "diag.text_input_snapshot".to_string(),
            "diag.shortcut_routing_trace".to_string(),
        ];
        if screenshots_enabled {
            caps.push("diag.screenshot_png".to_string());
        }
        cfg.capabilities = caps;

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
        _screenshots_enabled: bool,
    ) -> Option<()> {
        None
    }

    #[cfg(feature = "diagnostics-ws")]
    pub(crate) fn drain_inbox(
        &mut self,
        ws_url: Option<&str>,
        token: Option<&str>,
        screenshots_enabled: bool,
        out: &mut Vec<DiagTransportMessageV1>,
    ) {
        let Some(client) = self.ensure_connected(ws_url, token, screenshots_enabled) else {
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
        _screenshots_enabled: bool,
        _out: &mut Vec<DiagTransportMessageV1>,
    ) {
    }

    #[cfg(feature = "diagnostics-ws")]
    pub(crate) fn send(
        &mut self,
        ws_url: Option<&str>,
        token: Option<&str>,
        screenshots_enabled: bool,
        msg: DiagTransportMessageV1,
    ) {
        let Some(client) = self.ensure_connected(ws_url, token, screenshots_enabled) else {
            return;
        };
        client.send(msg);
    }

    #[cfg(not(feature = "diagnostics-ws"))]
    pub(crate) fn send(
        &mut self,
        _ws_url: Option<&str>,
        _token: Option<&str>,
        _screenshots_enabled: bool,
        _msg: DiagTransportMessageV1,
    ) {
    }
}
