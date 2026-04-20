use fret_diag_protocol::DiagTransportMessageV1;

#[cfg(not(target_arch = "wasm32"))]
use fret_diag_ws::client::{ClientKindV1, DevtoolsWsClient, DevtoolsWsClientConfig};

#[cfg(target_arch = "wasm32")]
use fret_diag_ws::client::{
    ClientKindV1, DevtoolsWsClient, DevtoolsWsClientConfig, devtools_ws_config_from_window_query,
};

pub(crate) fn devtools_ws_config() -> (Option<String>, Option<String>) {
    #[cfg(target_arch = "wasm32")]
    {
        devtools_ws_config_from_window_query()
    }

    #[cfg(not(target_arch = "wasm32"))]
    {
        let ws_url = std::env::var("FRET_DEVTOOLS_WS")
            .ok()
            .filter(|v| !v.trim().is_empty());
        let token = std::env::var("FRET_DEVTOOLS_TOKEN")
            .ok()
            .filter(|v| !v.trim().is_empty());
        (ws_url, token)
    }
}

#[derive(Debug, Default)]
pub(crate) struct UiDiagnosticsWsBridge {
    client: Option<DevtoolsWsClient>,
}

impl UiDiagnosticsWsBridge {
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
            "inspect".to_string(),
            "pick".to_string(),
            "scripts".to_string(),
            "bundles".to_string(),
            "devtools.inspect".to_string(),
            "devtools.pick".to_string(),
            "devtools.scripts".to_string(),
            "devtools.bundles".to_string(),
            "environment_sources".to_string(),
            "devtools.environment_sources".to_string(),
            "diag.script_v2".to_string(),
            "diag.pointer_kind_touch".to_string(),
            "diag.pointer_kind_pen".to_string(),
            "diag.gesture_tap".to_string(),
            "diag.gesture_long_press".to_string(),
            "diag.gesture_swipe".to_string(),
            "diag.gesture_pinch".to_string(),
            "diag.text_ime_trace".to_string(),
            "diag.text_input_snapshot".to_string(),
            "diag.shortcut_routing_trace".to_string(),
            "diag.overlay_placement_trace".to_string(),
        ];
        if !cfg!(target_arch = "wasm32") {
            caps.push("diag.multi_window".to_string());
            caps.push("diag.window_insets_override".to_string());
            if cfg!(any(target_os = "windows", target_os = "macos")) {
                caps.push("diag.window_style_patch_v1".to_string());
                caps.push("diag.platform_window_receiver_at_cursor_v1".to_string());
            }
            caps.push("diag.clipboard_force_unavailable".to_string());
            caps.push("diag.clipboard_text".to_string());
            caps.push("diag.incoming_open_inject".to_string());
            if cfg!(any(
                target_os = "windows",
                target_os = "macos",
                target_os = "linux"
            )) {
                caps.push("diag.cursor_screen_pos_override".to_string());
                caps.push("diag.mouse_buttons_override".to_string());
            }
        }
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
}
