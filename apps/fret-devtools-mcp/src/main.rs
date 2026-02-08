use std::collections::{HashSet, VecDeque};
use std::net::{IpAddr, Ipv4Addr, SocketAddr};
use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::Duration;

use base64::Engine;
use fret_diag::artifacts;
use fret_diag::transport::{
    ClientKindV1, DevtoolsWsClientConfig, DiagTransportKind, FsDiagTransportConfig,
    ToolingDiagClient, WsDiagTransportConfig,
};
use fret_diag_protocol::{
    DevtoolsSessionAddedV1, DevtoolsSessionDescriptorV1, DevtoolsSessionListV1,
    DevtoolsSessionRemovedV1, DiagTransportMessageV1, UiScriptResultV1, UiScriptStageV1,
};
use fret_diag_ws::server::{DevtoolsWsServer, DevtoolsWsServerConfig};
use rmcp::handler::server::tool::ToolRouter;
use rmcp::model::*;
use rmcp::transport::stdio;
use rmcp::{
    ErrorData as McpError, Json, ServerHandler, ServiceExt, tool, tool_handler, tool_router,
};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use tokio::sync::Mutex;
use tokio::sync::mpsc;
use tokio::sync::oneshot;

static NEXT_REQUEST_ID: AtomicU64 = AtomicU64::new(1000);

const RESOURCE_SCHEME: &str = "fret-diag://";
const RESOURCE_KIND_BUNDLE_JSON: &str = "bundle.json";
const RESOURCE_KIND_BUNDLE_ZIP: &str = "bundle.zip";
const RESOURCE_KIND_REPRO_SUMMARY_JSON: &str = "repro.summary.json";

#[derive(Clone)]
struct WsState {
    ws_url: Arc<str>,
    token: Arc<str>,
}

#[derive(Debug, Clone)]
struct ConnectConfig {
    kind: DiagTransportKind,
    ws_url: Option<String>,
    token: Option<String>,
    fs_out_dir: Option<String>,
}

#[derive(Debug)]
enum ClientCommand {
    Send(DiagTransportMessageV1),
    Connect(ConnectConfig, oneshot::Sender<Result<(), String>>),
    SetDefaultSessionId(Option<String>),
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
    client_tx: mpsc::UnboundedSender<ClientCommand>,
    client_kind: Arc<Mutex<DiagTransportKind>>,
    inbox: Arc<Mutex<VecDeque<DiagTransportMessageV1>>>,
    sessions: Arc<Mutex<Vec<DevtoolsSessionDescriptorV1>>>,
    selected_session_id: Arc<Mutex<Option<String>>>,
    peer: Arc<Mutex<Option<rmcp::Peer<rmcp::RoleServer>>>>,
    subscribed_resources: Arc<Mutex<HashSet<String>>>,
    tool_router: ToolRouter<Self>,
}

#[tool_router]
impl FretDevtoolsMcp {
    fn new(
        ws: WsState,
        client_tx: mpsc::UnboundedSender<ClientCommand>,
        client_kind: Arc<Mutex<DiagTransportKind>>,
        inbox: Arc<Mutex<VecDeque<DiagTransportMessageV1>>>,
        sessions: Arc<Mutex<Vec<DevtoolsSessionDescriptorV1>>>,
        selected_session_id: Arc<Mutex<Option<String>>>,
        peer: Arc<Mutex<Option<rmcp::Peer<rmcp::RoleServer>>>>,
        subscribed_resources: Arc<Mutex<HashSet<String>>>,
    ) -> Self {
        Self {
            ws,
            client_tx,
            client_kind,
            inbox,
            sessions,
            selected_session_id,
            peer,
            subscribed_resources,
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
        let _ = self
            .client_tx
            .send(ClientCommand::SetDefaultSessionId(Some(session_id)));
        Ok("ok".to_string())
    }

    #[tool(description = "Connect (or switch) the diagnostics transport for subsequent commands.")]
    async fn fret_diag_connect(
        &self,
        params: rmcp::handler::server::wrapper::Parameters<ConnectRequestV1>,
    ) -> Result<Json<ConnectResultV1>, String> {
        let kind = match params.0.transport.trim().to_lowercase().as_str() {
            "ws" | "websocket" => DiagTransportKind::WebSocket,
            "fs" | "filesystem" => DiagTransportKind::FileSystem,
            other => return Err(format!("unsupported transport: {other}")),
        };

        let resolved_ws_url = (kind == DiagTransportKind::WebSocket).then(|| {
            params
                .0
                .ws_url
                .as_deref()
                .map(|s| s.trim().to_string())
                .filter(|s| !s.is_empty())
                .unwrap_or_else(|| self.ws.ws_url.to_string())
        });
        let resolved_fs_out_dir = (kind == DiagTransportKind::FileSystem).then(|| {
            params
                .0
                .fs_out_dir
                .as_deref()
                .map(|s| s.trim().to_string())
                .filter(|s| !s.is_empty())
                .or_else(|| std::env::var("FRET_DIAG_DIR").ok())
                .unwrap_or_else(|| "target/fret-diag".to_string())
        });

        {
            let mut inbox = self.inbox.lock().await;
            inbox.clear();
        }
        *self.sessions.lock().await = Vec::new();
        *self.selected_session_id.lock().await = None;

        let (tx, rx) = oneshot::channel();
        self.client_tx
            .send(ClientCommand::Connect(
                ConnectConfig {
                    kind,
                    ws_url: params.0.ws_url.clone(),
                    token: params.0.token.clone(),
                    fs_out_dir: params.0.fs_out_dir.clone(),
                },
                tx,
            ))
            .map_err(|_| "client task is not running".to_string())?;
        rx.await
            .map_err(|_| "client connect ack dropped".to_string())?
            .map_err(|e| e.to_string())?;

        *self.client_kind.lock().await = kind;

        Ok(Json(ConnectResultV1 {
            schema_version: 1,
            transport: match kind {
                DiagTransportKind::WebSocket => "ws".to_string(),
                DiagTransportKind::FileSystem => "fs".to_string(),
            },
            ws_url: resolved_ws_url,
            token: (kind == DiagTransportKind::WebSocket).then(|| "<redacted>".to_string()),
            fs_out_dir: resolved_fs_out_dir,
        }))
    }

    #[tool(description = "Set UI inspection mode (overlay on/off).")]
    async fn fret_diag_inspect_set(
        &self,
        params: rmcp::handler::server::wrapper::Parameters<InspectSetRequestV1>,
    ) -> Result<String, String> {
        let session_id = self.resolve_session_id(params.0.session_id).await?;
        self.client_tx
            .send(ClientCommand::Send(DiagTransportMessageV1 {
                schema_version: 1,
                r#type: "inspect.set".to_string(),
                session_id: Some(session_id),
                request_id: None,
                payload: serde_json::json!({
                    "enabled": params.0.enabled,
                    "consume_clicks": params.0.consume_clicks,
                }),
            }))
            .map_err(|_| "client task is not running".to_string())?;
        Ok("ok".to_string())
    }

    #[tool(description = "Arm pick and wait for a pick.result message (returns JSON text).")]
    async fn fret_diag_pick(
        &self,
        params: rmcp::handler::server::wrapper::Parameters<PickRequestV1>,
    ) -> Result<String, String> {
        let session_id = self.resolve_session_id(params.0.session_id).await?;
        self.client_tx
            .send(ClientCommand::Send(DiagTransportMessageV1 {
                schema_version: 1,
                r#type: "pick.arm".to_string(),
                session_id: Some(session_id.clone()),
                request_id: None,
                payload: serde_json::json!({}),
            }))
            .map_err(|_| "client task is not running".to_string())?;
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
        self.client_tx
            .send(ClientCommand::Send(DiagTransportMessageV1 {
                schema_version: 1,
                r#type: "bundle.dump".to_string(),
                session_id: Some(session_id.clone()),
                request_id: None,
                payload: serde_json::json!({ "label": label }),
            }))
            .map_err(|_| "client task is not running".to_string())?;
        let msg = self
            .wait_for_type_and_session("bundle.dumped", &session_id, params.0.timeout_ms)
            .await
            .ok_or_else(|| "timeout waiting for bundle.dumped".to_string())?;
        Ok(serde_json::to_string_pretty(&msg.payload).unwrap_or_else(|_| "{}".to_string()))
    }

    #[tool(
        description = "Pack the latest bundle into a repro zip. Always performs a fresh bundle dump first."
    )]
    async fn fret_diag_pack_last_bundle(
        &self,
        params: rmcp::handler::server::wrapper::Parameters<PackLastBundleRequestV1>,
    ) -> Result<Json<PackLastBundleResultV1>, String> {
        let session_id = self.resolve_session_id(params.0.session_id.clone()).await?;
        let label = params.0.label.as_deref().unwrap_or("devtools-mcp");
        let include_all = params.0.include_all.unwrap_or(true);

        self.client_tx
            .send(ClientCommand::Send(DiagTransportMessageV1 {
                schema_version: 1,
                r#type: "bundle.dump".to_string(),
                session_id: Some(session_id.clone()),
                request_id: None,
                payload: serde_json::json!({ "label": label }),
            }))
            .map_err(|_| "client task is not running".to_string())?;

        let dumped = self
            .wait_for_type_and_session("bundle.dumped", &session_id, params.0.timeout_ms)
            .await
            .ok_or_else(|| "timeout waiting for bundle.dumped".to_string())?;

        let repo_root = repo_root_from_manifest_dir()
            .or_else(|| std::env::current_dir().ok())
            .ok_or_else(|| "failed to resolve repo root".to_string())?;

        let (out_dir_arg, bundle_dir_arg) = materialize_or_resolve_bundle_dir(
            &repo_root,
            &dumped.payload,
            params.0.export_out_dir.as_deref(),
        )?;

        let pack_out = match params.0.pack_out.as_deref() {
            Some(path) if !path.trim().is_empty() => PathBuf::from(path.trim()),
            _ => default_pack_out_path(&repo_root, &bundle_dir_arg),
        };

        let mut args = vec![
            "--dir".to_string(),
            out_dir_arg.clone(),
            "--pack-out".to_string(),
            pack_out.to_string_lossy().to_string(),
        ];
        if include_all {
            args.push("--include-all".to_string());
        }
        args.push("pack".to_string());
        args.push(bundle_dir_arg.clone());

        tokio::task::spawn_blocking(move || fret_diag::diag_cmd(args))
            .await
            .map_err(|e| e.to_string())?
            .map_err(|e| e.to_string())?;

        Ok(Json(PackLastBundleResultV1 {
            schema_version: 1,
            out_dir: out_dir_arg,
            bundle_dir: bundle_dir_arg,
            pack_path: pack_out.to_string_lossy().to_string(),
            bundle_dumped_json: serde_json::to_string_pretty(&dumped.payload)
                .unwrap_or_else(|_| "{}".to_string()),
        }))
    }

    #[tool(description = "Return the most recent bundle.dumped payload currently in the inbox.")]
    async fn fret_diag_bundle_dump_latest(
        &self,
        params: rmcp::handler::server::wrapper::Parameters<BundleDumpLatestRequestV1>,
    ) -> Result<Json<BundleDumpLatestResultV1>, String> {
        let session_id = self.resolve_session_id(params.0.session_id).await?;
        let inbox = self.inbox.lock().await;
        let msg = inbox
            .iter()
            .rev()
            .find(|m| m.r#type == "bundle.dumped" && m.session_id.as_deref() == Some(&session_id))
            .cloned();
        drop(inbox);

        let Some(msg) = msg else {
            return Ok(Json(BundleDumpLatestResultV1 {
                schema_version: 1,
                found: false,
                payload_json: None,
            }));
        };

        Ok(Json(BundleDumpLatestResultV1 {
            schema_version: 1,
            found: true,
            payload_json: Some(
                serde_json::to_string_pretty(&msg.payload).unwrap_or_else(|_| "{}".to_string()),
            ),
        }))
    }

    #[tool(
        description = "Return the latest bundle.json text (from the most recent bundle.dumped payload)."
    )]
    async fn fret_diag_bundle_json_latest(
        &self,
        params: rmcp::handler::server::wrapper::Parameters<BundleJsonLatestRequestV1>,
    ) -> Result<Json<BundleJsonLatestResultV1>, String> {
        let session_id = self.resolve_session_id(params.0.session_id).await?;

        let dumped_payload = {
            let inbox = self.inbox.lock().await;
            inbox
                .iter()
                .rev()
                .find(|m| {
                    m.r#type == "bundle.dumped" && m.session_id.as_deref() == Some(&session_id)
                })
                .map(|m| m.payload.clone())
        };

        let Some(dumped_payload) = dumped_payload else {
            return Ok(Json(BundleJsonLatestResultV1 {
                schema_version: 1,
                found: false,
                bundle_json: None,
            }));
        };

        let repo_root = repo_root_from_manifest_dir()
            .or_else(|| std::env::current_dir().ok())
            .ok_or_else(|| "failed to resolve repo root".to_string())?;
        let bundle_json = bundle_json_from_bundle_dumped_payload(&repo_root, &dumped_payload)?;

        Ok(Json(BundleJsonLatestResultV1 {
            schema_version: 1,
            found: true,
            bundle_json: Some(bundle_json),
        }))
    }

    #[tool(
        description = "Create a zip (base64) containing bundle.json. Always performs a fresh bundle dump first."
    )]
    async fn fret_diag_pack_last_bundle_zip_bytes(
        &self,
        params: rmcp::handler::server::wrapper::Parameters<PackLastBundleZipBytesRequestV1>,
    ) -> Result<Json<PackLastBundleZipBytesResultV1>, String> {
        let session_id = self.resolve_session_id(params.0.session_id.clone()).await?;
        let label = params.0.label.as_deref().unwrap_or("devtools-mcp");

        self.client_tx
            .send(ClientCommand::Send(DiagTransportMessageV1 {
                schema_version: 1,
                r#type: "bundle.dump".to_string(),
                session_id: Some(session_id.clone()),
                request_id: None,
                payload: serde_json::json!({ "label": label }),
            }))
            .map_err(|_| "client task is not running".to_string())?;

        let dumped = self
            .wait_for_type_and_session("bundle.dumped", &session_id, params.0.timeout_ms)
            .await
            .ok_or_else(|| "timeout waiting for bundle.dumped".to_string())?;

        let repo_root = repo_root_from_manifest_dir()
            .or_else(|| std::env::current_dir().ok())
            .ok_or_else(|| "failed to resolve repo root".to_string())?;
        let bundle_json = bundle_json_from_bundle_dumped_payload(&repo_root, &dumped.payload)?;

        let zip_bytes = artifacts::pack_bundle_json_to_zip_bytes(&bundle_json)?;
        let zip_base64 = base64::engine::general_purpose::STANDARD.encode(zip_bytes);

        Ok(Json(PackLastBundleZipBytesResultV1 {
            schema_version: 1,
            zip_base64,
            bundle_dumped_json: serde_json::to_string_pretty(&dumped.payload)
                .unwrap_or_else(|_| "{}".to_string()),
        }))
    }

    #[tool(
        description = "Compare two bundles (bundle.json paths or containing dirs) and return a JSON report."
    )]
    async fn fret_diag_compare(
        &self,
        params: rmcp::handler::server::wrapper::Parameters<CompareBundlesRequestV1>,
    ) -> Result<Json<CompareBundlesResultV1>, String> {
        let repo_root = repo_root_from_manifest_dir()
            .or_else(|| std::env::current_dir().ok())
            .ok_or_else(|| "failed to resolve repo root".to_string())?;

        let a_src = resolve_repo_path(&repo_root, &params.0.a);
        let b_src = resolve_repo_path(&repo_root, &params.0.b);
        let a_bundle = resolve_bundle_json_path(&a_src);
        let b_bundle = resolve_bundle_json_path(&b_src);

        let opts = fret_diag::api::CompareOptionsV1 {
            warmup_frames: params.0.warmup_frames.unwrap_or(0),
            eps_px: params.0.eps_px.unwrap_or(0.5),
            ignore_bounds: params.0.ignore_bounds.unwrap_or(false),
            ignore_scene_fingerprint: params.0.ignore_scene_fingerprint.unwrap_or(false),
        };
        let report = fret_diag::api::compare_bundles_to_json(&a_bundle, &b_bundle, opts)?;
        let ok = report.get("ok").and_then(|v| v.as_bool()).unwrap_or(false);

        Ok(Json(CompareBundlesResultV1 {
            schema_version: 1,
            ok,
            report_json: serde_json::to_string_pretty(&report).unwrap_or_else(|_| "{}".to_string()),
        }))
    }

    #[tool(
        description = "Request a screenshot capture and wait for screenshot.result (returns JSON text)."
    )]
    async fn fret_diag_screenshot_request(
        &self,
        params: rmcp::handler::server::wrapper::Parameters<ScreenshotRequestToolV1>,
    ) -> Result<String, String> {
        let kind = *self.client_kind.lock().await;
        if kind != DiagTransportKind::WebSocket {
            return Err("screenshot.request requires WebSocket transport".to_string());
        }

        let session_id = self.resolve_session_id(params.0.session_id.clone()).await?;
        let label = params.0.label.as_deref().unwrap_or("devtools-mcp");
        let timeout_frames = params.0.timeout_frames.unwrap_or(300);

        let request_id = NEXT_REQUEST_ID.fetch_add(1, Ordering::Relaxed);
        self.client_tx
            .send(ClientCommand::Send(DiagTransportMessageV1 {
                schema_version: 1,
                r#type: "screenshot.request".to_string(),
                session_id: Some(session_id.clone()),
                request_id: Some(request_id),
                payload: serde_json::json!({
                    "label": label,
                    "timeout_frames": timeout_frames,
                }),
            }))
            .map_err(|_| "client task is not running".to_string())?;

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
        self.run_script_value_and_wait(&session_id, script, params.0.timeout_ms)
            .await
    }

    #[tool(
        description = "List available diagnostics scripts under tools/diag-scripts and .fret/diag/scripts."
    )]
    async fn fret_diag_scripts_list(
        &self,
        params: rmcp::handler::server::wrapper::Parameters<ScriptsListRequestV1>,
    ) -> Result<Json<ScriptsListResultV1>, String> {
        let repo_root = repo_root_from_manifest_dir()
            .or_else(|| std::env::current_dir().ok())
            .ok_or_else(|| "failed to resolve repo root".to_string())?;

        let mut scripts = Vec::<ScriptDescriptorV1>::new();
        scripts.extend(scan_scripts_dir(
            &repo_root,
            &repo_root.join("tools").join("diag-scripts"),
            "workspace",
        ));

        let include_user = params.0.include_user.unwrap_or(true);
        if include_user {
            scripts.extend(scan_scripts_dir(
                &repo_root,
                &repo_root.join(".fret").join("diag").join("scripts"),
                "user",
            ));
        }

        scripts.sort_by(|a, b| {
            (a.origin.as_str(), a.name.as_str(), a.rel_path.as_str()).cmp(&(
                b.origin.as_str(),
                b.name.as_str(),
                b.rel_path.as_str(),
            ))
        });

        Ok(Json(ScriptsListResultV1 {
            schema_version: 1,
            scripts,
        }))
    }

    #[tool(
        description = "Run a script by file name or relative path (tools/diag-scripts or .fret/diag/scripts)."
    )]
    async fn fret_diag_run_script_file(
        &self,
        params: rmcp::handler::server::wrapper::Parameters<RunScriptFileRequestV1>,
    ) -> Result<String, String> {
        let repo_root = repo_root_from_manifest_dir()
            .or_else(|| std::env::current_dir().ok())
            .ok_or_else(|| "failed to resolve repo root".to_string())?;

        let session_id = self.resolve_session_id(params.0.session_id.clone()).await?;
        let script_path = resolve_script_path(&repo_root, &params.0.script)?;
        let script_text = std::fs::read_to_string(&script_path).map_err(|e| e.to_string())?;
        let script_value: serde_json::Value =
            serde_json::from_str(&script_text).map_err(|e| e.to_string())?;

        self.run_script_value_and_wait(&session_id, script_value, params.0.timeout_ms)
            .await
    }

    #[tool(
        description = "Run a list of scripts (by file name/relative path, or via a simple '*' wildcard pattern) and return a structured summary."
    )]
    async fn fret_diag_run(
        &self,
        params: rmcp::handler::server::wrapper::Parameters<RunScriptsRequestV1>,
    ) -> Result<Json<RunScriptsResultV1>, String> {
        let repo_root = repo_root_from_manifest_dir()
            .or_else(|| std::env::current_dir().ok())
            .ok_or_else(|| "failed to resolve repo root".to_string())?;

        let session_id = self.resolve_session_id(params.0.session_id.clone()).await?;

        let stop_on_failure = params.0.stop_on_failure.unwrap_or(true);
        let timeout_ms_per_script = params.0.timeout_ms_per_script.unwrap_or(120_000);

        let mut scripts: Vec<String> = Vec::new();
        if let Some(list) = params.0.scripts.clone() {
            scripts.extend(list.into_iter().filter(|s| !s.trim().is_empty()));
        } else if let Some(glob) = params
            .0
            .glob
            .as_deref()
            .map(|s| s.trim())
            .filter(|s| !s.is_empty())
        {
            scripts.extend(resolve_scripts_by_glob(
                &repo_root,
                glob,
                params.0.include_user.unwrap_or(true),
            ));
        } else {
            return Err("missing scripts or glob (use fret_diag_scripts_list)".to_string());
        }

        if scripts.is_empty() {
            return Err("no scripts selected to run".to_string());
        }

        let started_unix_ms = unix_ms_now();
        let started = tokio::time::Instant::now();

        let mut entries: Vec<RunScriptsEntryV1> = Vec::new();
        let mut passed = 0u32;
        let mut failed = 0u32;

        for spec in scripts {
            let script_path = resolve_script_path(&repo_root, &spec)?;
            let rel_path = script_path
                .strip_prefix(&repo_root)
                .ok()
                .map(|p| p.to_string_lossy().to_string())
                .unwrap_or_else(|| script_path.to_string_lossy().to_string());

            let script_text = std::fs::read_to_string(&script_path).map_err(|e| e.to_string())?;
            let script_value: serde_json::Value =
                serde_json::from_str(&script_text).map_err(|e| e.to_string())?;

            let result = self
                .run_script_value_and_wait_parsed(&session_id, script_value, timeout_ms_per_script)
                .await;

            match result {
                Ok(parsed) => {
                    let stage = format!("{:?}", parsed.stage);
                    if matches!(parsed.stage, UiScriptStageV1::Passed) {
                        passed = passed.saturating_add(1);
                    } else if matches!(parsed.stage, UiScriptStageV1::Failed) {
                        failed = failed.saturating_add(1);
                    }
                    let ok = matches!(parsed.stage, UiScriptStageV1::Passed);
                    entries.push(RunScriptsEntryV1 {
                        script: rel_path,
                        ok,
                        stage,
                        run_id: parsed.run_id,
                        step_index: parsed.step_index,
                        reason: parsed.reason,
                        last_bundle_dir: parsed.last_bundle_dir,
                        updated_unix_ms: parsed.updated_unix_ms,
                    });
                    if stop_on_failure && !ok {
                        break;
                    }
                }
                Err(err) => {
                    failed = failed.saturating_add(1);
                    entries.push(RunScriptsEntryV1 {
                        script: rel_path,
                        ok: false,
                        stage: "Error".to_string(),
                        run_id: 0,
                        step_index: None,
                        reason: Some(err),
                        last_bundle_dir: None,
                        updated_unix_ms: unix_ms_now(),
                    });
                    if stop_on_failure {
                        break;
                    }
                }
            }
        }

        let ran = entries.len() as u32;
        let ok = failed == 0 && ran > 0;

        Ok(Json(RunScriptsResultV1 {
            schema_version: 1,
            ok,
            started_unix_ms,
            elapsed_ms: started.elapsed().as_millis().min(u128::from(u64::MAX)) as u64,
            ran,
            passed,
            failed,
            entries,
        }))
    }

    async fn run_script_value_and_wait(
        &self,
        session_id: &str,
        script: serde_json::Value,
        timeout_ms: u64,
    ) -> Result<String, String> {
        let parsed = self
            .run_script_value_and_wait_parsed(session_id, script, timeout_ms)
            .await?;
        Ok(serde_json::to_string_pretty(&parsed).unwrap_or_else(|_| "{}".to_string()))
    }

    async fn run_script_value_and_wait_parsed(
        &self,
        session_id: &str,
        script: serde_json::Value,
        timeout_ms: u64,
    ) -> Result<UiScriptResultV1, String> {
        // Avoid picking up stale script.result messages from a previous run.
        self.drain_inbox_type_for_session("script.result", session_id)
            .await;

        self.client_tx
            .send(ClientCommand::Send(DiagTransportMessageV1 {
                schema_version: 1,
                r#type: "script.run".to_string(),
                session_id: Some(session_id.to_string()),
                request_id: None,
                payload: serde_json::json!({ "script": script }),
            }))
            .map_err(|_| "client task is not running".to_string())?;

        let start = tokio::time::Instant::now();
        let mut expected_run_id: Option<u64> = None;
        loop {
            if start.elapsed() > Duration::from_millis(timeout_ms) {
                return Err("timeout waiting for script.result".to_string());
            }

            if let Some(msg) = self
                .pop_next_of_type_and_session("script.result", session_id)
                .await
            {
                if let Ok(parsed) = serde_json::from_value::<UiScriptResultV1>(msg.payload.clone())
                {
                    if expected_run_id.is_none() {
                        expected_run_id = Some(parsed.run_id);
                    }
                    if expected_run_id != Some(parsed.run_id) {
                        continue;
                    }
                    match parsed.stage {
                        UiScriptStageV1::Passed | UiScriptStageV1::Failed => {
                            return Ok(parsed);
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

    async fn drain_inbox_type_for_session(&self, ty: &str, session_id: &str) {
        let mut inbox = self.inbox.lock().await;
        inbox.retain(|m| !(m.r#type == ty && m.session_id.as_deref() == Some(session_id)));
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
            capabilities: ServerCapabilities::builder()
                .enable_tools()
                .enable_resources_with(ResourcesCapability {
                    subscribe: Some(true),
                    list_changed: Some(true),
                })
                .build(),
            ..Default::default()
        }
    }

    fn on_initialized(
        &self,
        context: rmcp::service::NotificationContext<rmcp::RoleServer>,
    ) -> impl std::future::Future<Output = ()> + Send + '_ {
        async move {
            *self.peer.lock().await = Some(context.peer.clone());
        }
    }

    fn subscribe(
        &self,
        request: SubscribeRequestParams,
        _context: rmcp::service::RequestContext<rmcp::RoleServer>,
    ) -> impl std::future::Future<Output = Result<(), McpError>> + Send + '_ {
        async move {
            self.subscribed_resources.lock().await.insert(request.uri);
            Ok(())
        }
    }

    fn unsubscribe(
        &self,
        request: UnsubscribeRequestParams,
        _context: rmcp::service::RequestContext<rmcp::RoleServer>,
    ) -> impl std::future::Future<Output = Result<(), McpError>> + Send + '_ {
        async move {
            self.subscribed_resources.lock().await.remove(&request.uri);
            Ok(())
        }
    }

    fn list_resources(
        &self,
        _request: Option<PaginatedRequestParams>,
        _context: rmcp::service::RequestContext<rmcp::RoleServer>,
    ) -> impl std::future::Future<Output = Result<ListResourcesResult, McpError>> + Send + '_ {
        async move {
            let sessions = self.sessions.lock().await.clone();
            let inbox = self.inbox.lock().await;

            let mut resources: Vec<Resource> = Vec::new();
            for s in sessions {
                let Some(_payload) = inbox.iter().rev().find(|m| {
                    m.r#type == "bundle.dumped"
                        && m.session_id.as_deref() == Some(s.session_id.as_str())
                }) else {
                    continue;
                };

                let sid = s.session_id;
                let base = format!("{RESOURCE_SCHEME}sessions/{sid}/");

                let mut bundle_json = RawResource::new(
                    format!("{base}{RESOURCE_KIND_BUNDLE_JSON}"),
                    format!("bundle.json [{sid}]"),
                );
                bundle_json.mime_type = Some("application/json".to_string());
                bundle_json.description = Some(
                    "Latest bundle.json derived from the most recent bundle.dumped event (call fret_diag_bundle_dump to refresh)."
                        .to_string(),
                );
                resources.push(bundle_json.no_annotation());

                let mut bundle_zip = RawResource::new(
                    format!("{base}{RESOURCE_KIND_BUNDLE_ZIP}"),
                    format!("bundle.zip [{sid}]"),
                );
                bundle_zip.mime_type = Some("application/zip".to_string());
                bundle_zip.description = Some(
                    "A zip containing bundle.json (same layout as diag pack). Generated on read from the latest bundle.dumped event."
                        .to_string(),
                );
                resources.push(bundle_zip.no_annotation());

                if let Some(repro_path) =
                    repro_summary_path_from_latest_bundle_dumped_payload(&inbox, &sid)
                {
                    if repro_path.is_file() {
                        let mut repro = RawResource::new(
                            format!("{base}{RESOURCE_KIND_REPRO_SUMMARY_JSON}"),
                            format!("repro.summary.json [{sid}]"),
                        );
                        repro.mime_type = Some("application/json".to_string());
                        repro.description = Some(
                            "Repro summary generated by fretboard diag repro (if present in the artifacts root)."
                                .to_string(),
                        );
                        resources.push(repro.no_annotation());
                    }
                }
            }

            Ok(ListResourcesResult::with_all_items(resources))
        }
    }

    fn list_resource_templates(
        &self,
        _request: Option<PaginatedRequestParams>,
        _context: rmcp::service::RequestContext<rmcp::RoleServer>,
    ) -> impl std::future::Future<Output = Result<ListResourceTemplatesResult, McpError>> + Send + '_
    {
        async move {
            let mk = |uri_template: &str, name: &str, mime: &str, description: &str| {
                let t = RawResourceTemplate {
                    uri_template: uri_template.to_string(),
                    name: name.to_string(),
                    title: None,
                    description: Some(description.to_string()),
                    mime_type: Some(mime.to_string()),
                    icons: None,
                };
                t.no_annotation()
            };

            Ok(ListResourceTemplatesResult::with_all_items(vec![
                mk(
                    "fret-diag://sessions/{session_id}/bundle.json",
                    "bundle.json",
                    "application/json",
                    "Latest bundle.json for a session (requires an existing bundle.dumped event).",
                ),
                mk(
                    "fret-diag://sessions/{session_id}/bundle.zip",
                    "bundle.zip",
                    "application/zip",
                    "Zip containing bundle.json for a session (generated on read).",
                ),
                mk(
                    "fret-diag://sessions/{session_id}/repro.summary.json",
                    "repro.summary.json",
                    "application/json",
                    "Repro summary for a session (only if present on disk).",
                ),
            ]))
        }
    }

    fn read_resource(
        &self,
        request: ReadResourceRequestParams,
        _context: rmcp::service::RequestContext<rmcp::RoleServer>,
    ) -> impl std::future::Future<Output = Result<ReadResourceResult, McpError>> + Send + '_ {
        async move {
            let uri = request.uri.trim();
            let parsed = parse_resource_uri(uri)
                .ok_or_else(|| McpError::resource_not_found("unknown resource uri", None))?;

            let session_id = self
                .resolve_session_id(parsed.session_id.clone())
                .await
                .map_err(|e| McpError::resource_not_found(e, None))?;

            let dumped_payload = {
                let inbox = self.inbox.lock().await;
                inbox
                    .iter()
                    .rev()
                    .find(|m| {
                        m.r#type == "bundle.dumped"
                            && m.session_id.as_deref() == Some(session_id.as_str())
                    })
                    .map(|m| m.payload.clone())
            };

            let Some(dumped_payload) = dumped_payload else {
                return Err(McpError::resource_not_found(
                    "no bundle.dumped available (call fret_diag_bundle_dump first)",
                    None,
                ));
            };

            let repo_root = repo_root_from_manifest_dir()
                .or_else(|| std::env::current_dir().ok())
                .ok_or_else(|| McpError::internal_error("failed to resolve repo root", None))?;

            match parsed.kind.as_str() {
                RESOURCE_KIND_BUNDLE_JSON => {
                    let bundle_json =
                        bundle_json_from_bundle_dumped_payload(&repo_root, &dumped_payload)
                            .map_err(|e| McpError::resource_not_found(e, None))?;
                    Ok(ReadResourceResult {
                        contents: vec![ResourceContents::TextResourceContents {
                            uri: uri.to_string(),
                            mime_type: Some("application/json".to_string()),
                            text: bundle_json,
                            meta: None,
                        }],
                    })
                }
                RESOURCE_KIND_BUNDLE_ZIP => {
                    let bundle_json =
                        bundle_json_from_bundle_dumped_payload(&repo_root, &dumped_payload)
                            .map_err(|e| McpError::resource_not_found(e, None))?;
                    let bundle_name = bundle_name_from_bundle_dumped_payload(&dumped_payload);
                    let zip_bytes =
                        artifacts::pack_bundle_json_to_zip_bytes_named(&bundle_name, &bundle_json)
                            .map_err(|e| McpError::internal_error(e, None))?;
                    let zip_base64 = base64::engine::general_purpose::STANDARD.encode(zip_bytes);

                    Ok(ReadResourceResult {
                        contents: vec![ResourceContents::BlobResourceContents {
                            uri: uri.to_string(),
                            mime_type: Some("application/zip".to_string()),
                            blob: zip_base64,
                            meta: None,
                        }],
                    })
                }
                RESOURCE_KIND_REPRO_SUMMARY_JSON => {
                    let path =
                        repro_summary_path_from_bundle_dumped_payload(&repo_root, &dumped_payload)
                            .ok_or_else(|| {
                                McpError::resource_not_found(
                                    "bundle.dumped missing out_dir/dir",
                                    None,
                                )
                            })?;
                    if !path.is_file() {
                        return Err(McpError::resource_not_found(
                            "repro.summary.json not found for this session",
                            None,
                        ));
                    }
                    let text = std::fs::read_to_string(&path)
                        .map_err(|e| McpError::internal_error(e.to_string(), None))?;
                    Ok(ReadResourceResult {
                        contents: vec![ResourceContents::TextResourceContents {
                            uri: uri.to_string(),
                            mime_type: Some("application/json".to_string()),
                            text,
                            meta: None,
                        }],
                    })
                }
                _ => Err(McpError::resource_not_found("unknown resource kind", None)),
            }
        }
    }
}

#[derive(Serialize, Deserialize, JsonSchema)]
struct ConnectRequestV1 {
    /// Transport kind: "ws" | "fs"
    transport: String,
    /// Optional WS URL override (defaults to the locally hosted WS hub).
    #[serde(default)]
    ws_url: Option<String>,
    /// Optional capability token override (defaults to the locally hosted WS hub token).
    #[serde(default)]
    token: Option<String>,
    /// Filesystem out_dir used for file-trigger transport (defaults to `FRET_DIAG_DIR` or `target/fret-diag`).
    #[serde(default)]
    fs_out_dir: Option<String>,
}

#[derive(Serialize, Deserialize, JsonSchema)]
struct ConnectResultV1 {
    schema_version: u32,
    transport: String,
    #[serde(default)]
    ws_url: Option<String>,
    #[serde(default)]
    token: Option<String>,
    #[serde(default)]
    fs_out_dir: Option<String>,
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
struct PackLastBundleRequestV1 {
    #[serde(default)]
    session_id: Option<String>,
    #[serde(default)]
    label: Option<String>,
    /// Optional override for where to materialize web-runner bundles (defaults to `.fret/diag/exports`).
    #[serde(default)]
    export_out_dir: Option<String>,
    /// Optional override for output zip path.
    #[serde(default)]
    pack_out: Option<String>,
    /// When true (default), includes triage/screenshot/root artifacts if present.
    #[serde(default)]
    include_all: Option<bool>,
    timeout_ms: u64,
}

#[derive(Serialize, Deserialize, JsonSchema)]
struct PackLastBundleResultV1 {
    schema_version: u32,
    out_dir: String,
    bundle_dir: String,
    pack_path: String,
    bundle_dumped_json: String,
}

#[derive(Serialize, Deserialize, JsonSchema)]
struct BundleDumpLatestRequestV1 {
    #[serde(default)]
    session_id: Option<String>,
}

#[derive(Serialize, Deserialize, JsonSchema)]
struct BundleDumpLatestResultV1 {
    schema_version: u32,
    found: bool,
    #[serde(default)]
    payload_json: Option<String>,
}

#[derive(Serialize, Deserialize, JsonSchema)]
struct BundleJsonLatestRequestV1 {
    #[serde(default)]
    session_id: Option<String>,
}

#[derive(Serialize, Deserialize, JsonSchema)]
struct BundleJsonLatestResultV1 {
    schema_version: u32,
    found: bool,
    #[serde(default)]
    bundle_json: Option<String>,
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
struct PackLastBundleZipBytesRequestV1 {
    #[serde(default)]
    session_id: Option<String>,
    #[serde(default)]
    label: Option<String>,
    timeout_ms: u64,
}

#[derive(Serialize, Deserialize, JsonSchema)]
struct PackLastBundleZipBytesResultV1 {
    schema_version: u32,
    zip_base64: String,
    bundle_dumped_json: String,
}

#[derive(Serialize, Deserialize, JsonSchema)]
struct CompareBundlesRequestV1 {
    /// Bundle A path (bundle.json file or containing directory). Relative paths are resolved against the repo root.
    a: String,
    /// Bundle B path (bundle.json file or containing directory). Relative paths are resolved against the repo root.
    b: String,
    #[serde(default)]
    warmup_frames: Option<u64>,
    #[serde(default)]
    eps_px: Option<f32>,
    #[serde(default)]
    ignore_bounds: Option<bool>,
    #[serde(default)]
    ignore_scene_fingerprint: Option<bool>,
}

#[derive(Serialize, Deserialize, JsonSchema)]
struct CompareBundlesResultV1 {
    schema_version: u32,
    ok: bool,
    report_json: String,
}

#[derive(Serialize, Deserialize, JsonSchema)]
struct ScriptsListRequestV1 {
    /// When true (default), includes `.fret/diag/scripts` in addition to `tools/diag-scripts`.
    #[serde(default)]
    include_user: Option<bool>,
}

#[derive(Serialize, Deserialize, JsonSchema)]
struct ScriptDescriptorV1 {
    origin: String,
    name: String,
    /// Repo-relative path (best-effort).
    rel_path: String,
}

#[derive(Serialize, Deserialize, JsonSchema)]
struct ScriptsListResultV1 {
    schema_version: u32,
    scripts: Vec<ScriptDescriptorV1>,
}

#[derive(Serialize, Deserialize, JsonSchema)]
struct RunScriptFileRequestV1 {
    #[serde(default)]
    session_id: Option<String>,
    /// File name (e.g. `todo-baseline.json`) or repo-relative path under tools/diag-scripts or .fret/diag/scripts.
    script: String,
    timeout_ms: u64,
}

#[derive(Serialize, Deserialize, JsonSchema)]
struct RunScriptsRequestV1 {
    #[serde(default)]
    session_id: Option<String>,
    /// Explicit list of scripts (file names or repo-relative paths). Mutually exclusive with `glob`.
    #[serde(default)]
    scripts: Option<Vec<String>>,
    /// Simple wildcard pattern using `*` to match file names or repo-relative paths (e.g. `ui-gallery-*.json`).
    #[serde(default)]
    glob: Option<String>,
    /// When true (default), includes `.fret/diag/scripts` in addition to `tools/diag-scripts` for `glob` resolution.
    #[serde(default)]
    include_user: Option<bool>,
    /// When true (default), stops after the first failed script.
    #[serde(default)]
    stop_on_failure: Option<bool>,
    /// Timeout per script (default 120_000ms).
    #[serde(default)]
    timeout_ms_per_script: Option<u64>,
}

#[derive(Serialize, Deserialize, JsonSchema)]
struct RunScriptsEntryV1 {
    script: String,
    ok: bool,
    stage: String,
    run_id: u64,
    #[serde(default)]
    step_index: Option<u32>,
    #[serde(default)]
    reason: Option<String>,
    #[serde(default)]
    last_bundle_dir: Option<String>,
    updated_unix_ms: u64,
}

#[derive(Serialize, Deserialize, JsonSchema)]
struct RunScriptsResultV1 {
    schema_version: u32,
    ok: bool,
    started_unix_ms: u64,
    elapsed_ms: u64,
    ran: u32,
    passed: u32,
    failed: u32,
    entries: Vec<RunScriptsEntryV1>,
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
    let client = ToolingDiagClient::connect_ws(WsDiagTransportConfig::native(cfg))
        .map_err(anyhow::Error::msg)?;

    let client_kind = Arc::new(Mutex::new(DiagTransportKind::WebSocket));
    let inbox = Arc::new(Mutex::new(VecDeque::new()));
    let sessions = Arc::new(Mutex::new(Vec::<DevtoolsSessionDescriptorV1>::new()));
    let selected_session_id = Arc::new(Mutex::new(None::<String>));
    let peer = Arc::new(Mutex::new(None::<rmcp::Peer<rmcp::RoleServer>>));
    let subscribed_resources = Arc::new(Mutex::new(HashSet::<String>::new()));

    let (client_tx, client_rx) = mpsc::unbounded_channel::<ClientCommand>();
    tokio::spawn(run_client_task(
        client,
        client_rx,
        inbox.clone(),
        sessions.clone(),
        selected_session_id.clone(),
        WsState {
            ws_url: ws_url.clone(),
            token: token.clone(),
        },
        peer.clone(),
        subscribed_resources.clone(),
    ));

    let service = FretDevtoolsMcp::new(
        WsState { ws_url, token },
        client_tx,
        client_kind,
        inbox,
        sessions,
        selected_session_id,
        peer,
        subscribed_resources,
    )
    .serve(stdio())
    .await?;
    service.waiting().await?;
    Ok(())
}

async fn run_client_task(
    mut client: ToolingDiagClient,
    mut rx: mpsc::UnboundedReceiver<ClientCommand>,
    inbox: Arc<Mutex<VecDeque<DiagTransportMessageV1>>>,
    sessions: Arc<Mutex<Vec<DevtoolsSessionDescriptorV1>>>,
    selected_session_id: Arc<Mutex<Option<String>>>,
    ws_defaults: WsState,
    peer: Arc<Mutex<Option<rmcp::Peer<rmcp::RoleServer>>>>,
    subscribed_resources: Arc<Mutex<HashSet<String>>>,
) {
    loop {
        while let Ok(cmd) = rx.try_recv() {
            match cmd {
                ClientCommand::Send(msg) => {
                    client.send(msg);
                }
                ClientCommand::SetDefaultSessionId(session_id) => {
                    client.set_default_session_id(session_id);
                }
                ClientCommand::Connect(cfg, ack) => {
                    let result = connect_client(&ws_defaults, cfg).and_then(|new_client| {
                        client = new_client;
                        Ok(())
                    });
                    let _ = ack.send(result);
                }
            }
        }

        let mut drained = false;
        while let Some(msg) = client.try_recv() {
            drained = true;

            let mut notify_resources_list_changed: bool = false;
            let mut resource_updated_uris: Vec<String> = Vec::new();

            match msg.r#type.as_str() {
                "session.list" => {
                    if let Ok(parsed) =
                        serde_json::from_value::<DevtoolsSessionListV1>(msg.payload.clone())
                    {
                        *sessions.lock().await = parsed.sessions;
                        notify_resources_list_changed = true;
                    }
                }
                "session.added" => {
                    if let Ok(parsed) =
                        serde_json::from_value::<DevtoolsSessionAddedV1>(msg.payload.clone())
                    {
                        let mut s = sessions.lock().await;
                        if let Some(pos) = s
                            .iter()
                            .position(|x| x.session_id == parsed.session.session_id)
                        {
                            s[pos] = parsed.session;
                        } else {
                            s.push(parsed.session);
                        }
                        notify_resources_list_changed = true;
                    }
                }
                "session.removed" => {
                    if let Ok(parsed) =
                        serde_json::from_value::<DevtoolsSessionRemovedV1>(msg.payload.clone())
                    {
                        let mut s = sessions.lock().await;
                        s.retain(|x| x.session_id != parsed.session_id);
                        notify_resources_list_changed = true;
                    }
                }
                "bundle.dumped" => {
                    notify_resources_list_changed = true;
                    if let Some(sid) = msg.session_id.as_deref() {
                        let base = format!("{RESOURCE_SCHEME}sessions/{sid}/");
                        resource_updated_uris.push(format!("{base}{RESOURCE_KIND_BUNDLE_JSON}"));
                        resource_updated_uris.push(format!("{base}{RESOURCE_KIND_BUNDLE_ZIP}"));
                        resource_updated_uris
                            .push(format!("{base}{RESOURCE_KIND_REPRO_SUMMARY_JSON}"));

                        let selected = selected_session_id.lock().await.clone();
                        if selected.as_deref() == Some(sid) {
                            let selected_base = format!("{RESOURCE_SCHEME}selected/");
                            resource_updated_uris
                                .push(format!("{selected_base}{RESOURCE_KIND_BUNDLE_JSON}"));
                            resource_updated_uris
                                .push(format!("{selected_base}{RESOURCE_KIND_BUNDLE_ZIP}"));
                            resource_updated_uris
                                .push(format!("{selected_base}{RESOURCE_KIND_REPRO_SUMMARY_JSON}"));
                        }
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

            {
                let mut inbox = inbox.lock().await;
                inbox.push_back(msg);
                if inbox.len() > 2000 {
                    let drain = inbox.len().saturating_sub(2000);
                    inbox.drain(0..drain);
                }
            }

            if notify_resources_list_changed || !resource_updated_uris.is_empty() {
                let peer = peer.lock().await.clone();
                let subscribed = subscribed_resources.lock().await.clone();

                if let Some(peer) = peer {
                    if notify_resources_list_changed {
                        let n = ResourceListChangedNotification {
                            method: Default::default(),
                            extensions: Extensions::default(),
                        };
                        let _ = peer
                            .send_notification(ServerNotification::ResourceListChangedNotification(
                                n,
                            ))
                            .await;
                    }

                    for uri in resource_updated_uris {
                        if !subscribed.contains(&uri) {
                            continue;
                        }
                        let n =
                            ResourceUpdatedNotification::new(ResourceUpdatedNotificationParam {
                                uri,
                            });
                        let _ = peer
                            .send_notification(ServerNotification::ResourceUpdatedNotification(n))
                            .await;
                    }
                }
            }
        }

        if !drained {
            tokio::time::sleep(Duration::from_millis(10)).await;
        }
    }
}

fn connect_client(ws_defaults: &WsState, cfg: ConnectConfig) -> Result<ToolingDiagClient, String> {
    match cfg.kind {
        DiagTransportKind::WebSocket => {
            let ws_url = cfg
                .ws_url
                .as_deref()
                .map(|s| s.trim().to_string())
                .filter(|s| !s.is_empty())
                .unwrap_or_else(|| ws_defaults.ws_url.to_string());
            let token = cfg
                .token
                .as_deref()
                .map(|s| s.trim().to_string())
                .filter(|s| !s.is_empty())
                .unwrap_or_else(|| ws_defaults.token.to_string());

            let mut client_cfg = DevtoolsWsClientConfig::with_defaults(ws_url, token);
            client_cfg.client_kind = ClientKindV1::Tooling;
            client_cfg.capabilities = vec![
                "inspect".to_string(),
                "pick".to_string(),
                "scripts".to_string(),
                "bundles".to_string(),
            ];
            ToolingDiagClient::connect_ws(WsDiagTransportConfig::native(client_cfg))
        }
        DiagTransportKind::FileSystem => {
            let out_dir = cfg
                .fs_out_dir
                .as_deref()
                .map(|s| s.trim().to_string())
                .filter(|s| !s.is_empty())
                .or_else(|| std::env::var("FRET_DIAG_DIR").ok())
                .unwrap_or_else(|| "target/fret-diag".to_string());
            let fs_cfg = FsDiagTransportConfig::from_out_dir(PathBuf::from(out_dir));
            ToolingDiagClient::connect_fs(fs_cfg)
        }
    }
}

fn env_u16(key: &str) -> Option<u16> {
    std::env::var(key).ok().and_then(|v| v.parse().ok())
}

fn repo_root_from_manifest_dir() -> Option<PathBuf> {
    let manifest_dir = Path::new(env!("CARGO_MANIFEST_DIR"));
    let apps_dir = manifest_dir.parent()?;
    apps_dir.parent().map(|p| p.to_path_buf())
}

fn resolve_repo_path(repo_root: &Path, raw: &str) -> PathBuf {
    let raw = raw.trim();
    if raw.is_empty() {
        return repo_root.to_path_buf();
    }
    let p = PathBuf::from(raw);
    if p.is_absolute() {
        p
    } else {
        repo_root.join(p)
    }
}

fn resolve_bundle_json_path(src: &Path) -> PathBuf {
    if src.is_file() {
        return src.to_path_buf();
    }
    src.join("bundle.json")
}

fn scan_scripts_dir(repo_root: &Path, dir: &Path, origin: &str) -> Vec<ScriptDescriptorV1> {
    let mut out = Vec::new();
    let Ok(entries) = std::fs::read_dir(dir) else {
        return out;
    };
    for entry in entries.flatten() {
        let path = entry.path();
        if path.extension().and_then(|s| s.to_str()) != Some("json") {
            continue;
        }
        let file_name = path
            .file_name()
            .and_then(|s| s.to_str())
            .unwrap_or("")
            .to_string();
        if file_name.trim().is_empty() {
            continue;
        }
        let rel_path = path
            .strip_prefix(repo_root)
            .ok()
            .map(|p| p.to_string_lossy().to_string())
            .unwrap_or_else(|| path.to_string_lossy().to_string());
        out.push(ScriptDescriptorV1 {
            origin: origin.to_string(),
            name: file_name,
            rel_path,
        });
    }
    out
}

fn resolve_scripts_by_glob(repo_root: &Path, glob: &str, include_user: bool) -> Vec<String> {
    let mut out: Vec<String> = Vec::new();
    let glob = glob.trim();
    if glob.is_empty() {
        return out;
    }

    let mut all = Vec::<ScriptDescriptorV1>::new();
    all.extend(scan_scripts_dir(
        repo_root,
        &repo_root.join("tools").join("diag-scripts"),
        "workspace",
    ));
    if include_user {
        all.extend(scan_scripts_dir(
            repo_root,
            &repo_root.join(".fret").join("diag").join("scripts"),
            "user",
        ));
    }

    for s in all {
        if wildcard_match(glob, &s.name) || wildcard_match(glob, &s.rel_path) {
            out.push(s.rel_path);
        }
    }
    out
}

fn wildcard_match(pattern: &str, text: &str) -> bool {
    let pattern = pattern.trim();
    if pattern == "*" {
        return true;
    }
    let parts: Vec<&str> = pattern.split('*').collect();
    if parts.len() == 1 {
        return pattern == text;
    }

    let mut pos = 0usize;
    let mut first = true;
    for part in parts.iter().copied() {
        if part.is_empty() {
            continue;
        }
        if let Some(idx) = text[pos..].find(part) {
            if first && !pattern.starts_with('*') && idx != 0 {
                return false;
            }
            pos += idx + part.len();
        } else {
            return false;
        }
        first = false;
    }
    if !pattern.ends_with('*') {
        if let Some(last) = parts.iter().rev().find(|p| !p.is_empty()) {
            return text.ends_with(last);
        }
    }
    true
}

fn resolve_script_path(repo_root: &Path, script: &str) -> Result<PathBuf, String> {
    let script = script.trim();
    if script.is_empty() {
        return Err("missing script".to_string());
    }

    let candidate = PathBuf::from(script);
    if candidate.components().count() == 1 {
        let tools = repo_root
            .join("tools")
            .join("diag-scripts")
            .join(candidate.clone());
        if tools.is_file() {
            return Ok(tools);
        }
        let user = repo_root
            .join(".fret")
            .join("diag")
            .join("scripts")
            .join(candidate);
        if user.is_file() {
            return Ok(user);
        }
        return Err("script not found (try fret_diag_scripts_list)".to_string());
    }

    let full = resolve_repo_path(repo_root, script);
    let full_canon = full.canonicalize().map_err(|e| e.to_string())?;
    let repo_canon = repo_root.canonicalize().map_err(|e| e.to_string())?;
    if !full_canon.starts_with(&repo_canon) {
        return Err("script path must be under repo root".to_string());
    }

    let allowed_a = repo_canon.join("tools").join("diag-scripts");
    let allowed_b = repo_canon.join(".fret").join("diag").join("scripts");
    if !full_canon.starts_with(&allowed_a) && !full_canon.starts_with(&allowed_b) {
        return Err(
            "script path must be under tools/diag-scripts or .fret/diag/scripts".to_string(),
        );
    }
    if full_canon.extension().and_then(|s| s.to_str()) != Some("json") {
        return Err("script file must be a .json".to_string());
    }
    if !full_canon.is_file() {
        return Err("script path is not a file".to_string());
    }
    Ok(full_canon)
}

fn bundle_json_from_bundle_dumped_payload(
    repo_root: &Path,
    dumped_payload: &serde_json::Value,
) -> Result<String, String> {
    if let Some(bundle) = dumped_payload.get("bundle") {
        return serde_json::to_string_pretty(bundle).map_err(|e| e.to_string());
    }

    let out_dir = dumped_payload
        .get("out_dir")
        .and_then(|v| v.as_str())
        .ok_or_else(|| "bundle.dumped missing out_dir (and no bundle payload)".to_string())?;
    let dir = dumped_payload
        .get("dir")
        .and_then(|v| v.as_str())
        .ok_or_else(|| "bundle.dumped missing dir (and no bundle payload)".to_string())?;

    let out_dir_path = Path::new(out_dir);
    let out_dir_abs = if out_dir_path.is_absolute() {
        out_dir_path.to_path_buf()
    } else {
        repo_root.join(out_dir_path)
    };

    let bundle_dir = if Path::new(dir).is_absolute() {
        PathBuf::from(dir)
    } else {
        out_dir_abs.join(dir)
    };

    std::fs::read_to_string(bundle_dir.join("bundle.json")).map_err(|e| e.to_string())
}

fn default_pack_out_path(repo_root: &Path, bundle_dir_arg: &str) -> PathBuf {
    let pack_dir = repo_root.join(".fret").join("diag").join("packs");
    let _ = std::fs::create_dir_all(&pack_dir);

    let bundle_name = Path::new(bundle_dir_arg)
        .file_name()
        .and_then(|s| s.to_str())
        .filter(|s| !s.trim().is_empty())
        .unwrap_or("bundle");
    pack_dir.join(format!("{bundle_name}-{}.zip", unix_ms_now()))
}

fn materialize_or_resolve_bundle_dir(
    repo_root: &Path,
    dumped_payload: &serde_json::Value,
    export_out_dir_override: Option<&str>,
) -> Result<(String, String), String> {
    let exported_unix_ms = dumped_payload
        .get("exported_unix_ms")
        .and_then(|v| v.as_u64())
        .unwrap_or_else(unix_ms_now);

    if let Some(bundle) = dumped_payload.get("bundle") {
        let export_root = match export_out_dir_override
            .map(|s| s.trim().to_string())
            .filter(|s| !s.is_empty())
        {
            Some(p) if Path::new(&p).is_absolute() => PathBuf::from(p),
            Some(p) => repo_root.join(p),
            None => repo_root.join(".fret").join("diag").join("exports"),
        };
        let text = serde_json::to_string_pretty(bundle).unwrap_or_else(|_| "{}".to_string());
        let export_dir = artifacts::materialize_bundle_json(&export_root, exported_unix_ms, &text)?;
        return Ok((
            export_root.to_string_lossy().to_string(),
            export_dir.to_string_lossy().to_string(),
        ));
    }

    let out_dir = dumped_payload
        .get("out_dir")
        .and_then(|v| v.as_str())
        .ok_or_else(|| "bundle.dumped missing out_dir (and no bundle payload)".to_string())?;
    let dir = dumped_payload
        .get("dir")
        .and_then(|v| v.as_str())
        .ok_or_else(|| "bundle.dumped missing dir (and no bundle payload)".to_string())?;

    let bundle_dir = if Path::new(dir).is_absolute() {
        dir.to_string()
    } else {
        let joined = Path::new(out_dir).join(dir);
        joined.to_string_lossy().to_string()
    };

    Ok((out_dir.to_string(), bundle_dir))
}

fn unix_ms_now() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .ok()
        .map(|d| d.as_millis().min(u128::from(u64::MAX)) as u64)
        .unwrap_or(0)
}

#[derive(Debug, Clone)]
struct ParsedResourceUri {
    session_id: Option<String>,
    kind: String,
}

fn parse_resource_uri(uri: &str) -> Option<ParsedResourceUri> {
    let uri = uri.trim();
    if !uri.starts_with(RESOURCE_SCHEME) {
        return None;
    }
    let rest = uri.strip_prefix(RESOURCE_SCHEME)?;
    let mut parts = rest.split('/').filter(|p| !p.trim().is_empty());
    let head = parts.next()?;
    match head {
        "sessions" => {
            let session_id = parts.next()?.to_string();
            let kind = parts.next()?.to_string();
            Some(ParsedResourceUri {
                session_id: Some(session_id),
                kind,
            })
        }
        "selected" => {
            let kind = parts.next()?.to_string();
            Some(ParsedResourceUri {
                session_id: None,
                kind,
            })
        }
        _ => None,
    }
}

fn bundle_name_from_bundle_dumped_payload(dumped_payload: &serde_json::Value) -> String {
    let dir = dumped_payload
        .get("dir")
        .and_then(|v| v.as_str())
        .unwrap_or("");
    let name = Path::new(dir)
        .file_name()
        .and_then(|s| s.to_str())
        .filter(|s| !s.trim().is_empty())
        .unwrap_or("bundle");
    zip_safe_component(name)
}

fn zip_safe_component(s: &str) -> String {
    let s = s.trim();
    if s.is_empty() {
        return "bundle".to_string();
    }
    let mut out = String::with_capacity(s.len());
    for ch in s.chars() {
        let ok = ch.is_ascii_alphanumeric() || ch == '-' || ch == '_' || ch == '.';
        out.push(if ok { ch } else { '_' });
    }
    out
}

fn repro_summary_path_from_latest_bundle_dumped_payload(
    inbox: &VecDeque<DiagTransportMessageV1>,
    session_id: &str,
) -> Option<PathBuf> {
    let payload = inbox
        .iter()
        .rev()
        .find(|m| m.r#type == "bundle.dumped" && m.session_id.as_deref() == Some(session_id))
        .map(|m| m.payload.clone())?;

    let repo_root = repo_root_from_manifest_dir().or_else(|| std::env::current_dir().ok())?;
    repro_summary_path_from_bundle_dumped_payload(&repo_root, &payload)
}

fn repro_summary_path_from_bundle_dumped_payload(
    repo_root: &Path,
    dumped_payload: &serde_json::Value,
) -> Option<PathBuf> {
    let out_dir = dumped_payload.get("out_dir").and_then(|v| v.as_str())?;
    let dir = dumped_payload.get("dir").and_then(|v| v.as_str())?;

    let out_dir_path = Path::new(out_dir);
    let out_dir_abs = if out_dir_path.is_absolute() {
        out_dir_path.to_path_buf()
    } else {
        repo_root.join(out_dir_path)
    };

    let bundle_dir = if Path::new(dir).is_absolute() {
        PathBuf::from(dir)
    } else {
        out_dir_abs.join(dir)
    };

    let artifacts_root = if bundle_dir.starts_with(&out_dir_abs) {
        out_dir_abs
    } else {
        bundle_dir.parent().unwrap_or(repo_root).to_path_buf()
    };

    Some(artifacts_root.join("repro.summary.json"))
}
