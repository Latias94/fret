use std::collections::{HashSet, VecDeque};
use std::net::{IpAddr, Ipv4Addr, SocketAddr};
use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::Duration;

use base64::Engine;
use fret_diag::artifacts;
use fret_diag::regression_summary::{
    DIAG_REGRESSION_INDEX_FILENAME_V1, DIAG_REGRESSION_SUMMARY_FILENAME_V1,
    RegressionBundleFollowupCommandV1, RegressionSummaryV1,
    regression_bundle_followup_command_lines, regression_bundle_followup_commands,
    regression_summary_drilldown,
};
use fret_diag::transport::{
    ClientKindV1, DevtoolsWsClientConfig, DiagTransportKind, FsDiagTransportConfig,
    ToolingDiagClient, WsDiagTransportConfig,
};
use fret_diag::{
    DashboardCountEntry, DashboardFailingSummaryEntry, DashboardReasonCodeEntry,
    dashboard_human_lines_from_projection, project_dashboard_summary,
};
use fret_diag_protocol::{
    DevtoolsSessionAddedV1, DevtoolsSessionDescriptorV1, DevtoolsSessionListV1,
    DevtoolsSessionRemovedV1, DiagTransportMessageV1, UiScriptResultV1, UiScriptStageV1,
    UiSelectorV1,
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
const RESOURCE_URI_FIRST_OPEN_MD: &str = "fret-diag://first-open.md";
const RESOURCE_URI_RECENT_EVIDENCE_JSON: &str = "fret-diag://recent-evidence.json";
const RESOURCE_KIND_FIRST_OPEN_MD: &str = "first-open.md";
const RESOURCE_KIND_RECENT_EVIDENCE_JSON: &str = "recent-evidence.json";
const RESOURCE_KIND_BUNDLE_JSON: &str = "bundle.json";
const RESOURCE_KIND_BUNDLE_ZIP: &str = "bundle.zip";
const RESOURCE_KIND_REPRO_SUMMARY_JSON: &str = "repro.summary.json";
const RESOURCE_KIND_REGRESSION_SUMMARY_JSON: &str = DIAG_REGRESSION_SUMMARY_FILENAME_V1;
const RESOURCE_KIND_REGRESSION_INDEX_JSON: &str = DIAG_REGRESSION_INDEX_FILENAME_V1;
const DEVTOOLS_FIRST_OPEN_DOC: &str = "docs/diagnostics-first-open.md";
const DEVTOOLS_GUI_BRANCH_DOC: &str =
    "docs/workstreams/diag-fearless-refactor-v2/DEVTOOLS_GUI_DOGFOOD_WORKFLOW.md";
const DEVTOOLS_MCP_DOC: &str =
    "docs/workstreams/diag-devtools-gui-v1/diag-devtools-gui-v1-ai-mcp.md";
const DEVTOOLS_REPO_PREFLIGHT_COMMAND: &str = "cargo run -p fretboard-dev -- diag doctor campaigns";
const DEVTOOLS_REPO_PREFLIGHT_JSON_COMMAND: &str =
    "cargo run -p fretboard-dev -- diag doctor campaigns --json";
const DEVTOOLS_TOOL_APP_INDEX_COMMAND: &str = "cargo run -p fretboard-dev -- list tool-apps";
const DEVTOOLS_TOOL_APP_INDEX_JSON_COMMAND: &str =
    "cargo run -p fretboard-dev -- list tool-apps --json";
const IMUI_PRODUCT_WORKFLOW_ID: &str = "imui-product-chain";
const IMUI_PRODUCT_WORKFLOW_DOC: &str =
    "docs/workstreams/imui-editor-grade-product-closure-v1/EVIDENCE_AND_GATES.md";
const IMUI_PRODUCT_WORKFLOW_COMMAND: &str = "python tools/diag_gate_imui_product_chain.py";
const IMUI_PRODUCT_WORKFLOW_FOCUSED_COMMAND: &str =
    "python tools/diag_gate_imui_product_chain.py --only discovery";
const IMUI_PRODUCT_WORKFLOW_LAUNCHED_COMMAND: &str = "python tools/diag_gate_imui_product_chain.py --reuse-built --launched --only perf-docking --release";
const IMUI_PRODUCT_WORKFLOW_SUITE: &str =
    "tools/diag-scripts/suites/perf-docking-arbitration-steady/suite.json";
const IMUI_PRODUCT_WORKFLOW_ARTIFACTS: &[&str] = &[
    "perf-docking/regression.summary.json",
    "perf-docking/check.perf_thresholds.json",
    "perf-docking/*/trace.chrome.json",
];
const DEMO_METRICS_DEBUG_ROUTE_ID: &str = fret_first_open::demo_metrics_debug::ROUTE_ID;
const DEMO_EDITOR_WORKBENCH_COMMAND: &str =
    fret_first_open::demo_metrics_debug::DEMO_EDITOR_WORKBENCH_COMMAND;
const DEMO_EDITOR_PROOF_COMMAND: &str =
    fret_first_open::demo_metrics_debug::DEMO_EDITOR_PROOF_COMMAND;
const DEMO_EDITOR_NOTES_COMMAND: &str =
    fret_first_open::demo_metrics_debug::DEMO_EDITOR_NOTES_COMMAND;
const DEMO_DEVICE_SHELL_COMMAND: &str =
    fret_first_open::demo_metrics_debug::DEMO_DEVICE_SHELL_COMMAND;
const METRICS_STATS_COMMAND: &str = fret_first_open::demo_metrics_debug::METRICS_STATS_COMMAND;
const METRICS_LAYOUT_PERF_COMMAND: &str =
    "cargo run -p fretboard-dev -- diag layout-perf-summary <bundle-or-dir> --json";
const METRICS_MEMORY_COMMAND: &str =
    "cargo run -p fretboard-dev -- diag memory-summary <bundle-or-dir> --json";
const DEBUG_TRIAGE_COMMAND: &str =
    "cargo run -p fretboard-dev -- diag triage <bundle-or-dir> --json";
const DEBUG_HOTSPOTS_COMMAND: &str =
    "cargo run -p fretboard-dev -- diag hotspots <bundle-or-dir> --json";
const DEBUG_TRACE_COMMAND: &str = fret_first_open::demo_metrics_debug::DEBUG_TRACE_COMMAND;
const DEMO_METRICS_DEBUG_OWNER_DOC: &str = fret_first_open::demo_metrics_debug::OWNER_DOC;
const DEMO_METRICS_DEBUG_ACTION_METADATA_DOC: &str =
    fret_first_open::demo_metrics_debug::ACTION_METADATA_DOC;
const DEMO_METRICS_DEBUG_DOCKING_OWNER_DOC: &str =
    fret_first_open::demo_metrics_debug::DOCKING_OWNER_DOC;
const DEMO_METRICS_DEBUG_WAYLAND_ACCEPTANCE_DOC: &str =
    fret_first_open::demo_metrics_debug::WAYLAND_ACCEPTANCE_DOC;
const DOCKING_ARBITRATION_COMMAND: &str =
    fret_first_open::demo_metrics_debug::DOCKING_ARBITRATION_COMMAND;
const DOCKING_CAMPAIGN_VALIDATE_COMMAND: &str =
    fret_first_open::demo_metrics_debug::DOCKING_CAMPAIGN_VALIDATE_COMMAND;
const DOCKING_POLICY_SKIP_COMMAND: &str =
    fret_first_open::demo_metrics_debug::DOCKING_POLICY_SKIP_COMMAND;
type DemoMetricsDebugActionSpec = fret_first_open::demo_metrics_debug::RouteCommand;
const DEMO_METRICS_DEBUG_ACTIONS: &[DemoMetricsDebugActionSpec] =
    fret_first_open::demo_metrics_debug::ACTION_COMMANDS;
const RECENT_EVIDENCE_GATE_RUNS_DIR: &str = ".fret/diag/gate-runs";
const RECENT_EVIDENCE_WORKFLOW_RUNS_DIR: &str = ".fret/diag/workflow-runs";
const RECENT_EVIDENCE_FOLLOWUPS_DIR: &str = ".fret/diag/followups";
const RECENT_EVIDENCE_GATE_RUN_KIND: &str = "fret_devtools_gate_run_result";
const RECENT_EVIDENCE_WORKFLOW_RUN_KIND: &str = "fret_devtools_workflow_run_result";
const RECENT_EVIDENCE_FOLLOWUP_KIND: &str = "fret_devtools_regression_followup_result";

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct SessionlessResourceSpec {
    uri: &'static str,
    name: &'static str,
    mime_type: &'static str,
    description: &'static str,
}

fn sessionless_resource_specs() -> &'static [SessionlessResourceSpec] {
    &[
        SessionlessResourceSpec {
            uri: RESOURCE_URI_FIRST_OPEN_MD,
            name: "first-open.md",
            mime_type: "text/markdown",
            description: "Canonical DevTools MCP first-open diagnostics path, including the shared IMUI product-chain evidence workflow.",
        },
        SessionlessResourceSpec {
            uri: RESOURCE_URI_RECENT_EVIDENCE_JSON,
            name: "recent-evidence.json",
            mime_type: "application/json",
            description: "Recent GUI-launched gate/workflow/follow-up evidence restored from .fret/diag result records.",
        },
    ]
}

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
    #[allow(clippy::too_many_arguments)]
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

    async fn resolve_regression_context(
        &self,
        repo_root: &Path,
        session_id: Option<String>,
        dir: Option<String>,
    ) -> Result<(PathBuf, String, Option<String>), String> {
        let (dir_abs, resolved_session_id) = if let Some(dir) =
            dir.as_deref().map(str::trim).filter(|s| !s.is_empty())
        {
            (resolve_repo_path(repo_root, dir), None)
        } else {
            let session_id = self.resolve_session_id(session_id).await?;
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
            }
            .ok_or_else(|| {
                "missing dir and no bundle.dumped available for the selected session".to_string()
            })?;
            let dir_abs = artifacts_root_from_bundle_dumped_payload(repo_root, &dumped_payload)
                .ok_or_else(|| {
                    "bundle.dumped missing out_dir/dir for artifacts root resolution".to_string()
                })?;
            (dir_abs, Some(session_id))
        };

        let dir_arg = dir_abs
            .strip_prefix(repo_root)
            .ok()
            .map(|p| p.to_string_lossy().to_string())
            .unwrap_or_else(|| dir_abs.to_string_lossy().to_string());

        Ok((dir_abs, dir_arg, resolved_session_id))
    }

    async fn notify_resource_updates(
        &self,
        notify_resources_list_changed: bool,
        resource_updated_uris: Vec<String>,
    ) {
        if !notify_resources_list_changed && resource_updated_uris.is_empty() {
            return;
        }
        let peer = self.peer.lock().await.clone();
        let subscribed = self.subscribed_resources.lock().await.clone();
        if let Some(peer) = peer {
            if notify_resources_list_changed {
                let n = ResourceListChangedNotification {
                    method: Default::default(),
                    extensions: Extensions::default(),
                };
                let _ = peer
                    .send_notification(ServerNotification::ResourceListChangedNotification(n))
                    .await;
            }

            for uri in resource_updated_uris {
                if !subscribed.contains(&uri) {
                    continue;
                }
                let n = ResourceUpdatedNotification::new(ResourceUpdatedNotificationParam { uri });
                let _ = peer
                    .send_notification(ServerNotification::ResourceUpdatedNotification(n))
                    .await;
            }
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

    #[tool(
        description = "Aggregate regression summaries under a directory and return the generated summary/index paths. When dir is omitted, reuse the current session artifacts root from the latest bundle.dumped event."
    )]
    async fn fret_diag_regression_summarize(
        &self,
        params: rmcp::handler::server::wrapper::Parameters<RegressionSummarizeRequestV1>,
    ) -> Result<Json<RegressionSummarizeResultV1>, String> {
        let repo_root = repo_root_from_manifest_dir()
            .or_else(|| std::env::current_dir().ok())
            .ok_or_else(|| "failed to resolve repo root".to_string())?;

        let (dir_abs, dir_arg, resolved_session_id) = self
            .resolve_regression_context(
                &repo_root,
                params.0.session_id.clone(),
                params.0.dir.clone(),
            )
            .await?;

        let mut args = vec![
            "--dir".to_string(),
            dir_arg.clone(),
            "summarize".to_string(),
        ];
        if let Some(inputs) = params.0.inputs.clone() {
            args.extend(inputs.into_iter().filter(|s| !s.trim().is_empty()));
        }

        tokio::task::spawn_blocking(move || fret_diag::diag_cmd(args))
            .await
            .map_err(|e| e.to_string())?
            .map_err(|e| e.to_string())?;

        let summary_path = dir_abs.join(DIAG_REGRESSION_SUMMARY_FILENAME_V1);
        let index_path = dir_abs.join(DIAG_REGRESSION_INDEX_FILENAME_V1);
        let include_json = params.0.include_json.unwrap_or(false);

        if let Some(session_id) = resolved_session_id.as_deref() {
            let selected_session_id = self.selected_session_id.lock().await.clone();
            let uris = session_resource_uris(
                session_id,
                selected_session_id.as_deref(),
                &[
                    RESOURCE_KIND_REGRESSION_SUMMARY_JSON,
                    RESOURCE_KIND_REGRESSION_INDEX_JSON,
                ],
            );
            self.notify_resource_updates(true, uris).await;
        }

        Ok(Json(RegressionSummarizeResultV1 {
            schema_version: 1,
            dir: dir_arg,
            summary_path: summary_path.to_string_lossy().to_string(),
            index_path: index_path.to_string_lossy().to_string(),
            summary_json: if include_json {
                Some(std::fs::read_to_string(&summary_path).map_err(|e| e.to_string())?)
            } else {
                None
            },
            index_json: if include_json {
                Some(std::fs::read_to_string(&index_path).map_err(|e| e.to_string())?)
            } else {
                None
            },
        }))
    }

    #[tool(
        description = "Read regression.index.json and return a first-open dashboard summary. When dir is omitted, reuse the current session artifacts root from the latest bundle.dumped event."
    )]
    async fn fret_diag_regression_dashboard(
        &self,
        params: rmcp::handler::server::wrapper::Parameters<RegressionDashboardRequestV1>,
    ) -> Result<Json<RegressionDashboardResultV1>, String> {
        let repo_root = repo_root_from_manifest_dir()
            .or_else(|| std::env::current_dir().ok())
            .ok_or_else(|| "failed to resolve repo root".to_string())?;

        let (dir_abs, dir_arg, _resolved_session_id) = self
            .resolve_regression_context(
                &repo_root,
                params.0.session_id.clone(),
                params.0.dir.clone(),
            )
            .await?;
        let index_path = dir_abs.join(DIAG_REGRESSION_INDEX_FILENAME_V1);
        if !index_path.is_file() {
            let summary_path = dir_abs.join(DIAG_REGRESSION_SUMMARY_FILENAME_V1);
            if summary_path.is_file() {
                return Err(format!(
                    "regression.index.json is missing under {} (call fret_diag_regression_summarize or `fretboard-dev diag summarize --dir {}` first)",
                    dir_abs.display(),
                    dir_arg,
                ));
            }
            return Err(format!(
                "regression.index.json not found under {}",
                dir_abs.display()
            ));
        }

        let index_json = std::fs::read_to_string(&index_path).map_err(|e| e.to_string())?;
        let payload: serde_json::Value = serde_json::from_str(&index_json)
            .map_err(|e| format!("invalid dashboard index {}: {}", index_path.display(), e))?;

        Ok(Json(build_regression_dashboard_result(
            dir_arg,
            &index_path,
            &payload,
            params.0.top.unwrap_or(5).max(1),
            params.0.include_json.unwrap_or(false),
            Some(index_json),
        )))
    }

    #[tool(
        description = "Read recent GUI-launched gate/workflow/follow-up result records from .fret/diag and return a compact first-open evidence report."
    )]
    async fn fret_diag_recent_evidence(
        &self,
        params: rmcp::handler::server::wrapper::Parameters<RecentEvidenceRequestV1>,
    ) -> Result<Json<RecentEvidenceReportV1>, String> {
        let repo_root = repo_root_from_manifest_dir()
            .or_else(|| std::env::current_dir().ok())
            .ok_or_else(|| "failed to resolve repo root".to_string())?;
        let limit = params.0.limit.unwrap_or(8).clamp(1, 64);

        Ok(Json(build_recent_evidence_report(&repo_root, limit)))
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
        description = "Request hit_test.explain and wait for hit_test.explain_ack (returns JSON text)."
    )]
    async fn fret_diag_hit_test_explain(
        &self,
        params: rmcp::handler::server::wrapper::Parameters<HitTestExplainRequestV1>,
    ) -> Result<String, String> {
        let kind = *self.client_kind.lock().await;
        if kind != DiagTransportKind::WebSocket {
            return Err("hit_test.explain requires WebSocket transport".to_string());
        }

        let session_id = self.resolve_session_id(params.0.session_id.clone()).await?;
        let selector = selector_from_request(&params.0)?;
        let request_id = NEXT_REQUEST_ID.fetch_add(1, Ordering::Relaxed);
        self.client_tx
            .send(ClientCommand::Send(DiagTransportMessageV1 {
                schema_version: 1,
                r#type: "hit_test.explain".to_string(),
                session_id: Some(session_id.clone()),
                request_id: Some(request_id),
                payload: serde_json::json!({
                    "schema_version": 1,
                    "window": params.0.window,
                    "target": selector,
                }),
            }))
            .map_err(|_| "client task is not running".to_string())?;

        let msg = self
            .wait_for_type_session_request_id(
                "hit_test.explain_ack",
                &session_id,
                request_id,
                params.0.timeout_ms,
            )
            .await
            .ok_or_else(|| "timeout waiting for hit_test.explain_ack".to_string())?;
        Ok(serde_json::to_string_pretty(&msg.payload).unwrap_or_else(|_| "{}".to_string()))
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
                && let Ok(parsed) = serde_json::from_value::<UiScriptResultV1>(msg.payload.clone())
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
        inbox.remove(pos)
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
        inbox.remove(pos)
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
#[allow(clippy::manual_async_fn)]
impl ServerHandler for FretDevtoolsMcp {
    fn get_info(&self) -> ServerInfo {
        ServerInfo {
            instructions: Some(mcp_server_instructions().into()),
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
            for spec in sessionless_resource_specs() {
                let mut resource = RawResource::new(spec.uri, spec.name);
                resource.mime_type = Some(spec.mime_type.to_string());
                resource.description = Some(spec.description.to_string());
                resources.push(resource.no_annotation());
            }

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
                    && repro_path.is_file()
                {
                    let mut repro = RawResource::new(
                        format!("{base}{RESOURCE_KIND_REPRO_SUMMARY_JSON}"),
                        format!("repro.summary.json [{sid}]"),
                    );
                    repro.mime_type = Some("application/json".to_string());
                    repro.description = Some(
                        "Repro summary generated by fretboard-dev diag repro (if present in the artifacts root)."
                            .to_string(),
                    );
                    resources.push(repro.no_annotation());
                }

                if let Some(summary_path) =
                    regression_summary_path_from_latest_bundle_dumped_payload(&inbox, &sid)
                    && summary_path.is_file()
                {
                    let mut summary = RawResource::new(
                        format!("{base}{RESOURCE_KIND_REGRESSION_SUMMARY_JSON}"),
                        format!("regression.summary.json [{sid}]"),
                    );
                    summary.mime_type = Some("application/json".to_string());
                    summary.description = Some(
                        "Aggregate regression summary for the artifacts root (if present on disk)."
                            .to_string(),
                    );
                    resources.push(summary.no_annotation());
                }

                if let Some(index_path) =
                    regression_index_path_from_latest_bundle_dumped_payload(&inbox, &sid)
                    && index_path.is_file()
                {
                    let mut index = RawResource::new(
                        format!("{base}{RESOURCE_KIND_REGRESSION_INDEX_JSON}"),
                        format!("regression.index.json [{sid}]"),
                    );
                    index.mime_type = Some("application/json".to_string());
                    index.description = Some(
                        "Consumer-oriented regression index for the artifacts root (if present on disk)."
                            .to_string(),
                    );
                    resources.push(index.no_annotation());
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

            let mut templates = Vec::new();
            for spec in sessionless_resource_specs() {
                templates.push(mk(spec.uri, spec.name, spec.mime_type, spec.description));
            }
            templates.extend([
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
                mk(
                    "fret-diag://sessions/{session_id}/regression.summary.json",
                    "regression.summary.json",
                    "application/json",
                    "Aggregate regression summary for a session artifacts root (only if present on disk).",
                ),
                mk(
                    "fret-diag://sessions/{session_id}/regression.index.json",
                    "regression.index.json",
                    "application/json",
                    "Consumer-oriented regression index for a session artifacts root (only if present on disk).",
                ),
            ]);

            Ok(ListResourceTemplatesResult::with_all_items(templates))
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

            if parsed.kind == RESOURCE_KIND_FIRST_OPEN_MD {
                return Ok(ReadResourceResult {
                    contents: vec![ResourceContents::TextResourceContents {
                        uri: uri.to_string(),
                        mime_type: Some("text/markdown".to_string()),
                        text: mcp_first_open_resource_text(),
                        meta: None,
                    }],
                });
            }

            if parsed.kind == RESOURCE_KIND_RECENT_EVIDENCE_JSON {
                let repo_root = repo_root_from_manifest_dir()
                    .or_else(|| std::env::current_dir().ok())
                    .ok_or_else(|| McpError::internal_error("failed to resolve repo root", None))?;
                let text = recent_evidence_resource_text(&repo_root)
                    .map_err(|err| McpError::internal_error(err.to_string(), None))?;
                return Ok(ReadResourceResult {
                    contents: vec![ResourceContents::TextResourceContents {
                        uri: uri.to_string(),
                        mime_type: Some("application/json".to_string()),
                        text,
                        meta: None,
                    }],
                });
            }

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
                RESOURCE_KIND_REGRESSION_SUMMARY_JSON => {
                    let path = regression_summary_path_from_bundle_dumped_payload(
                        &repo_root,
                        &dumped_payload,
                    )
                    .ok_or_else(|| {
                        McpError::resource_not_found("bundle.dumped missing out_dir/dir", None)
                    })?;
                    if !path.is_file() {
                        return Err(McpError::resource_not_found(
                            "regression.summary.json not found for this session",
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
                RESOURCE_KIND_REGRESSION_INDEX_JSON => {
                    let path = regression_index_path_from_bundle_dumped_payload(
                        &repo_root,
                        &dumped_payload,
                    )
                    .ok_or_else(|| {
                        McpError::resource_not_found("bundle.dumped missing out_dir/dir", None)
                    })?;
                    if !path.is_file() {
                        return Err(McpError::resource_not_found(
                            "regression.index.json not found for this session",
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
struct RegressionSummarizeRequestV1 {
    #[serde(default)]
    session_id: Option<String>,
    #[serde(default)]
    dir: Option<String>,
    #[serde(default)]
    inputs: Option<Vec<String>>,
    #[serde(default)]
    include_json: Option<bool>,
}

#[derive(Serialize, Deserialize, JsonSchema)]
struct RegressionSummarizeResultV1 {
    schema_version: u32,
    dir: String,
    summary_path: String,
    index_path: String,
    #[serde(default)]
    summary_json: Option<String>,
    #[serde(default)]
    index_json: Option<String>,
}

#[derive(Serialize, Deserialize, JsonSchema)]
struct RegressionDashboardRequestV1 {
    #[serde(default)]
    session_id: Option<String>,
    #[serde(default)]
    dir: Option<String>,
    #[serde(default)]
    top: Option<usize>,
    #[serde(default)]
    include_json: Option<bool>,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
struct DashboardCountEntryV1 {
    key: String,
    count: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
struct DashboardReasonCodeEntryV1 {
    reason_code: String,
    count: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
struct DashboardFailingSummaryEntryV1 {
    path: String,
    lane: String,
    failures: u64,
    items_total: u64,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize, JsonSchema)]
struct RegressionDashboardEvidenceV1 {
    #[serde(default)]
    bundle_dirs: Vec<String>,
    #[serde(default)]
    capability_sources: Vec<String>,
    #[serde(default)]
    capabilities_check_paths: Vec<String>,
    #[serde(default)]
    perf_evidence_lines: Vec<String>,
    #[serde(default)]
    first_open_evidence_lines: Vec<String>,
    #[serde(default)]
    share_artifacts: Vec<String>,
    #[serde(default)]
    followup_command_lines: Vec<String>,
    #[serde(default)]
    runnable_followup_command_lines: Vec<String>,
    #[serde(default)]
    manual_followup_command_lines: Vec<String>,
    #[serde(default)]
    followup_commands: Vec<RegressionDashboardFollowupCommandV1>,
    #[serde(default)]
    runnable_followup_commands: Vec<RegressionDashboardFollowupCommandV1>,
    #[serde(default)]
    manual_followup_commands: Vec<RegressionDashboardFollowupCommandV1>,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
struct RegressionDashboardFollowupCommandV1 {
    id: String,
    label: String,
    command_line: String,
    #[serde(default)]
    diag_args: Vec<String>,
    #[serde(default)]
    requires_baseline: bool,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    target_bundle_dir: Option<String>,
}

#[derive(Serialize, Deserialize, JsonSchema)]
struct RegressionDashboardResultV1 {
    schema_version: u32,
    dir: String,
    index_path: String,
    #[serde(default)]
    kind: Option<String>,
    #[serde(default)]
    out_dir: Option<String>,
    summaries_total: u64,
    items_total: u64,
    status_counters: Vec<DashboardCountEntryV1>,
    lane_counters: Vec<DashboardCountEntryV1>,
    tool_counters: Vec<DashboardCountEntryV1>,
    top_reason_codes: Vec<DashboardReasonCodeEntryV1>,
    failing_summaries: Vec<DashboardFailingSummaryEntryV1>,
    #[serde(default)]
    bundle_dirs: Vec<String>,
    #[serde(default)]
    capability_sources: Vec<String>,
    #[serde(default)]
    capabilities_check_paths: Vec<String>,
    #[serde(default)]
    perf_evidence_lines: Vec<String>,
    #[serde(default)]
    first_open_evidence_lines: Vec<String>,
    #[serde(default)]
    share_artifacts: Vec<String>,
    #[serde(default)]
    followup_command_lines: Vec<String>,
    #[serde(default)]
    runnable_followup_command_lines: Vec<String>,
    #[serde(default)]
    manual_followup_command_lines: Vec<String>,
    #[serde(default)]
    followup_commands: Vec<RegressionDashboardFollowupCommandV1>,
    #[serde(default)]
    runnable_followup_commands: Vec<RegressionDashboardFollowupCommandV1>,
    #[serde(default)]
    manual_followup_commands: Vec<RegressionDashboardFollowupCommandV1>,
    human_summary: String,
    #[serde(default)]
    index_json: Option<String>,
}

#[derive(Serialize, Deserialize, JsonSchema)]
struct RecentEvidenceRequestV1 {
    /// Maximum records per evidence lane (default 8, max 64).
    #[serde(default)]
    limit: Option<usize>,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
struct RecentEvidenceEntryV1 {
    kind: String,
    id: String,
    label: String,
    status: String,
    result_path: String,
    command_line: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    started_unix_ms: Option<u64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    finished_unix_ms: Option<u64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    bundle_dir: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    error: Option<String>,
}

#[derive(Serialize, Deserialize, JsonSchema)]
struct RecentEvidenceReportV1 {
    schema_version: u32,
    artifacts_root: String,
    gate_runs_dir: String,
    workflow_runs_dir: String,
    followups_dir: String,
    gate_runs_count: usize,
    workflow_runs_count: usize,
    followups_count: usize,
    failing_count: usize,
    latest_gate_run: Option<RecentEvidenceEntryV1>,
    latest_workflow_run: Option<RecentEvidenceEntryV1>,
    latest_followup: Option<RecentEvidenceEntryV1>,
    first_failed: Option<RecentEvidenceEntryV1>,
    next_action: String,
    human_summary: String,
}

impl From<RegressionBundleFollowupCommandV1> for RegressionDashboardFollowupCommandV1 {
    fn from(value: RegressionBundleFollowupCommandV1) -> Self {
        Self {
            id: value.id,
            label: value.label,
            command_line: value.command_line,
            diag_args: value.diag_args,
            requires_baseline: value.requires_baseline,
            target_bundle_dir: value.target_bundle_dir,
        }
    }
}

impl From<DashboardCountEntry> for DashboardCountEntryV1 {
    fn from(value: DashboardCountEntry) -> Self {
        Self {
            key: value.key,
            count: value.count,
        }
    }
}

impl From<DashboardReasonCodeEntry> for DashboardReasonCodeEntryV1 {
    fn from(value: DashboardReasonCodeEntry) -> Self {
        Self {
            reason_code: value.reason_code,
            count: value.count,
        }
    }
}

impl From<DashboardFailingSummaryEntry> for DashboardFailingSummaryEntryV1 {
    fn from(value: DashboardFailingSummaryEntry) -> Self {
        Self {
            path: value.path,
            lane: value.lane,
            failures: value.failures,
            items_total: value.items_total,
        }
    }
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
struct HitTestExplainRequestV1 {
    #[serde(default)]
    session_id: Option<String>,
    /// Target window ffi id.
    window: u64,
    /// Convenience selector input for the common case.
    #[serde(default)]
    test_id: Option<String>,
    /// Full `UiSelectorV1` JSON text. Takes precedence over `test_id` when present.
    #[serde(default)]
    selector_json: Option<String>,
    /// Wait timeout for `hit_test.explain_ack`.
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

fn selector_from_request(req: &HitTestExplainRequestV1) -> Result<UiSelectorV1, String> {
    if let Some(selector_json) = req
        .selector_json
        .as_deref()
        .map(str::trim)
        .filter(|s| !s.is_empty())
    {
        return serde_json::from_str(selector_json).map_err(|e| e.to_string());
    }
    if let Some(test_id) = req
        .test_id
        .as_deref()
        .map(str::trim)
        .filter(|s| !s.is_empty())
    {
        return Ok(UiSelectorV1::TestId {
            id: test_id.to_string(),
            root_z_index: None,
        });
    }
    Err("missing selector_json or test_id".to_string())
}

fn serde_default_true() -> bool {
    true
}

pub(crate) async fn run() -> anyhow::Result<()> {
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

#[allow(clippy::too_many_arguments)]
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
                    let result = connect_client(&ws_defaults, cfg).map(|new_client| {
                        client = new_client;
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
                        let selected = selected_session_id.lock().await.clone();
                        resource_updated_uris.extend(session_resource_uris(
                            sid,
                            selected.as_deref(),
                            &[
                                RESOURCE_KIND_BUNDLE_JSON,
                                RESOURCE_KIND_BUNDLE_ZIP,
                                RESOURCE_KIND_REPRO_SUMMARY_JSON,
                                RESOURCE_KIND_REGRESSION_SUMMARY_JSON,
                                RESOURCE_KIND_REGRESSION_INDEX_JSON,
                            ],
                        ));
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
                if selected.is_none() || !contains_selected {
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

fn session_resource_uris(
    session_id: &str,
    selected_session_id: Option<&str>,
    resource_kinds: &[&str],
) -> Vec<String> {
    let mut uris = Vec::with_capacity(resource_kinds.len() * 2);
    let base = format!("{RESOURCE_SCHEME}sessions/{session_id}/");
    for kind in resource_kinds {
        uris.push(format!("{base}{kind}"));
    }
    if selected_session_id == Some(session_id) {
        let selected_base = format!("{RESOURCE_SCHEME}selected/");
        for kind in resource_kinds {
            uris.push(format!("{selected_base}{kind}"));
        }
    }
    uris
}

fn collect_regression_dashboard_evidence(summary_path: &Path) -> RegressionDashboardEvidenceV1 {
    let Ok(summary_json) = std::fs::read_to_string(summary_path) else {
        return RegressionDashboardEvidenceV1::default();
    };
    let Ok(summary) = serde_json::from_str::<RegressionSummaryV1>(&summary_json) else {
        return RegressionDashboardEvidenceV1::default();
    };
    let drilldown = regression_summary_drilldown(&summary);
    let followup_commands =
        regression_bundle_followup_commands(drilldown.bundle_dirs.iter().map(String::as_str));
    let followup_command_lines =
        regression_bundle_followup_command_lines(drilldown.bundle_dirs.iter().map(String::as_str));
    let runnable_followup_command_lines = followup_commands
        .iter()
        .filter(|command| !command.requires_baseline)
        .map(|command| command.display_line())
        .collect();
    let manual_followup_command_lines = followup_commands
        .iter()
        .filter(|command| command.requires_baseline)
        .map(|command| command.display_line())
        .collect();
    let followup_command_entries = followup_commands
        .iter()
        .cloned()
        .map(RegressionDashboardFollowupCommandV1::from)
        .collect();
    let runnable_followup_command_entries = followup_commands
        .iter()
        .filter(|command| !command.requires_baseline)
        .cloned()
        .map(RegressionDashboardFollowupCommandV1::from)
        .collect();
    let manual_followup_command_entries = followup_commands
        .iter()
        .filter(|command| command.requires_baseline)
        .cloned()
        .map(RegressionDashboardFollowupCommandV1::from)
        .collect();
    RegressionDashboardEvidenceV1 {
        bundle_dirs: drilldown.bundle_dirs,
        capability_sources: drilldown.capability_sources,
        capabilities_check_paths: drilldown.capabilities_check_paths,
        perf_evidence_lines: drilldown.perf_evidence_lines,
        first_open_evidence_lines: drilldown.first_open_evidence_lines,
        share_artifacts: drilldown.share_artifacts,
        followup_command_lines,
        runnable_followup_command_lines,
        manual_followup_command_lines,
        followup_commands: followup_command_entries,
        runnable_followup_commands: runnable_followup_command_entries,
        manual_followup_commands: manual_followup_command_entries,
    }
}

fn build_recent_evidence_report(repo_root: &Path, limit: usize) -> RecentEvidenceReportV1 {
    let limit = limit.max(1);
    let gate_runs_dir = repo_root.join(RECENT_EVIDENCE_GATE_RUNS_DIR);
    let workflow_runs_dir = repo_root.join(RECENT_EVIDENCE_WORKFLOW_RUNS_DIR);
    let followups_dir = repo_root.join(RECENT_EVIDENCE_FOLLOWUPS_DIR);

    let gate_runs =
        load_recent_evidence_entries(&gate_runs_dir, RECENT_EVIDENCE_GATE_RUN_KIND, "gate", limit);
    let workflow_runs = load_recent_evidence_entries(
        &workflow_runs_dir,
        RECENT_EVIDENCE_WORKFLOW_RUN_KIND,
        "workflow",
        limit,
    );
    let followups = load_recent_evidence_entries(
        &followups_dir,
        RECENT_EVIDENCE_FOLLOWUP_KIND,
        "follow-up",
        limit,
    );

    let latest_gate_run = gate_runs.first().cloned();
    let latest_workflow_run = workflow_runs.first().cloned();
    let latest_followup = followups.first().cloned();
    let first_failed = recent_evidence_latest_failed_entry(&gate_runs, &workflow_runs, &followups);
    let failing_count = gate_runs
        .iter()
        .chain(workflow_runs.iter())
        .chain(followups.iter())
        .filter(|entry| recent_evidence_status_is_failing(&entry.status))
        .count();
    let evidence_empty = gate_runs.is_empty() && workflow_runs.is_empty() && followups.is_empty();
    let next_action = recent_evidence_report_next_action(evidence_empty, first_failed.as_ref());

    let artifacts_root = repo_root
        .join(".fret")
        .join("diag")
        .to_string_lossy()
        .to_string();
    let gate_runs_dir_text = gate_runs_dir.to_string_lossy().to_string();
    let workflow_runs_dir_text = workflow_runs_dir.to_string_lossy().to_string();
    let followups_dir_text = followups_dir.to_string_lossy().to_string();
    let human_summary = recent_evidence_human_summary(RecentEvidenceHumanSummaryInput {
        gate_runs_count: gate_runs.len(),
        workflow_runs_count: workflow_runs.len(),
        followups_count: followups.len(),
        failing_count,
        latest_gate_run: latest_gate_run.as_ref(),
        latest_workflow_run: latest_workflow_run.as_ref(),
        latest_followup: latest_followup.as_ref(),
        first_failed: first_failed.as_ref(),
        next_action: &next_action,
    });

    RecentEvidenceReportV1 {
        schema_version: 1,
        artifacts_root,
        gate_runs_dir: gate_runs_dir_text,
        workflow_runs_dir: workflow_runs_dir_text,
        followups_dir: followups_dir_text,
        gate_runs_count: gate_runs.len(),
        workflow_runs_count: workflow_runs.len(),
        followups_count: followups.len(),
        failing_count,
        latest_gate_run,
        latest_workflow_run,
        latest_followup,
        first_failed,
        next_action,
        human_summary,
    }
}

fn recent_evidence_resource_text(repo_root: &Path) -> Result<String, serde_json::Error> {
    serde_json::to_string_pretty(&build_recent_evidence_report(repo_root, 8))
}

fn recent_evidence_latest_failed_entry(
    gate_runs: &[RecentEvidenceEntryV1],
    workflow_runs: &[RecentEvidenceEntryV1],
    followups: &[RecentEvidenceEntryV1],
) -> Option<RecentEvidenceEntryV1> {
    let entries = gate_runs
        .iter()
        .chain(workflow_runs.iter())
        .chain(followups.iter())
        .filter(|entry| recent_evidence_status_is_failing(&entry.status))
        .cloned()
        .collect::<Vec<_>>();
    entries
        .iter()
        .filter_map(|entry| {
            recent_evidence_entry_sort_timestamp(entry).map(|timestamp| {
                (
                    timestamp,
                    RecentEvidenceKindOrder::from_kind(&entry.kind),
                    entry,
                )
            })
        })
        .max_by_key(|(timestamp, kind_order, _entry)| (*timestamp, *kind_order))
        .map(|(_, _, entry)| entry.clone())
        .or_else(|| entries.first().cloned())
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
enum RecentEvidenceKindOrder {
    Followup = 0,
    Workflow = 1,
    Gate = 2,
}

impl RecentEvidenceKindOrder {
    fn from_kind(kind: &str) -> Self {
        match kind {
            "workflow" => Self::Workflow,
            "follow-up" => Self::Followup,
            _ => Self::Gate,
        }
    }
}

fn recent_evidence_result_path_timestamp(path: &str) -> Option<u64> {
    Path::new(path)
        .file_name()
        .and_then(|value| value.to_str())
        .and_then(|file_name| file_name.split_once('-'))
        .and_then(|(prefix, _)| prefix.parse::<u64>().ok())
}

fn recent_evidence_entry_sort_timestamp(entry: &RecentEvidenceEntryV1) -> Option<u64> {
    entry
        .finished_unix_ms
        .or(entry.started_unix_ms)
        .or_else(|| recent_evidence_result_path_timestamp(&entry.result_path))
}

fn load_recent_evidence_entries(
    result_dir: &Path,
    record_kind: &str,
    evidence_kind: &str,
    limit: usize,
) -> Vec<RecentEvidenceEntryV1> {
    let Ok(read_dir) = std::fs::read_dir(result_dir) else {
        return Vec::new();
    };
    let mut candidates = read_dir
        .filter_map(Result::ok)
        .filter_map(|entry| {
            let path = entry.path();
            if path.extension().and_then(|value| value.to_str()) != Some("json") {
                return None;
            }
            let modified_unix_ms = entry
                .metadata()
                .and_then(|metadata| metadata.modified())
                .ok()
                .and_then(system_time_unix_ms);
            let result_json = std::fs::read_to_string(&path).ok()?;
            let evidence_entry = recent_evidence_entry_from_result_json(
                &path,
                &result_json,
                record_kind,
                evidence_kind,
            )?;
            let record_unix_ms = recent_evidence_entry_sort_timestamp(&evidence_entry);
            Some((evidence_entry, record_unix_ms, modified_unix_ms, path))
        })
        .collect::<Vec<_>>();
    candidates.sort_by(
        |(left_entry, left_record, left_modified, left_path),
         (right_entry, right_record, right_modified, right_path)| {
            (
                right_record,
                right_modified,
                right_path,
                &right_entry.result_path,
            )
                .cmp(&(
                    left_record,
                    left_modified,
                    left_path,
                    &left_entry.result_path,
                ))
        },
    );
    candidates
        .into_iter()
        .map(|(entry, _record_unix_ms, _modified_unix_ms, _path)| entry)
        .take(limit)
        .collect()
}

fn system_time_unix_ms(value: std::time::SystemTime) -> Option<u128> {
    value
        .duration_since(std::time::UNIX_EPOCH)
        .ok()
        .map(|duration| duration.as_millis())
}

fn recent_evidence_entry_from_result_json(
    result_path: &Path,
    result_json: &str,
    record_kind: &str,
    evidence_kind: &str,
) -> Option<RecentEvidenceEntryV1> {
    let value = serde_json::from_str::<serde_json::Value>(result_json).ok()?;
    if value.get("kind").and_then(|value| value.as_str()) != Some(record_kind) {
        return None;
    }

    Some(RecentEvidenceEntryV1 {
        kind: evidence_kind.to_string(),
        id: json_string_field_or_dash(&value, "id"),
        label: json_string_field_or_dash(&value, "label"),
        status: json_string_field_or_dash(&value, "status"),
        result_path: result_path.to_string_lossy().to_string(),
        command_line: json_string_field_or_dash(&value, "command_line"),
        started_unix_ms: value
            .get("started_unix_ms")
            .and_then(|value| value.as_u64()),
        finished_unix_ms: value
            .get("finished_unix_ms")
            .and_then(|value| value.as_u64()),
        bundle_dir: value
            .get("bundle_dir")
            .and_then(|value| value.as_str())
            .map(str::trim)
            .filter(|value| !value.is_empty())
            .map(ToOwned::to_owned),
        error: value
            .get("error")
            .and_then(|value| value.as_str())
            .map(str::trim)
            .filter(|value| !value.is_empty())
            .map(ToOwned::to_owned),
    })
}

fn json_string_field_or_dash(value: &serde_json::Value, key: &str) -> String {
    value
        .get(key)
        .and_then(|value| value.as_str())
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .unwrap_or("-")
        .to_string()
}

fn recent_evidence_status_is_failing(status: &str) -> bool {
    let status = status.trim();
    !status.is_empty() && status != "-" && !status.eq_ignore_ascii_case("passed")
}

fn recent_evidence_report_next_action(
    evidence_empty: bool,
    first_failed: Option<&RecentEvidenceEntryV1>,
) -> String {
    if evidence_empty {
        return "run a workflow or generated gate".to_string();
    }
    let Some(first_failed) = first_failed else {
        return "continue from latest passing evidence".to_string();
    };
    format!(
        "inspect failed {} evidence result JSON, then use the GUI Recent Evidence rerun controls when a current session is selected",
        first_failed.kind
    )
}

struct RecentEvidenceHumanSummaryInput<'a> {
    gate_runs_count: usize,
    workflow_runs_count: usize,
    followups_count: usize,
    failing_count: usize,
    latest_gate_run: Option<&'a RecentEvidenceEntryV1>,
    latest_workflow_run: Option<&'a RecentEvidenceEntryV1>,
    latest_followup: Option<&'a RecentEvidenceEntryV1>,
    first_failed: Option<&'a RecentEvidenceEntryV1>,
    next_action: &'a str,
}

fn recent_evidence_human_summary(input: RecentEvidenceHumanSummaryInput<'_>) -> String {
    let mut lines = vec![
        format!(
            "recent evidence counts: gate_runs={} workflow_runs={} followups={} failing={}",
            input.gate_runs_count,
            input.workflow_runs_count,
            input.followups_count,
            input.failing_count,
        ),
        recent_evidence_summary_line("latest gate", input.latest_gate_run),
        recent_evidence_summary_line("latest workflow", input.latest_workflow_run),
        recent_evidence_summary_line("latest follow-up", input.latest_followup),
    ];
    if let Some(first_failed) = input.first_failed {
        lines.push(format!(
            "first failed evidence: {} {} {}",
            first_failed.kind, first_failed.status, first_failed.result_path
        ));
        if let Some(bundle_dir) = first_failed.bundle_dir.as_deref() {
            lines.push(format!("first failed bundle_dir: {bundle_dir}"));
        }
        lines.push(format!(
            "first failed command: {}",
            first_failed.command_line
        ));
    } else {
        lines.push("first failed evidence: <none>".to_string());
    }
    lines.push(format!("next action: {}", input.next_action));
    lines.join("\n")
}

fn recent_evidence_summary_line(label: &str, entry: Option<&RecentEvidenceEntryV1>) -> String {
    let Some(entry) = entry else {
        return format!("{label}: <none>");
    };
    format!(
        "{label}: {} {} {}",
        entry.status, entry.id, entry.result_path
    )
}

fn build_regression_dashboard_result(
    dir: String,
    index_path: &Path,
    payload: &serde_json::Value,
    top: usize,
    include_json: bool,
    index_json: Option<String>,
) -> RegressionDashboardResultV1 {
    let projection = project_dashboard_summary(payload, top);
    let summary_path = index_path.with_file_name(DIAG_REGRESSION_SUMMARY_FILENAME_V1);
    let evidence = collect_regression_dashboard_evidence(&summary_path);
    let mut human_lines = dashboard_human_lines_from_projection(index_path, &projection);
    if !evidence.bundle_dirs.is_empty() {
        human_lines.push("bundle dirs:".to_string());
        human_lines.extend(evidence.bundle_dirs.iter().map(|dir| format!("  - {dir}")));
    }
    if !evidence.capability_sources.is_empty() {
        human_lines.push("capability sources:".to_string());
        human_lines.extend(
            evidence
                .capability_sources
                .iter()
                .map(|source| format!("  - {source}")),
        );
    }
    if !evidence.capabilities_check_paths.is_empty() {
        human_lines.push("capability checks:".to_string());
        human_lines.extend(
            evidence
                .capabilities_check_paths
                .iter()
                .map(|path| format!("  - {path}")),
        );
    }
    if !evidence.perf_evidence_lines.is_empty() {
        human_lines.push("perf evidence:".to_string());
        human_lines.extend(
            evidence
                .perf_evidence_lines
                .iter()
                .map(|line| format!("  - {line}")),
        );
    }
    if !evidence.first_open_evidence_lines.is_empty() {
        human_lines.push("first-open evidence:".to_string());
        human_lines.extend(
            evidence
                .first_open_evidence_lines
                .iter()
                .map(|line| format!("  - {line}")),
        );
    }
    if !evidence.share_artifacts.is_empty() {
        human_lines.push("share artifacts:".to_string());
        human_lines.extend(
            evidence
                .share_artifacts
                .iter()
                .map(|path| format!("  - {path}")),
        );
    }
    if !evidence.followup_command_lines.is_empty() {
        human_lines.push("follow-up commands:".to_string());
        human_lines.extend(
            evidence
                .followup_command_lines
                .iter()
                .map(|line| format!("  - {line}")),
        );
    }
    if !evidence.runnable_followup_command_lines.is_empty() {
        human_lines.push("runnable follow-up commands:".to_string());
        human_lines.extend(
            evidence
                .runnable_followup_command_lines
                .iter()
                .map(|line| format!("  - {line}")),
        );
    }
    if !evidence.manual_followup_command_lines.is_empty() {
        human_lines.push("manual compare follow-up commands:".to_string());
        human_lines.extend(
            evidence
                .manual_followup_command_lines
                .iter()
                .map(|line| format!("  - {line}")),
        );
    }
    let human_summary = human_lines.join("\n");

    RegressionDashboardResultV1 {
        schema_version: 1,
        dir,
        index_path: index_path.to_string_lossy().to_string(),
        kind: projection.kind,
        out_dir: projection.out_dir,
        summaries_total: projection.summaries_total as u64,
        items_total: projection.items_total,
        status_counters: projection
            .status_counters
            .into_iter()
            .map(Into::into)
            .collect(),
        lane_counters: projection
            .lane_counters
            .into_iter()
            .map(Into::into)
            .collect(),
        tool_counters: projection
            .tool_counters
            .into_iter()
            .map(Into::into)
            .collect(),
        top_reason_codes: projection
            .top_reason_codes
            .into_iter()
            .map(Into::into)
            .collect(),
        failing_summaries: projection
            .failing_summaries
            .into_iter()
            .map(Into::into)
            .collect(),
        bundle_dirs: evidence.bundle_dirs,
        capability_sources: evidence.capability_sources,
        capabilities_check_paths: evidence.capabilities_check_paths,
        perf_evidence_lines: evidence.perf_evidence_lines,
        first_open_evidence_lines: evidence.first_open_evidence_lines,
        share_artifacts: evidence.share_artifacts,
        followup_command_lines: evidence.followup_command_lines,
        runnable_followup_command_lines: evidence.runnable_followup_command_lines,
        manual_followup_command_lines: evidence.manual_followup_command_lines,
        followup_commands: evidence.followup_commands,
        runnable_followup_commands: evidence.runnable_followup_commands,
        manual_followup_commands: evidence.manual_followup_commands,
        human_summary,
        index_json: if include_json { index_json } else { None },
    }
}

fn mcp_server_instructions() -> String {
    format!(
        "Fret diagnostics DevTools MCP adapter. Starts a local WS hub and exposes tools to drive inspect/pick/scripts/bundles. Read {RESOURCE_URI_FIRST_OPEN_MD} for the first-open evidence path. Recent evidence: fret_diag_recent_evidence. Product workflow: {IMUI_PRODUCT_WORKFLOW_ID}; default: {IMUI_PRODUCT_WORKFLOW_COMMAND}; focused: {IMUI_PRODUCT_WORKFLOW_FOCUSED_COMMAND}; launched: {IMUI_PRODUCT_WORKFLOW_LAUNCHED_COMMAND}."
    )
}

fn mcp_first_open_lines() -> Vec<String> {
    let mut lines = vec![
        format!("mcp first-open: {DEVTOOLS_FIRST_OPEN_DOC}"),
        format!("mcp workflow: {DEVTOOLS_MCP_DOC}"),
        format!("gui branch: {DEVTOOLS_GUI_BRANCH_DOC}"),
        format!("repo preflight: {DEVTOOLS_REPO_PREFLIGHT_COMMAND}"),
        format!("repo preflight json: {DEVTOOLS_REPO_PREFLIGHT_JSON_COMMAND}"),
        format!("tool-app index: {DEVTOOLS_TOOL_APP_INDEX_COMMAND}"),
        format!("tool-app index json: {DEVTOOLS_TOOL_APP_INDEX_JSON_COMMAND}"),
        format!("resource: {RESOURCE_URI_FIRST_OPEN_MD}"),
        format!("recent evidence resource: {RESOURCE_URI_RECENT_EVIDENCE_JSON}"),
        format!("product workflow: {IMUI_PRODUCT_WORKFLOW_ID}"),
        format!("product workflow command: {IMUI_PRODUCT_WORKFLOW_COMMAND}"),
        format!("product workflow focused: {IMUI_PRODUCT_WORKFLOW_FOCUSED_COMMAND}"),
        format!("product workflow launched: {IMUI_PRODUCT_WORKFLOW_LAUNCHED_COMMAND}"),
        format!("product workflow suite: {IMUI_PRODUCT_WORKFLOW_SUITE}"),
        format!("product workflow docs: {IMUI_PRODUCT_WORKFLOW_DOC}"),
        format!(
            "product workflow artifacts: {}",
            IMUI_PRODUCT_WORKFLOW_ARTIFACTS.join(", ")
        ),
        format!("route: {DEMO_METRICS_DEBUG_ROUTE_ID}"),
        format!("route owner: {DEMO_METRICS_DEBUG_OWNER_DOC}"),
        format!("action metadata owner: {DEMO_METRICS_DEBUG_ACTION_METADATA_DOC}"),
        format!("docking owner: {DEMO_METRICS_DEBUG_DOCKING_OWNER_DOC}"),
        format!("wayland acceptance: {DEMO_METRICS_DEBUG_WAYLAND_ACCEPTANCE_DOC}"),
        "action surface: dedicated DevTools guide panel + MCP first-open action list".to_string(),
        "command palette: deferred until DevTools has a shared command palette contract"
            .to_string(),
    ];
    lines.extend(
        DEMO_METRICS_DEBUG_ACTIONS
            .iter()
            .map(|action| format!("action: {} -> {}", action.label, action.command)),
    );
    lines.extend(DEMO_METRICS_DEBUG_ACTIONS.iter().map(|action| {
        format!(
            "action metadata: {} | id={} | category={} | primary={} | requires_bundle={}",
            action.label, action.id, action.category, action.primary, action.requires_bundle
        )
    }));
    lines.extend([
        format!("demo editor workbench: {DEMO_EDITOR_WORKBENCH_COMMAND}"),
        format!("demo editor proof supporting: {DEMO_EDITOR_PROOF_COMMAND}"),
        format!("demo editor notes: {DEMO_EDITOR_NOTES_COMMAND}"),
        format!("demo device shell: {DEMO_DEVICE_SHELL_COMMAND}"),
        format!("metrics stats: {METRICS_STATS_COMMAND}"),
        format!("metrics layout perf: {METRICS_LAYOUT_PERF_COMMAND}"),
        format!("metrics memory: {METRICS_MEMORY_COMMAND}"),
        format!("debug triage: {DEBUG_TRIAGE_COMMAND}"),
        format!("debug hotspots: {DEBUG_HOTSPOTS_COMMAND}"),
        format!("debug trace: {DEBUG_TRACE_COMMAND}"),
        format!("docking arbitration supporting: {DOCKING_ARBITRATION_COMMAND}"),
        format!("docking campaign validate: {DOCKING_CAMPAIGN_VALIDATE_COMMAND}"),
        format!("docking policy-skip local: {DOCKING_POLICY_SKIP_COMMAND}"),
        "recent evidence tool: fret_diag_recent_evidence".to_string(),
        format!("recent evidence gate runs: {RECENT_EVIDENCE_GATE_RUNS_DIR}"),
        format!("recent evidence workflow runs: {RECENT_EVIDENCE_WORKFLOW_RUNS_DIR}"),
        format!("recent evidence followups: {RECENT_EVIDENCE_FOLLOWUPS_DIR}"),
        "recent evidence next action: inspect failed result JSON or run a workflow/generated gate"
            .to_string(),
    ]);
    lines
}

fn mcp_first_open_resource_text() -> String {
    let mut lines = vec![
        "# Fret DevTools MCP First-open".to_string(),
        String::new(),
        "This resource mirrors the repo-maintainer first-open index without adding a MCP-private workflow schema."
            .to_string(),
        String::new(),
    ];
    lines.extend(
        mcp_first_open_lines()
            .into_iter()
            .map(|line| format!("- {line}")),
    );
    lines.join("\n")
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
    if !pattern.ends_with('*')
        && let Some(last) = parts.iter().rev().find(|p| !p.is_empty())
    {
        return text.ends_with(last);
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
    if rest == RESOURCE_KIND_FIRST_OPEN_MD {
        return Some(ParsedResourceUri {
            session_id: None,
            kind: RESOURCE_KIND_FIRST_OPEN_MD.to_string(),
        });
    }
    if rest == RESOURCE_KIND_RECENT_EVIDENCE_JSON {
        return Some(ParsedResourceUri {
            session_id: None,
            kind: RESOURCE_KIND_RECENT_EVIDENCE_JSON.to_string(),
        });
    }
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
    artifact_path_from_latest_bundle_dumped_payload(
        inbox,
        session_id,
        RESOURCE_KIND_REPRO_SUMMARY_JSON,
    )
}

fn regression_summary_path_from_latest_bundle_dumped_payload(
    inbox: &VecDeque<DiagTransportMessageV1>,
    session_id: &str,
) -> Option<PathBuf> {
    artifact_path_from_latest_bundle_dumped_payload(
        inbox,
        session_id,
        RESOURCE_KIND_REGRESSION_SUMMARY_JSON,
    )
}

fn regression_index_path_from_latest_bundle_dumped_payload(
    inbox: &VecDeque<DiagTransportMessageV1>,
    session_id: &str,
) -> Option<PathBuf> {
    artifact_path_from_latest_bundle_dumped_payload(
        inbox,
        session_id,
        RESOURCE_KIND_REGRESSION_INDEX_JSON,
    )
}

fn artifact_path_from_latest_bundle_dumped_payload(
    inbox: &VecDeque<DiagTransportMessageV1>,
    session_id: &str,
    filename: &str,
) -> Option<PathBuf> {
    let payload = inbox
        .iter()
        .rev()
        .find(|m| m.r#type == "bundle.dumped" && m.session_id.as_deref() == Some(session_id))
        .map(|m| m.payload.clone())?;

    let repo_root = repo_root_from_manifest_dir().or_else(|| std::env::current_dir().ok())?;
    artifact_path_from_bundle_dumped_payload(&repo_root, &payload, filename)
}

fn repro_summary_path_from_bundle_dumped_payload(
    repo_root: &Path,
    dumped_payload: &serde_json::Value,
) -> Option<PathBuf> {
    artifact_path_from_bundle_dumped_payload(
        repo_root,
        dumped_payload,
        RESOURCE_KIND_REPRO_SUMMARY_JSON,
    )
}

fn regression_summary_path_from_bundle_dumped_payload(
    repo_root: &Path,
    dumped_payload: &serde_json::Value,
) -> Option<PathBuf> {
    artifact_path_from_bundle_dumped_payload(
        repo_root,
        dumped_payload,
        RESOURCE_KIND_REGRESSION_SUMMARY_JSON,
    )
}

fn regression_index_path_from_bundle_dumped_payload(
    repo_root: &Path,
    dumped_payload: &serde_json::Value,
) -> Option<PathBuf> {
    artifact_path_from_bundle_dumped_payload(
        repo_root,
        dumped_payload,
        RESOURCE_KIND_REGRESSION_INDEX_JSON,
    )
}

fn artifact_path_from_bundle_dumped_payload(
    repo_root: &Path,
    dumped_payload: &serde_json::Value,
    filename: &str,
) -> Option<PathBuf> {
    let artifacts_root = artifacts_root_from_bundle_dumped_payload(repo_root, dumped_payload)?;
    Some(artifacts_root.join(filename))
}

fn artifacts_root_from_bundle_dumped_payload(
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

    Some(artifacts_root)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_resource_uri_accepts_regression_index_for_selected_alias() {
        let parsed = parse_resource_uri("fret-diag://selected/regression.index.json")
            .expect("selected regression index resource uri should parse");
        assert_eq!(parsed.session_id, None);
        assert_eq!(parsed.kind, RESOURCE_KIND_REGRESSION_INDEX_JSON);
    }

    #[test]
    fn parse_resource_uri_accepts_first_open_resource() {
        let parsed = parse_resource_uri("fret-diag://first-open.md")
            .expect("first-open resource uri should parse");
        assert_eq!(parsed.session_id, None);
        assert_eq!(parsed.kind, RESOURCE_KIND_FIRST_OPEN_MD);
    }

    #[test]
    fn parse_resource_uri_accepts_recent_evidence_resource() {
        let parsed = parse_resource_uri("fret-diag://recent-evidence.json")
            .expect("recent-evidence resource uri should parse");
        assert_eq!(parsed.session_id, None);
        assert_eq!(parsed.kind, RESOURCE_KIND_RECENT_EVIDENCE_JSON);
    }

    #[test]
    fn sessionless_resource_specs_include_first_open_and_recent_evidence() {
        let specs = sessionless_resource_specs();
        assert_eq!(specs.len(), 2);

        let first_open = specs
            .iter()
            .find(|spec| spec.uri == RESOURCE_URI_FIRST_OPEN_MD)
            .expect("first-open sessionless resource spec");
        assert_eq!(first_open.name, "first-open.md");
        assert_eq!(first_open.mime_type, "text/markdown");
        assert!(first_open.description.contains("IMUI product-chain"));

        let recent_evidence = specs
            .iter()
            .find(|spec| spec.uri == RESOURCE_URI_RECENT_EVIDENCE_JSON)
            .expect("recent-evidence sessionless resource spec");
        assert_eq!(recent_evidence.name, "recent-evidence.json");
        assert_eq!(recent_evidence.mime_type, "application/json");
        assert!(recent_evidence.description.contains(".fret/diag"));
    }

    #[test]
    fn mcp_first_open_resource_text_surfaces_imui_product_chain() {
        let text = mcp_first_open_resource_text();
        assert!(text.contains("mcp first-open: docs/diagnostics-first-open.md"));
        assert!(text.contains(
            "mcp workflow: docs/workstreams/diag-devtools-gui-v1/diag-devtools-gui-v1-ai-mcp.md"
        ));
        assert!(text.contains(
            "gui branch: docs/workstreams/diag-fearless-refactor-v2/DEVTOOLS_GUI_DOGFOOD_WORKFLOW.md"
        ));
        assert!(
            text.contains("repo preflight: cargo run -p fretboard-dev -- diag doctor campaigns")
        );
        assert!(text.contains(
            "repo preflight json: cargo run -p fretboard-dev -- diag doctor campaigns --json"
        ));
        assert!(text.contains("tool-app index: cargo run -p fretboard-dev -- list tool-apps"));
        assert!(
            text.contains(
                "tool-app index json: cargo run -p fretboard-dev -- list tool-apps --json"
            )
        );
        assert!(text.contains("resource: fret-diag://first-open.md"));
        assert!(text.contains("recent evidence resource: fret-diag://recent-evidence.json"));
        assert!(text.contains("product workflow: imui-product-chain"));
        assert!(
            text.contains("product workflow command: python tools/diag_gate_imui_product_chain.py")
        );
        assert!(text.contains(
            "product workflow focused: python tools/diag_gate_imui_product_chain.py --only discovery"
        ));
        assert!(text.contains(
            "product workflow launched: python tools/diag_gate_imui_product_chain.py --reuse-built --launched --only perf-docking --release"
        ));
        assert!(text.contains(
            "product workflow suite: tools/diag-scripts/suites/perf-docking-arbitration-steady/suite.json"
        ));
        assert!(text.contains(
            "product workflow docs: docs/workstreams/imui-editor-grade-product-closure-v1/EVIDENCE_AND_GATES.md"
        ));
        assert!(text.contains(
            "product workflow artifacts: perf-docking/regression.summary.json, perf-docking/check.perf_thresholds.json, perf-docking/*/trace.chrome.json"
        ));
        assert!(text.contains("route: demo-metrics-debug"));
        assert!(text.contains(
            "route owner: docs/workstreams/imui-demo-metrics-debug-devtools-v1/WORKSTREAM.json"
        ));
        assert!(text.contains(
            "action metadata owner: docs/workstreams/imui-demo-metrics-debug-action-metadata-v1/WORKSTREAM.json"
        ));
        assert!(text.contains(
            "docking owner: docs/workstreams/docking-multiwindow-imgui-parity/WORKSTREAM.json"
        ));
        assert!(text.contains(
            "wayland acceptance: docs/workstreams/docking-multiwindow-imgui-parity/M5_WAYLAND_COMPOSITOR_ACCEPTANCE_RUNBOOK_2026-04-21.md"
        ));
        assert!(text.contains(
            "action surface: dedicated DevTools guide panel + MCP first-open action list"
        ));
        assert!(text.contains(
            "command palette: deferred until DevTools has a shared command palette contract"
        ));
        assert!(text.contains(
            "action: open workbench -> cargo run -p fret-demo --bin imui_editor_workbench_demo"
        ));
        assert!(text.contains(
            "action: run product discovery -> python tools/diag_gate_imui_product_chain.py --only discovery"
        ));
        assert!(text.contains(
            "action: inspect metrics stats -> cargo run -p fretboard-dev -- diag stats <bundle-or-dir> --json"
        ));
        assert!(text.contains(
            "action: inspect debug trace -> cargo run -p fretboard-dev -- diag trace <bundle-or-dir> --json"
        ));
        assert!(text.contains(
            "action: validate docking campaign -> cargo run -p fretboard-dev -- diag campaign validate tools/diag-campaigns/imui-p3-multiwindow-parity.json --json"
        ));
        assert!(text.contains(
            "action metadata: open workbench | id=open_workbench | category=demo | primary=true | requires_bundle=false"
        ));
        assert!(text.contains(
            "action metadata: inspect debug trace | id=inspect_debug_trace | category=debug | primary=false | requires_bundle=true"
        ));
        assert!(text.contains(
            "demo editor workbench: cargo run -p fret-demo --bin imui_editor_workbench_demo"
        ));
        assert!(text.contains(
            "demo editor proof supporting: cargo run -p fret-demo --bin imui_editor_proof_demo"
        ));
        assert!(text.contains(
            "metrics stats: cargo run -p fretboard-dev -- diag stats <bundle-or-dir> --json"
        ));
        assert!(text.contains(
            "debug trace: cargo run -p fretboard-dev -- diag trace <bundle-or-dir> --json"
        ));
        assert!(text.contains(
            "docking arbitration supporting: cargo run -p fret-demo --bin docking_arbitration_demo"
        ));
        assert!(text.contains(
            "docking campaign validate: cargo run -p fretboard-dev -- diag campaign validate tools/diag-campaigns/imui-p3-multiwindow-parity.json --json"
        ));
        assert!(text.contains(
            "docking policy-skip local: python tools/diag_gate_docking_wayland_policy_skip.py"
        ));
        assert!(text.contains("recent evidence tool: fret_diag_recent_evidence"));
        assert!(text.contains("recent evidence gate runs: .fret/diag/gate-runs"));
        assert!(text.contains("recent evidence workflow runs: .fret/diag/workflow-runs"));
        assert!(text.contains("recent evidence followups: .fret/diag/followups"));
        assert!(text.contains(
            "recent evidence next action: inspect failed result JSON or run a workflow/generated gate"
        ));
    }

    #[test]
    fn build_recent_evidence_report_reads_gui_result_records() {
        let repo_root = std::env::temp_dir().join(format!(
            "fret-devtools-mcp-recent-evidence-{}-{}",
            std::process::id(),
            unix_ms_now()
        ));
        let _ = std::fs::remove_dir_all(&repo_root);
        std::fs::create_dir_all(repo_root.join(RECENT_EVIDENCE_GATE_RUNS_DIR)).unwrap();
        std::fs::create_dir_all(repo_root.join(RECENT_EVIDENCE_WORKFLOW_RUNS_DIR)).unwrap();
        std::fs::create_dir_all(repo_root.join(RECENT_EVIDENCE_FOLLOWUPS_DIR)).unwrap();

        let gate_path = repo_root
            .join(RECENT_EVIDENCE_GATE_RUNS_DIR)
            .join("100-gate.json");
        let workflow_path = repo_root
            .join(RECENT_EVIDENCE_WORKFLOW_RUNS_DIR)
            .join("200-workflow.json");
        let followup_path = repo_root
            .join(RECENT_EVIDENCE_FOLLOWUPS_DIR)
            .join("300-followup.json");
        std::fs::write(
            &gate_path,
            serde_json::to_vec_pretty(&serde_json::json!({
                "schema_version": 1,
                "kind": RECENT_EVIDENCE_GATE_RUN_KIND,
                "id": "pixels-changed",
                "label": "pixels changed",
                "status": "passed",
                "command_line": "cargo run -p fretboard-dev -- diag run gate.json",
                "diag_args": ["run", "gate.json"]
            }))
            .unwrap(),
        )
        .unwrap();
        std::fs::write(
            &workflow_path,
            serde_json::to_vec_pretty(&serde_json::json!({
                "schema_version": 1,
                "kind": RECENT_EVIDENCE_WORKFLOW_RUN_KIND,
                "id": "perf-docking-suite",
                "label": "perf docking suite",
                "status": "failed",
                "command_line": "cargo run -p fretboard-dev -- diag suite perf-docking",
                "diag_args": ["suite", "perf-docking"]
            }))
            .unwrap(),
        )
        .unwrap();
        std::fs::write(
            &followup_path,
            serde_json::to_vec_pretty(&serde_json::json!({
                "schema_version": 1,
                "kind": RECENT_EVIDENCE_FOLLOWUP_KIND,
                "id": "trace",
                "label": "trace",
                "status": "passed",
                "command_line": "cargo run -p fretboard-dev -- diag trace bundle --json",
                "bundle_dir": "target/fret-diag/run-a"
            }))
            .unwrap(),
        )
        .unwrap();

        let report = build_recent_evidence_report(&repo_root, 8);

        assert_eq!(report.gate_runs_count, 1);
        assert_eq!(report.workflow_runs_count, 1);
        assert_eq!(report.followups_count, 1);
        assert_eq!(report.failing_count, 1);
        assert_eq!(
            report
                .latest_gate_run
                .as_ref()
                .map(|entry| entry.kind.as_str()),
            Some("gate")
        );
        assert_eq!(
            report
                .first_failed
                .as_ref()
                .map(|entry| entry.kind.as_str()),
            Some("workflow")
        );
        assert!(report.human_summary.contains("recent evidence counts:"));
        assert!(
            report
                .human_summary
                .contains("latest workflow: failed perf-docking-suite")
        );
        assert!(
            report
                .human_summary
                .contains("first failed evidence: workflow failed")
        );
        assert!(
            report
                .next_action
                .contains("inspect failed workflow evidence result JSON")
        );

        let _ = std::fs::remove_dir_all(&repo_root);
    }

    #[test]
    fn recent_evidence_status_is_failing_ignores_empty_placeholder_and_passed_case() {
        assert!(!recent_evidence_status_is_failing(""));
        assert!(!recent_evidence_status_is_failing("   "));
        assert!(!recent_evidence_status_is_failing("-"));
        assert!(!recent_evidence_status_is_failing("passed"));
        assert!(!recent_evidence_status_is_failing("Passed"));
        assert!(!recent_evidence_status_is_failing("PASSED"));
        assert!(recent_evidence_status_is_failing("failed"));
        assert!(recent_evidence_status_is_failing("error"));
    }

    #[test]
    fn recent_evidence_resource_text_matches_report_shape() {
        let repo_root = std::env::temp_dir().join(format!(
            "fret-devtools-mcp-recent-evidence-resource-{}-{}",
            std::process::id(),
            unix_ms_now()
        ));
        let _ = std::fs::remove_dir_all(&repo_root);
        std::fs::create_dir_all(repo_root.join(RECENT_EVIDENCE_WORKFLOW_RUNS_DIR)).unwrap();
        let workflow_path = repo_root
            .join(RECENT_EVIDENCE_WORKFLOW_RUNS_DIR)
            .join("100-workflow.json");
        std::fs::write(
            &workflow_path,
            serde_json::to_vec_pretty(&serde_json::json!({
                "schema_version": 1,
                "kind": RECENT_EVIDENCE_WORKFLOW_RUN_KIND,
                "id": "devtools-first-open-smoke-validate",
                "label": "devtools first-open smoke validate",
                "status": "failed",
                "command_line": "cargo run -p fretboard-dev -- diag campaign validate tools/diag-campaigns/devtools-first-open-smoke.json --json"
            }))
            .unwrap(),
        )
        .unwrap();

        let text =
            recent_evidence_resource_text(&repo_root).expect("recent evidence resource text");
        let report: RecentEvidenceReportV1 =
            serde_json::from_str(&text).expect("recent evidence resource json");

        assert_eq!(report.schema_version, 1);
        assert_eq!(report.workflow_runs_count, 1);
        assert_eq!(report.failing_count, 1);
        assert_eq!(
            report.first_failed.as_ref().map(|entry| entry.id.as_str()),
            Some("devtools-first-open-smoke-validate")
        );
        assert!(report.human_summary.contains("recent evidence counts:"));

        let _ = std::fs::remove_dir_all(&repo_root);
    }

    #[test]
    fn build_recent_evidence_report_prefers_latest_failed_result_across_lanes() {
        let repo_root = std::env::temp_dir().join(format!(
            "fret-devtools-mcp-recent-evidence-latest-failed-{}-{}",
            std::process::id(),
            unix_ms_now()
        ));
        let _ = std::fs::remove_dir_all(&repo_root);
        std::fs::create_dir_all(repo_root.join(RECENT_EVIDENCE_GATE_RUNS_DIR)).unwrap();
        std::fs::create_dir_all(repo_root.join(RECENT_EVIDENCE_WORKFLOW_RUNS_DIR)).unwrap();
        std::fs::create_dir_all(repo_root.join(RECENT_EVIDENCE_FOLLOWUPS_DIR)).unwrap();

        std::fs::write(
            repo_root
                .join(RECENT_EVIDENCE_GATE_RUNS_DIR)
                .join("100-old-gate.json"),
            serde_json::to_vec_pretty(&serde_json::json!({
                "schema_version": 1,
                "kind": RECENT_EVIDENCE_GATE_RUN_KIND,
                "id": "old-gate",
                "label": "old gate",
                "status": "failed",
                "command_line": "gate old"
            }))
            .unwrap(),
        )
        .unwrap();
        std::fs::write(
            repo_root
                .join(RECENT_EVIDENCE_WORKFLOW_RUNS_DIR)
                .join("300-workflow.json"),
            serde_json::to_vec_pretty(&serde_json::json!({
                "schema_version": 1,
                "kind": RECENT_EVIDENCE_WORKFLOW_RUN_KIND,
                "id": "workflow",
                "label": "workflow",
                "status": "failed",
                "command_line": "workflow failed",
                "started_unix_ms": 300,
                "finished_unix_ms": 900
            }))
            .unwrap(),
        )
        .unwrap();
        std::fs::write(
            repo_root
                .join(RECENT_EVIDENCE_FOLLOWUPS_DIR)
                .join("500-trace.json"),
            serde_json::to_vec_pretty(&serde_json::json!({
                "schema_version": 1,
                "kind": RECENT_EVIDENCE_FOLLOWUP_KIND,
                "id": "trace",
                "label": "trace",
                "status": "failed",
                "command_line": "trace failed",
                "bundle_dir": "target/fret-diag/latest",
                "started_unix_ms": 500,
                "finished_unix_ms": 600
            }))
            .unwrap(),
        )
        .unwrap();

        let report = build_recent_evidence_report(&repo_root, 8);

        assert_eq!(report.failing_count, 3);
        assert_eq!(
            report
                .first_failed
                .as_ref()
                .map(|entry| entry.kind.as_str()),
            Some("workflow")
        );
        assert_eq!(
            report.first_failed.as_ref().map(|entry| entry.id.as_str()),
            Some("workflow")
        );
        assert_eq!(
            report
                .first_failed
                .as_ref()
                .and_then(|entry| entry.finished_unix_ms),
            Some(900)
        );
        assert!(
            report
                .human_summary
                .contains("first failed evidence: workflow failed")
        );
        assert!(
            report
                .next_action
                .contains("inspect failed workflow evidence result JSON")
        );

        let _ = std::fs::remove_dir_all(&repo_root);
    }

    #[test]
    fn load_recent_evidence_entries_prefers_record_time_over_file_mtime() {
        let repo_root = std::env::temp_dir().join(format!(
            "fret-devtools-mcp-recent-evidence-record-time-{}-{}",
            std::process::id(),
            unix_ms_now()
        ));
        let _ = std::fs::remove_dir_all(&repo_root);
        let result_dir = repo_root.join(RECENT_EVIDENCE_WORKFLOW_RUNS_DIR);
        std::fs::create_dir_all(&result_dir).unwrap();
        let older_mtime = result_dir.join("100-record-newer.json");
        let newer_mtime = result_dir.join("200-record-older.json");

        std::fs::write(
            &older_mtime,
            serde_json::to_vec_pretty(&serde_json::json!({
                "schema_version": 1,
                "kind": RECENT_EVIDENCE_WORKFLOW_RUN_KIND,
                "id": "record-newer",
                "label": "record newer",
                "status": "failed",
                "command_line": "workflow record newer",
                "started_unix_ms": 100,
                "finished_unix_ms": 900
            }))
            .unwrap(),
        )
        .unwrap();
        std::thread::sleep(std::time::Duration::from_millis(5));
        std::fs::write(
            &newer_mtime,
            serde_json::to_vec_pretty(&serde_json::json!({
                "schema_version": 1,
                "kind": RECENT_EVIDENCE_WORKFLOW_RUN_KIND,
                "id": "record-older",
                "label": "record older",
                "status": "failed",
                "command_line": "workflow record older",
                "started_unix_ms": 500,
                "finished_unix_ms": 600
            }))
            .unwrap(),
        )
        .unwrap();

        let entries = load_recent_evidence_entries(
            &result_dir,
            RECENT_EVIDENCE_WORKFLOW_RUN_KIND,
            "workflow",
            8,
        );

        assert_eq!(entries.len(), 2);
        assert_eq!(entries[0].id, "record-newer");
        assert_eq!(entries[0].result_path, older_mtime.to_string_lossy());
        assert_eq!(entries[1].id, "record-older");

        let _ = std::fs::remove_dir_all(&repo_root);
    }

    #[test]
    fn mcp_server_instructions_point_to_first_open_resource() {
        let text = mcp_server_instructions();
        assert!(text.contains("fret-diag://first-open.md"));
        assert!(text.contains("Recent evidence: fret_diag_recent_evidence"));
        assert!(text.contains("Product workflow: imui-product-chain"));
        assert!(text.contains("python tools/diag_gate_imui_product_chain.py --only discovery"));
        assert!(text.contains("--only perf-docking"));
    }

    #[test]
    fn artifact_path_from_bundle_dumped_payload_resolves_regression_files() {
        let repo_root = Path::new("F:/repo");
        let payload = serde_json::json!({
            "out_dir": "target/fret-diag/campaigns/ui-gallery-pr",
            "dir": "runs/case-a",
        });

        let summary = regression_summary_path_from_bundle_dumped_payload(repo_root, &payload)
            .expect("summary path");
        let index = regression_index_path_from_bundle_dumped_payload(repo_root, &payload)
            .expect("index path");

        assert_eq!(
            summary,
            PathBuf::from("F:/repo")
                .join("target/fret-diag/campaigns/ui-gallery-pr")
                .join(RESOURCE_KIND_REGRESSION_SUMMARY_JSON)
        );
        assert_eq!(
            index,
            PathBuf::from("F:/repo")
                .join("target/fret-diag/campaigns/ui-gallery-pr")
                .join(RESOURCE_KIND_REGRESSION_INDEX_JSON)
        );
    }

    #[test]
    fn build_regression_dashboard_result_limits_top_rows_and_builds_human_summary() {
        let dir = std::env::temp_dir().join(format!(
            "fret-devtools-mcp-regression-dashboard-{}-{}",
            std::process::id(),
            unix_ms_now()
        ));
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).unwrap();
        let index_path = dir.join("regression.index.json");
        let summary_path = dir.join("regression.summary.json");
        let payload = serde_json::json!({
            "kind": "diag_regression_index",
            "out_dir": "target/fret-diag/campaigns/ui-gallery-pr",
            "summaries": [
                { "items_total": 2 },
                { "items_total": 3 }
            ],
            "counters": {
                "by_status": { "passed": 3, "failed_deterministic": 2, "skipped_policy": 1 },
                "by_lane": { "smoke": 1, "perf": 1 },
                "by_tool": { "suite": 1, "perf": 1 }
            },
            "top_reason_codes": [
                { "reason_code": "pixel_diff", "count": 4 },
                { "reason_code": "perf_budget_exceeded", "count": 2 }
            ],
            "failing_summaries": [
                { "path": "a/regression.summary.json", "lane": "smoke", "failures": 2, "items_total": 5 },
                { "path": "b/regression.summary.json", "lane": "perf", "failures": 1, "items_total": 3 }
            ]
        });
        let summary_payload = serde_json::json!({
            "schema_version": 1,
            "kind": "diag_regression_summary",
            "campaign": { "name": "ui-gallery-pr", "lane": "smoke" },
            "run": { "run_id": "run-1", "created_unix_ms": 1, "tool": "suite" },
            "totals": { "items_total": 1, "passed": 0, "failed_deterministic": 0, "failed_flaky": 0, "failed_tooling": 0, "failed_timeout": 0, "skipped_policy": 1, "quarantined": 0 },
            "items": [
                {
                    "item_id": "capability-check",
                    "kind": "script",
                    "name": "capability-check",
                    "status": "skipped_policy",
                    "lane": "smoke",
                    "reason_code": "capability.missing",
                    "evidence": {
                        "bundle_dir": "target/fret-diag/campaigns/ui-gallery/run-a",
                        "triage_artifact": "target/fret-diag/campaigns/ui-gallery/run-a/triage.json",
                        "script_result": "target/fret-diag/campaigns/ui-gallery/run-a/script.result.json",
                        "share_artifact": "target/fret-diag/campaigns/ui-gallery/share/capability-check.ai.zip",
                        "perf_summary_json": "target/fret-diag/campaigns/ui-gallery/layout.perf.summary.v1.json",
                        "compare_json": "target/fret-diag/campaigns/ui-gallery/check.perf_thresholds.json",
                        "extra": {
                            "capability_source": {
                                "kind": "filesystem",
                                "path": "target/fret-diag/capabilities.json",
                                "label": "filesystem:target/fret-diag/capabilities.json",
                                "transport": "filesystem",
                                "session_id": null
                            },
                            "capabilities_check_path": "target/fret-diag/campaigns/ui-gallery/check.capabilities.json"
                        }
                    }
                }
            ]
        });
        std::fs::write(
            &summary_path,
            serde_json::to_vec_pretty(&summary_payload).unwrap(),
        )
        .unwrap();

        let result = build_regression_dashboard_result(
            "target/fret-diag/campaigns/ui-gallery-pr".to_string(),
            &index_path,
            &payload,
            1,
            false,
            None,
        );

        assert_eq!(result.summaries_total, 2);
        assert_eq!(result.items_total, 5);
        assert_eq!(result.top_reason_codes.len(), 1);
        assert_eq!(result.failing_summaries.len(), 1);
        assert!(
            result
                .status_counters
                .iter()
                .any(|entry| entry.key == "skipped_policy" && entry.count == 1)
        );
        assert!(result.human_summary.contains("normalized status counters:"));
        assert!(result.human_summary.contains("skipped_policy: 1"));
        assert!(result.human_summary.contains("non-passing summaries:"));
        assert!(result.human_summary.contains("top reason codes:"));
        assert!(result.human_summary.contains("pixel_diff: 4"));
        assert_eq!(
            result.bundle_dirs,
            vec!["target/fret-diag/campaigns/ui-gallery/run-a".to_string()]
        );
        assert_eq!(
            result.capability_sources,
            vec!["target/fret-diag/capabilities.json".to_string()]
        );
        assert_eq!(
            result.capabilities_check_paths,
            vec!["target/fret-diag/campaigns/ui-gallery/check.capabilities.json".to_string()]
        );
        assert!(result.perf_evidence_lines.iter().any(|line| line.contains(
            "capability-check [skipped_policy] perf_summary_json: target/fret-diag/campaigns/ui-gallery/layout.perf.summary.v1.json"
        )));
        assert!(result.first_open_evidence_lines.iter().any(|line| line.contains(
            "capability-check [skipped_policy] triage_artifact: target/fret-diag/campaigns/ui-gallery/run-a/triage.json"
        )));
        assert_eq!(
            result.share_artifacts,
            vec!["target/fret-diag/campaigns/ui-gallery/share/capability-check.ai.zip".to_string()]
        );
        assert!(result.followup_command_lines.iter().any(|line| line.contains(
            "diag stats: cargo run -p fretboard-dev -- diag stats target/fret-diag/campaigns/ui-gallery/run-a --json"
        )));
        assert!(
            result
                .runnable_followup_command_lines
                .iter()
                .any(|line| line.contains("diag stats: cargo run -p fretboard-dev -- diag stats"))
        );
        assert!(
            result
                .runnable_followup_command_lines
                .iter()
                .any(|line| line.contains("trace: cargo run -p fretboard-dev -- diag trace"))
        );
        assert!(result.runnable_followup_commands.iter().any(|command| {
            command.id == "trace"
                && command.diag_args
                    == vec![
                        "trace".to_string(),
                        "target/fret-diag/campaigns/ui-gallery/run-a".to_string(),
                        "--json".to_string(),
                    ]
                && !command.requires_baseline
        }));
        assert!(
            result
                .manual_followup_commands
                .iter()
                .any(|command| command.id == "visual-compare" && command.requires_baseline)
        );
        assert!(result
            .manual_followup_command_lines
            .iter()
            .any(|line| line.contains("visual compare: cargo run -p fretboard-dev -- diag compare <baseline-bundle-or-dir>")));
        assert!(result.human_summary.contains("bundle dirs:"));
        assert!(result.human_summary.contains("capability sources:"));
        assert!(result.human_summary.contains("capability checks:"));
        assert!(result.human_summary.contains("perf evidence:"));
        assert!(result.human_summary.contains("first-open evidence:"));
        assert!(result.human_summary.contains("share artifacts:"));
        assert!(result.human_summary.contains("follow-up commands:"));
        assert!(
            result
                .human_summary
                .contains("runnable follow-up commands:")
        );
        assert!(
            result
                .human_summary
                .contains("manual compare follow-up commands:")
        );

        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn collect_regression_dashboard_evidence_falls_back_to_capability_source_label() {
        let dir = std::env::temp_dir().join(format!(
            "fret-devtools-mcp-regression-evidence-{}-{}",
            std::process::id(),
            unix_ms_now()
        ));
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).unwrap();
        let summary_path = dir.join("regression.summary.json");
        let summary_payload = serde_json::json!({
            "schema_version": 1,
            "kind": "diag_regression_summary",
            "campaign": { "name": "ui-gallery-pr", "lane": "smoke" },
            "run": { "run_id": "run-1", "created_unix_ms": 1, "tool": "suite" },
            "totals": { "items_total": 1, "passed": 0, "failed_deterministic": 0, "failed_flaky": 0, "failed_tooling": 0, "failed_timeout": 0, "skipped_policy": 1, "quarantined": 0 },
            "items": [
                {
                    "item_id": "capability-check",
                    "kind": "script",
                    "name": "capability-check",
                    "status": "skipped_policy",
                    "lane": "smoke",
                    "reason_code": "capability.missing",
                    "source": {
                        "metadata": {
                            "capability_source": {
                                "kind": "transport_session",
                                "path": null,
                                "label": "devtools_ws:session-123",
                                "transport": "devtools_ws",
                                "session_id": "session-123"
                            }
                        }
                    }
                }
            ]
        });
        std::fs::write(
            &summary_path,
            serde_json::to_vec_pretty(&summary_payload).unwrap(),
        )
        .unwrap();

        let evidence = collect_regression_dashboard_evidence(&summary_path);

        assert_eq!(
            evidence.capability_sources,
            vec!["devtools_ws:session-123".to_string()]
        );
        assert!(evidence.capabilities_check_paths.is_empty());

        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn session_resource_uris_includes_selected_alias_when_session_matches() {
        let uris = session_resource_uris(
            "session-a",
            Some("session-a"),
            &[
                RESOURCE_KIND_REGRESSION_SUMMARY_JSON,
                RESOURCE_KIND_REGRESSION_INDEX_JSON,
            ],
        );

        assert!(
            uris.contains(&"fret-diag://sessions/session-a/regression.summary.json".to_string())
        );
        assert!(uris.contains(&"fret-diag://sessions/session-a/regression.index.json".to_string()));
        assert!(uris.contains(&"fret-diag://selected/regression.summary.json".to_string()));
        assert!(uris.contains(&"fret-diag://selected/regression.index.json".to_string()));
    }
}
