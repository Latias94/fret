use std::collections::VecDeque;
use std::net::{IpAddr, Ipv4Addr, SocketAddr};
use std::sync::Arc;
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::Duration;

use fret_diag_protocol::{
    DevtoolsSessionAddedV1, DevtoolsSessionDescriptorV1, DevtoolsSessionListV1,
    DevtoolsSessionRemovedV1, DiagTransportMessageV1, UiScriptResultV1, UiScriptStageV1,
};
use fret_diag_ws::client::{ClientKindV1, DevtoolsWsClient, DevtoolsWsClientConfig};
use fret_diag_ws::server::{DevtoolsWsServer, DevtoolsWsServerConfig};
use rmcp::handler::server::tool::ToolRouter;
use rmcp::model::*;
use rmcp::transport::stdio;
use rmcp::{Json, ServerHandler, ServiceExt, tool, tool_handler, tool_router};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use tokio::sync::Mutex;

static NEXT_REQUEST_ID: AtomicU64 = AtomicU64::new(1000);

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
    sessions: Arc<Mutex<Vec<DevtoolsSessionDescriptorV1>>>,
    selected_session_id: Arc<Mutex<Option<String>>>,
    tool_router: ToolRouter<Self>,
}

#[tool_router]
impl FretDevtoolsMcp {
    fn new(
        ws: WsState,
        client: Arc<DevtoolsWsClient>,
        inbox: Arc<Mutex<VecDeque<DiagTransportMessageV1>>>,
        sessions: Arc<Mutex<Vec<DevtoolsSessionDescriptorV1>>>,
        selected_session_id: Arc<Mutex<Option<String>>>,
    ) -> Self {
        Self {
            ws,
            client,
            inbox,
            sessions,
            selected_session_id,
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

    #[tool(description = "List active diagnostics sessions (connected apps).")]
    async fn fret_diag_sessions_list(&self) -> Result<Json<SessionsListV1>, String> {
        let sessions = self.sessions.lock().await.clone();
        Ok(Json(SessionsListV1 {
            schema_version: 1,
            sessions: sessions
                .into_iter()
                .map(|s| SessionInfoV1 {
                    session_id: s.session_id,
                    client_kind: s.client_kind,
                    client_version: s.client_version,
                    capabilities: s.capabilities,
                })
                .collect(),
        }))
    }

    #[tool(description = "Select a default session_id for subsequent commands.")]
    async fn fret_diag_sessions_select(
        &self,
        params: rmcp::handler::server::wrapper::Parameters<SessionsSelectRequestV1>,
    ) -> Result<String, String> {
        let session_id = params.0.session_id;
        let sessions = self.sessions.lock().await;
        if !sessions.iter().any(|s| s.session_id == session_id) {
            return Err("unknown session_id (use fret_diag_sessions_list)".to_string());
        }
        drop(sessions);

        *self.selected_session_id.lock().await = Some(session_id.clone());
        self.client.set_default_session_id(Some(session_id));
        Ok("ok".to_string())
    }

    #[tool(description = "Set UI inspection mode (overlay on/off).")]
    async fn fret_diag_inspect_set(
        &self,
        params: rmcp::handler::server::wrapper::Parameters<InspectSetRequestV1>,
    ) -> Result<String, String> {
        let session_id = self.resolve_session_id(params.0.session_id).await?;
        self.client.send(DiagTransportMessageV1 {
            schema_version: 1,
            r#type: "inspect.set".to_string(),
            session_id: Some(session_id),
            request_id: None,
            payload: serde_json::json!({
                "enabled": params.0.enabled,
                "consume_clicks": params.0.consume_clicks,
            }),
        });
        Ok("ok".to_string())
    }

    #[tool(description = "Arm pick and wait for a pick.result message (returns JSON text).")]
    async fn fret_diag_pick(
        &self,
        params: rmcp::handler::server::wrapper::Parameters<PickRequestV1>,
    ) -> Result<String, String> {
        let session_id = self.resolve_session_id(params.0.session_id).await?;
        self.client.send(DiagTransportMessageV1 {
            schema_version: 1,
            r#type: "pick.arm".to_string(),
            session_id: Some(session_id.clone()),
            request_id: None,
            payload: serde_json::json!({}),
        });
        let msg = self
            .wait_for_type_and_session("pick.result", &session_id, params.0.timeout_ms)
            .await
            .ok_or_else(|| "timeout waiting for pick.result".to_string())?;
        Ok(serde_json::to_string_pretty(&msg.payload).unwrap_or_else(|_| "{}".to_string()))
    }

    #[tool(description = "Request a bundle dump and wait for bundle.dumped (returns JSON text).")]
    async fn fret_diag_bundle_dump(
        &self,
        params: rmcp::handler::server::wrapper::Parameters<BundleDumpRequestV1>,
    ) -> Result<String, String> {
        let session_id = self.resolve_session_id(params.0.session_id.clone()).await?;
        let label = params.0.label.as_deref().unwrap_or("devtools-mcp");
        self.client.send(DiagTransportMessageV1 {
            schema_version: 1,
            r#type: "bundle.dump".to_string(),
            session_id: Some(session_id.clone()),
            request_id: None,
            payload: serde_json::json!({ "label": label }),
        });
        let msg = self
            .wait_for_type_and_session("bundle.dumped", &session_id, params.0.timeout_ms)
            .await
            .ok_or_else(|| "timeout waiting for bundle.dumped".to_string())?;
        Ok(serde_json::to_string_pretty(&msg.payload).unwrap_or_else(|_| "{}".to_string()))
    }

    #[tool(
        description = "Request a screenshot capture and wait for screenshot.result (returns JSON text)."
    )]
    async fn fret_diag_screenshot_request(
        &self,
        params: rmcp::handler::server::wrapper::Parameters<ScreenshotRequestToolV1>,
    ) -> Result<String, String> {
        let session_id = self.resolve_session_id(params.0.session_id.clone()).await?;
        let label = params.0.label.as_deref().unwrap_or("devtools-mcp");
        let timeout_frames = params.0.timeout_frames.unwrap_or(300);

        let request_id = NEXT_REQUEST_ID.fetch_add(1, Ordering::Relaxed);
        self.client.send(DiagTransportMessageV1 {
            schema_version: 1,
            r#type: "screenshot.request".to_string(),
            session_id: Some(session_id.clone()),
            request_id: Some(request_id),
            payload: serde_json::json!({
                "label": label,
                "timeout_frames": timeout_frames,
            }),
        });

        let msg = self
            .wait_for_type_session_request_id(
                "screenshot.result",
                &session_id,
                request_id,
                params.0.timeout_ms,
            )
            .await
            .ok_or_else(|| "timeout waiting for screenshot.result".to_string())?;
        Ok(serde_json::to_string_pretty(&msg.payload).unwrap_or_else(|_| "{}".to_string()))
    }

    #[tool(
        description = "Run a script (schema v1/v2) and wait for a passed/failed script.result (returns JSON text)."
    )]
    async fn fret_diag_run_script_json(
        &self,
        params: rmcp::handler::server::wrapper::Parameters<RunScriptJsonRequestV1>,
    ) -> Result<String, String> {
        let session_id = self.resolve_session_id(params.0.session_id.clone()).await?;
        let script: serde_json::Value =
            serde_json::from_str(&params.0.script_json).map_err(|e| e.to_string())?;
        self.client.send(DiagTransportMessageV1 {
            schema_version: 1,
            r#type: "script.run".to_string(),
            session_id: Some(session_id.clone()),
            request_id: None,
            payload: serde_json::json!({
                "script": script,
            }),
        });

        let timeout_ms = params.0.timeout_ms;
        let start = tokio::time::Instant::now();
        loop {
            if start.elapsed() > Duration::from_millis(timeout_ms) {
                return Err("timeout waiting for script.result".to_string());
            }

            if let Some(msg) = self
                .pop_next_of_type_and_session("script.result", &session_id)
                .await
            {
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

    async fn wait_for_type_and_session(
        &self,
        ty: &str,
        session_id: &str,
        timeout_ms: u64,
    ) -> Option<DiagTransportMessageV1> {
        let start = tokio::time::Instant::now();
        loop {
            if let Some(msg) = self.pop_next_of_type_and_session(ty, session_id).await {
                return Some(msg);
            }
            if start.elapsed() > Duration::from_millis(timeout_ms) {
                return None;
            }
            tokio::time::sleep(Duration::from_millis(20)).await;
        }
    }

    async fn wait_for_type_session_request_id(
        &self,
        ty: &str,
        session_id: &str,
        request_id: u64,
        timeout_ms: u64,
    ) -> Option<DiagTransportMessageV1> {
        let start = tokio::time::Instant::now();
        loop {
            if let Some(msg) = self
                .pop_next_of_type_session_request_id(ty, session_id, request_id)
                .await
            {
                return Some(msg);
            }
            if start.elapsed() > Duration::from_millis(timeout_ms) {
                return None;
            }
            tokio::time::sleep(Duration::from_millis(20)).await;
        }
    }

    async fn pop_next_of_type_and_session(
        &self,
        ty: &str,
        session_id: &str,
    ) -> Option<DiagTransportMessageV1> {
        let mut inbox = self.inbox.lock().await;
        let pos = inbox
            .iter()
            .position(|m| m.r#type == ty && m.session_id.as_deref() == Some(session_id))?;
        Some(inbox.remove(pos)?)
    }

    async fn pop_next_of_type_session_request_id(
        &self,
        ty: &str,
        session_id: &str,
        request_id: u64,
    ) -> Option<DiagTransportMessageV1> {
        let mut inbox = self.inbox.lock().await;
        let pos = inbox.iter().position(|m| {
            m.r#type == ty
                && m.session_id.as_deref() == Some(session_id)
                && m.request_id == Some(request_id)
        })?;
        Some(inbox.remove(pos)?)
    }

    async fn resolve_session_id(&self, requested: Option<String>) -> Result<String, String> {
        if let Some(s) = requested {
            return Ok(s);
        }

        let selected = self.selected_session_id.lock().await.clone();
        if let Some(s) = selected {
            return Ok(s);
        }

        let sessions = self.sessions.lock().await;
        if let Some(first) = sessions.first() {
            return Ok(first.session_id.clone());
        }

        Err("no sessions available (connect an app and call fret_diag_sessions_list)".to_string())
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
    #[serde(default)]
    session_id: Option<String>,
    timeout_ms: u64,
}

#[derive(Serialize, Deserialize, JsonSchema)]
struct BundleDumpRequestV1 {
    #[serde(default)]
    session_id: Option<String>,
    #[serde(default)]
    label: Option<String>,
    timeout_ms: u64,
}

#[derive(Serialize, Deserialize, JsonSchema)]
struct InspectSetRequestV1 {
    #[serde(default)]
    session_id: Option<String>,
    enabled: bool,
    #[serde(default = "serde_default_true")]
    consume_clicks: bool,
}

#[derive(Serialize, Deserialize, JsonSchema)]
struct RunScriptJsonRequestV1 {
    #[serde(default)]
    session_id: Option<String>,
    /// JSON text for a `UiActionScriptV1` or `UiActionScriptV2` payload.
    script_json: String,
    timeout_ms: u64,
}

#[derive(Serialize, Deserialize, JsonSchema)]
struct ScreenshotRequestToolV1 {
    #[serde(default)]
    session_id: Option<String>,
    #[serde(default)]
    label: Option<String>,
    #[serde(default)]
    timeout_frames: Option<u32>,
    timeout_ms: u64,
}

#[derive(Serialize, Deserialize, JsonSchema)]
struct SessionsListV1 {
    schema_version: u32,
    sessions: Vec<SessionInfoV1>,
}

#[derive(Serialize, Deserialize, JsonSchema)]
struct SessionInfoV1 {
    session_id: String,
    client_kind: String,
    client_version: String,
    #[serde(default)]
    capabilities: Vec<String>,
}

#[derive(Serialize, Deserialize, JsonSchema)]
struct SessionsSelectRequestV1 {
    session_id: String,
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
    let sessions = Arc::new(Mutex::new(Vec::<DevtoolsSessionDescriptorV1>::new()));
    let selected_session_id = Arc::new(Mutex::new(None::<String>));
    tokio::spawn({
        let client = Arc::clone(&client);
        let inbox = Arc::clone(&inbox);
        let sessions = Arc::clone(&sessions);
        let selected_session_id = Arc::clone(&selected_session_id);
        async move {
            loop {
                let mut drained = false;
                while let Some(msg) = client.try_recv() {
                    drained = true;

                    match msg.r#type.as_str() {
                        "session.list" => {
                            if let Ok(parsed) =
                                serde_json::from_value::<DevtoolsSessionListV1>(msg.payload.clone())
                            {
                                *sessions.lock().await = parsed.sessions;
                            }
                        }
                        "session.added" => {
                            if let Ok(parsed) = serde_json::from_value::<DevtoolsSessionAddedV1>(
                                msg.payload.clone(),
                            ) {
                                let mut s = sessions.lock().await;
                                if let Some(pos) = s
                                    .iter()
                                    .position(|x| x.session_id == parsed.session.session_id)
                                {
                                    s[pos] = parsed.session;
                                } else {
                                    s.push(parsed.session);
                                }
                            }
                        }
                        "session.removed" => {
                            if let Ok(parsed) = serde_json::from_value::<DevtoolsSessionRemovedV1>(
                                msg.payload.clone(),
                            ) {
                                let mut s = sessions.lock().await;
                                s.retain(|x| x.session_id != parsed.session_id);
                            }
                        }
                        _ => {}
                    }

                    {
                        let (first, contains_selected) = {
                            let s = sessions.lock().await;
                            let first = s.first().map(|x| x.session_id.clone());
                            let current = selected_session_id.lock().await.clone();
                            let contains_selected = current
                                .as_deref()
                                .is_some_and(|sel| s.iter().any(|x| x.session_id == sel));
                            (first, contains_selected)
                        };

                        let mut selected = selected_session_id.lock().await;
                        if selected.is_none() {
                            *selected = first.clone();
                            client.set_default_session_id(first);
                        } else if !contains_selected {
                            *selected = first.clone();
                            client.set_default_session_id(first);
                        }
                    }

                    inbox.lock().await.push_back(msg);
                }
                if !drained {
                    tokio::time::sleep(Duration::from_millis(10)).await;
                }
            }
        }
    });

    let service = FretDevtoolsMcp::new(
        WsState { ws_url, token },
        client,
        inbox,
        sessions,
        selected_session_id,
    )
    .serve(stdio())
    .await?;
    service.waiting().await?;
    Ok(())
}

fn env_u16(key: &str) -> Option<u16> {
    std::env::var(key).ok().and_then(|v| v.parse().ok())
}
