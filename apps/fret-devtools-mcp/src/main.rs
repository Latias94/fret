use std::collections::VecDeque;
use std::net::{IpAddr, Ipv4Addr, SocketAddr};
use std::sync::Arc;
use std::time::Duration;

use fret_diag_protocol::{DiagTransportMessageV1, UiScriptResultV1, UiScriptStageV1};
use fret_diag_ws::client::{ClientKindV1, DevtoolsWsClient, DevtoolsWsClientConfig};
use fret_diag_ws::server::{DevtoolsWsServer, DevtoolsWsServerConfig};
use rmcp::handler::server::tool::ToolRouter;
use rmcp::model::*;
use rmcp::transport::stdio;
use rmcp::{Json, ServerHandler, ServiceExt, tool, tool_handler, tool_router};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use tokio::sync::Mutex;

#[derive(Clone)]
struct WsState {
    ws_url: Arc<str>,
    token: Arc<str>,
}

#[derive(Serialize, Deserialize, JsonSchema)]
struct WsInfoV1 {
    schema_version: u32,
    ws_url: String,
    token: String,
}

#[derive(Clone)]
struct FretDevtoolsMcp {
    ws: WsState,
    client: Arc<DevtoolsWsClient>,
    inbox: Arc<Mutex<VecDeque<DiagTransportMessageV1>>>,
    tool_router: ToolRouter<Self>,
}

#[tool_router]
impl FretDevtoolsMcp {
    fn new(
        ws: WsState,
        client: Arc<DevtoolsWsClient>,
        inbox: Arc<Mutex<VecDeque<DiagTransportMessageV1>>>,
    ) -> Self {
        Self {
            ws,
            client,
            inbox,
            tool_router: Self::tool_router(),
        }
    }

    #[tool(
        name = "fret_devtools_ws_info",
        description = "Return the WS URL and capability token for connecting target apps."
    )]
    async fn ws_info(&self) -> Result<Json<WsInfoV1>, String> {
        Ok(Json(WsInfoV1 {
            schema_version: 1,
            ws_url: self.ws.ws_url.to_string(),
            token: self.ws.token.to_string(),
        }))
    }

    #[tool(description = "Set UI inspection mode (overlay on/off).")]
    async fn fret_diag_inspect_set(
        &self,
        params: rmcp::handler::server::wrapper::Parameters<InspectSetRequestV1>,
    ) -> Result<String, String> {
        self.client.send_type_payload(
            "inspect.set",
            serde_json::json!({
                "enabled": params.0.enabled,
                "consume_clicks": params.0.consume_clicks,
            }),
        );
        Ok("ok".to_string())
    }

    #[tool(description = "Arm pick and wait for a pick.result message (returns JSON text).")]
    async fn fret_diag_pick(
        &self,
        params: rmcp::handler::server::wrapper::Parameters<PickRequestV1>,
    ) -> Result<String, String> {
        self.client
            .send_type_payload("pick.arm", serde_json::json!({}));
        let msg = self
            .wait_for_type("pick.result", params.0.timeout_ms)
            .await
            .ok_or_else(|| "timeout waiting for pick.result".to_string())?;
        Ok(serde_json::to_string_pretty(&msg.payload).unwrap_or_else(|_| "{}".to_string()))
    }

    #[tool(description = "Request a bundle dump and wait for bundle.dumped (returns JSON text).")]
    async fn fret_diag_bundle_dump(
        &self,
        params: rmcp::handler::server::wrapper::Parameters<BundleDumpRequestV1>,
    ) -> Result<String, String> {
        let label = params.0.label.as_deref().unwrap_or("devtools-mcp");
        self.client
            .send_type_payload("bundle.dump", serde_json::json!({ "label": label }));
        let msg = self
            .wait_for_type("bundle.dumped", params.0.timeout_ms)
            .await
            .ok_or_else(|| "timeout waiting for bundle.dumped".to_string())?;
        Ok(serde_json::to_string_pretty(&msg.payload).unwrap_or_else(|_| "{}".to_string()))
    }

    #[tool(
        description = "Run a script (schema v1/v2) and wait for a passed/failed script.result (returns JSON text)."
    )]
    async fn fret_diag_run_script_json(
        &self,
        params: rmcp::handler::server::wrapper::Parameters<RunScriptJsonRequestV1>,
    ) -> Result<String, String> {
        let script: serde_json::Value =
            serde_json::from_str(&params.0.script_json).map_err(|e| e.to_string())?;
        self.client.send_type_payload(
            "script.run",
            serde_json::json!({
                "script": script,
            }),
        );

        let timeout_ms = params.0.timeout_ms;
        let start = tokio::time::Instant::now();
        loop {
            if start.elapsed() > Duration::from_millis(timeout_ms) {
                return Err("timeout waiting for script.result".to_string());
            }

            if let Some(msg) = self.pop_next_of_type("script.result").await {
                if let Ok(parsed) = serde_json::from_value::<UiScriptResultV1>(msg.payload.clone())
                {
                    match parsed.stage {
                        UiScriptStageV1::Passed | UiScriptStageV1::Failed => {
                            return Ok(serde_json::to_string_pretty(&parsed)
                                .unwrap_or_else(|_| "{}".to_string()));
                        }
                        _ => {}
                    }
                }
            }

            tokio::time::sleep(Duration::from_millis(20)).await;
        }
    }

    async fn wait_for_type(&self, ty: &str, timeout_ms: u64) -> Option<DiagTransportMessageV1> {
        let start = tokio::time::Instant::now();
        loop {
            if let Some(msg) = self.pop_next_of_type(ty).await {
                return Some(msg);
            }
            if start.elapsed() > Duration::from_millis(timeout_ms) {
                return None;
            }
            tokio::time::sleep(Duration::from_millis(20)).await;
        }
    }

    async fn pop_next_of_type(&self, ty: &str) -> Option<DiagTransportMessageV1> {
        let mut inbox = self.inbox.lock().await;
        let pos = inbox.iter().position(|m| m.r#type == ty)?;
        Some(inbox.remove(pos)?)
    }
}

#[tool_handler]
impl ServerHandler for FretDevtoolsMcp {
    fn get_info(&self) -> ServerInfo {
        ServerInfo {
            instructions: Some(
                "Fret diagnostics DevTools MCP adapter. Starts a local WS hub and exposes tools to drive inspect/pick/scripts/bundles."
                    .into(),
            ),
            capabilities: ServerCapabilities::builder().enable_tools().build(),
            ..Default::default()
        }
    }
}

#[derive(Serialize, Deserialize, JsonSchema)]
struct PickRequestV1 {
    timeout_ms: u64,
}

#[derive(Serialize, Deserialize, JsonSchema)]
struct BundleDumpRequestV1 {
    #[serde(default)]
    label: Option<String>,
    timeout_ms: u64,
}

#[derive(Serialize, Deserialize, JsonSchema)]
struct InspectSetRequestV1 {
    enabled: bool,
    #[serde(default = "serde_default_true")]
    consume_clicks: bool,
}

#[derive(Serialize, Deserialize, JsonSchema)]
struct RunScriptJsonRequestV1 {
    /// JSON text for a `UiActionScriptV1` or `UiActionScriptV2` payload.
    script_json: String,
    timeout_ms: u64,
}

fn serde_default_true() -> bool {
    true
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let port = env_u16("FRET_DEVTOOLS_WS_PORT").unwrap_or(7331);
    let token =
        std::env::var("FRET_DEVTOOLS_TOKEN").unwrap_or_else(|_| uuid::Uuid::new_v4().to_string());
    let bind = SocketAddr::new(IpAddr::V4(Ipv4Addr::LOCALHOST), port);

    std::thread::spawn({
        let token = token.clone();
        move || {
            let server = DevtoolsWsServer::new(DevtoolsWsServerConfig { bind, token });
            let _ = server.run();
        }
    });

    let ws_url = Arc::<str>::from(format!("ws://127.0.0.1:{port}/"));
    let token = Arc::<str>::from(token);

    let mut cfg = DevtoolsWsClientConfig::with_defaults(ws_url.to_string(), token.to_string());
    cfg.client_kind = ClientKindV1::Tooling;
    cfg.capabilities = vec![
        "inspect".to_string(),
        "pick".to_string(),
        "scripts".to_string(),
        "bundles".to_string(),
    ];
    let client = Arc::new(DevtoolsWsClient::connect_native(cfg).map_err(anyhow::Error::msg)?);

    let inbox = Arc::new(Mutex::new(VecDeque::new()));
    tokio::spawn({
        let client = Arc::clone(&client);
        let inbox = Arc::clone(&inbox);
        async move {
            loop {
                let mut drained = false;
                while let Some(msg) = client.try_recv() {
                    drained = true;
                    inbox.lock().await.push_back(msg);
                }
                if !drained {
                    tokio::time::sleep(Duration::from_millis(10)).await;
                }
            }
        }
    });

    let service = FretDevtoolsMcp::new(WsState { ws_url, token }, client, inbox)
        .serve(stdio())
        .await?;
    service.waiting().await?;
    Ok(())
}

fn env_u16(key: &str) -> Option<u16> {
    std::env::var(key).ok().and_then(|v| v.parse().ok())
}
