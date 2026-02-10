use std::collections::HashSet;
use std::net::{IpAddr, Ipv4Addr, SocketAddr};
use std::path::PathBuf;
use std::sync::Arc;

use fret_app::{App, CommandId, Effect};
use fret_bootstrap::BootstrapBuilder;
use fret_bootstrap::ui_app_driver::{UiAppDriver, ViewElements};
use fret_core::{AppWindowId, Px, UiServices};
use fret_diag::devtools::DevtoolsOps;
use fret_diag::transport::{
    ClientKindV1, DevtoolsWsClientConfig, DiagTransportKind, FsDiagTransportConfig,
    ToolingDiagClient, WsDiagTransportConfig,
};
use fret_diag_protocol::{
    DevtoolsSessionDescriptorV1, UiActionScriptV1, UiActionScriptV2, UiScriptStageV1,
};
use fret_diag_ws::server::{DevtoolsWsServer, DevtoolsWsServerConfig};
use fret_runtime::Model;
use fret_ui::element::{AnyElement, LayoutStyle, Length, VirtualListOptions};
use fret_ui::elements::ContinuousFrames;
use fret_ui::scroll::ScrollStrategy;
use fret_ui::scroll::VirtualListScrollHandle;
use fret_ui::{ElementContext, Invalidation, Theme};
use fret_ui_shadcn as shadcn;

mod pack;
mod script_studio;
mod semantics;
mod ws;

const CMD_COPY_WS_URL: &str = "fret.devtools.copy_ws_url";
const CMD_COPY_TOKEN: &str = "fret.devtools.copy_token";
const CMD_INSPECT_ENABLE: &str = "fret.devtools.inspect_enable";
const CMD_INSPECT_DISABLE: &str = "fret.devtools.inspect_disable";
const CMD_PICK_ARM: &str = "fret.devtools.pick_arm";
const CMD_BUNDLE_DUMP: &str = "fret.devtools.bundle_dump";
const CMD_SCREENSHOT_REQUEST: &str = "fret.devtools.screenshot_request";
const CMD_SCRIPT_PUSH: &str = "fret.devtools.script_push";
const CMD_SCRIPT_RUN: &str = "fret.devtools.script_run";
const CMD_SCRIPT_RUN_AND_PACK: &str = "fret.devtools.script_run_and_pack";
const CMD_SCRIPTS_REFRESH: &str = "fret.devtools.scripts.refresh";
const CMD_SCRIPT_FORK: &str = "fret.devtools.script.fork";
const CMD_SCRIPT_SAVE: &str = "fret.devtools.script.save";
const CMD_SCRIPT_APPLY_PICK: &str = "fret.devtools.script.apply_pick";
const CMD_PACK_LAST_BUNDLE: &str = "fret.devtools.pack_last_bundle";
const CMD_COPY_PACK_PATH: &str = "fret.devtools.copy_pack_path";
const CMD_OPEN_VIEWER_URL: &str = "fret.devtools.open_viewer_url";

#[derive(Clone)]
struct DevtoolsConfig {
    transport: DiagTransportKind,
    fs_out_dir: Arc<str>,
    ws_port: u16,
    ws_url: Arc<str>,
    token: Arc<str>,
}

struct State {
    cfg: DevtoolsConfig,

    panel_fractions: Model<Vec<f32>>,
    left_tab: Model<Option<Arc<str>>>,
    details_tab: Model<Option<Arc<str>>>,
    sessions: Model<Vec<DevtoolsSessionDescriptorV1>>,
    selected_session_id: Model<Option<Arc<str>>>,
    selected_session_open: Model<bool>,
    inspect_consume_clicks: Model<bool>,

    script_paths: script_studio::ScriptPaths,
    script_library: Model<Vec<script_studio::ScriptItem>>,
    loaded_script_origin: Model<Option<script_studio::ScriptOrigin>>,
    loaded_script_path: Model<Option<Arc<str>>>,
    script_apply_pointer: Model<String>,
    script_text: Model<String>,
    script_studio_helper_tab: Model<Option<Arc<str>>>,
    script_step_insert_index: Model<String>,
    script_selector_kind: Model<Option<Arc<str>>>,
    script_selector_kind_open: Model<bool>,
    script_selector_test_id: Model<String>,
    script_selector_role: Model<String>,
    script_selector_name: Model<String>,
    script_selector_ancestors: Model<String>,
    script_selector_node_id: Model<String>,
    script_selector_element_id: Model<String>,
    script_predicate_kind: Model<Option<Arc<str>>>,
    script_predicate_kind_open: Model<bool>,
    script_predicate_other_selector_json: Model<String>,
    script_predicate_role: Model<String>,
    script_predicate_checked: Model<bool>,
    script_predicate_padding_px: Model<String>,
    script_predicate_eps_px: Model<String>,
    script_predicate_min_w_px: Model<String>,
    script_predicate_min_h_px: Model<String>,
    script_predicate_barrier_root: Model<Option<Arc<str>>>,
    script_predicate_barrier_root_open: Model<bool>,
    script_predicate_focus_barrier_root: Model<Option<Arc<str>>>,
    script_predicate_focus_barrier_root_open: Model<bool>,
    script_predicate_require_equal: Model<Option<Arc<str>>>,
    script_predicate_require_equal_open: Model<bool>,

    script_last_stage: Model<Option<UiScriptStageV1>>,
    script_last_step_index: Model<Option<u32>>,
    script_last_reason: Model<Option<Arc<str>>>,
    script_last_bundle_dir: Model<Option<Arc<str>>>,
    script_pack_after_run: Model<bool>,

    target_out_dir: Model<Option<Arc<str>>>,
    last_bundle_dir_abs: Model<Option<Arc<str>>>,
    last_bundle_dump_exported_unix_ms: Model<Option<u64>>,
    last_bundle_dump_bundle_json: Model<Option<Arc<str>>>,
    last_pack_path: Model<Option<Arc<str>>>,
    pack_in_flight: Model<bool>,
    pack_last_error: Model<Option<Arc<str>>>,
    viewer_url: Model<String>,

    last_pick_json: Model<String>,
    last_inspect_hover_json: Model<String>,
    last_inspect_focus_json: Model<String>,
    last_script_result_json: Model<String>,
    last_bundle_json: Model<String>,
    last_screenshot_json: Model<String>,
    log_lines: Model<Vec<Arc<str>>>,

    semantics_cache: Model<Option<Arc<semantics::SemanticsIndex>>>,
    semantics_source_hash: Model<Option<u64>>,
    semantics_error: Model<Option<Arc<str>>>,
    semantics_search: Model<String>,
    semantics_expanded: Model<HashSet<u64>>,
    semantics_selected_id: Model<Option<u64>>,
    semantics_selected_node_json: Model<String>,
    semantics_selected_node_live_json: Model<String>,
    semantics_selected_node_live_status: Model<Option<Arc<str>>>,
    semantics_selected_node_live_updated_unix_ms: Model<Option<u64>>,
    semantics_selected_node_live_children: Model<Vec<u64>>,
    semantics_live_enabled: Model<bool>,
    semantics_live_force_nonce: Model<u64>,

    devtools: DevtoolsOps,
    applied_session_id: Option<Arc<str>>,

    live_semantics_last_target: Option<(u64, u64)>,
    live_semantics_last_sent_unix_ms: Option<u64>,
    live_semantics_last_force_nonce: u64,

    pack_tx: std::sync::mpsc::Sender<pack::PackJobResult>,
    pack_rx: std::sync::mpsc::Receiver<pack::PackJobResult>,
}

fn main() -> anyhow::Result<()> {
    let transport =
        env_transport_kind("FRET_DEVTOOLS_TRANSPORT").unwrap_or(DiagTransportKind::WebSocket);
    let fs_out_dir =
        std::env::var("FRET_DIAG_DIR").unwrap_or_else(|_| "target/fret-diag".to_string());

    let port = env_u16("FRET_DEVTOOLS_WS_PORT").unwrap_or(7331);
    let token =
        std::env::var("FRET_DEVTOOLS_TOKEN").unwrap_or_else(|_| uuid::Uuid::new_v4().to_string());
    let bind = SocketAddr::new(IpAddr::V4(Ipv4Addr::LOCALHOST), port);

    eprintln!("fret-devtools: bind={bind} token={token}");
    eprintln!("fret-devtools: url=ws://127.0.0.1:{port}/?fret_devtools_token={token}");
    eprintln!("fret-devtools: transport={transport:?} fs_out_dir={fs_out_dir}");

    std::thread::spawn({
        let token = token.clone();
        move || {
            let server = DevtoolsWsServer::new(DevtoolsWsServerConfig { bind, token });
            let _ = server.run();
        }
    });

    let ws_url = Arc::<str>::from(format!("ws://127.0.0.1:{port}/"));
    let token = Arc::<str>::from(token);

    let mut app = App::new();
    app.set_global(DevtoolsConfig {
        transport,
        fs_out_dir: Arc::<str>::from(fs_out_dir),
        ws_port: port,
        ws_url: ws_url.clone(),
        token: token.clone(),
    });

    let driver = UiAppDriver::new("fret-devtools", init_window, view)
        .on_command(on_command)
        .into_fn_driver();

    BootstrapBuilder::new(app, driver)
        .with_default_config_files()?
        .with_lucide_icons()
        .run()
        .map_err(anyhow::Error::from)
}

fn init_window(app: &mut App, _window: AppWindowId) -> State {
    let cfg = app
        .global::<DevtoolsConfig>()
        .cloned()
        .expect("DevtoolsConfig must be set before starting the app");

    let panel_fractions = app.models_mut().insert(vec![0.25f32, 0.45f32, 0.30f32]);
    let left_tab = app.models_mut().insert(Some(Arc::<str>::from("semantics")));
    let details_tab = app.models_mut().insert(Some(Arc::<str>::from("pick")));
    let sessions = app
        .models_mut()
        .insert(Vec::<DevtoolsSessionDescriptorV1>::new());
    let selected_session_id = app.models_mut().insert(None::<Arc<str>>);
    let selected_session_open = app.models_mut().insert(false);
    let inspect_consume_clicks = app.models_mut().insert(false);

    let repo_root = script_studio::repo_root_from_manifest_dir()
        .unwrap_or_else(|| std::env::current_dir().unwrap_or_else(|_| PathBuf::from(".")));
    let script_paths = script_studio::ScriptPaths::from_repo_root(repo_root);
    let script_library = app
        .models_mut()
        .insert(Vec::<script_studio::ScriptItem>::new());
    let loaded_script_origin = app.models_mut().insert(None::<script_studio::ScriptOrigin>);
    let loaded_script_path = app.models_mut().insert(None::<Arc<str>>);
    let script_apply_pointer = app.models_mut().insert("/steps/0/target".to_string());
    let script_studio_helper_tab = app.models_mut().insert(Some(Arc::<str>::from("steps")));
    let script_step_insert_index = app.models_mut().insert(String::new());
    let script_selector_kind = app.models_mut().insert(Some(Arc::<str>::from("test_id")));
    let script_selector_kind_open = app.models_mut().insert(false);
    let script_selector_test_id = app.models_mut().insert("TODO".to_string());
    let script_selector_role = app.models_mut().insert("button".to_string());
    let script_selector_name = app.models_mut().insert("TODO".to_string());
    let script_selector_ancestors = app.models_mut().insert(String::new());
    let script_selector_node_id = app.models_mut().insert("0".to_string());
    let script_selector_element_id = app.models_mut().insert("0".to_string());
    let script_predicate_kind = app.models_mut().insert(Some(Arc::<str>::from("exists")));
    let script_predicate_kind_open = app.models_mut().insert(false);
    let script_predicate_other_selector_json = app.models_mut().insert(String::new());
    let script_predicate_role = app.models_mut().insert("button".to_string());
    let script_predicate_checked = app.models_mut().insert(false);
    let script_predicate_padding_px = app.models_mut().insert("0".to_string());
    let script_predicate_eps_px = app.models_mut().insert("0".to_string());
    let script_predicate_min_w_px = app.models_mut().insert("0".to_string());
    let script_predicate_min_h_px = app.models_mut().insert("0".to_string());
    let script_predicate_barrier_root = app.models_mut().insert(Some(Arc::<str>::from("any")));
    let script_predicate_barrier_root_open = app.models_mut().insert(false);
    let script_predicate_focus_barrier_root =
        app.models_mut().insert(Some(Arc::<str>::from("any")));
    let script_predicate_focus_barrier_root_open = app.models_mut().insert(false);
    let script_predicate_require_equal = app.models_mut().insert(Some(Arc::<str>::from("unset")));
    let script_predicate_require_equal_open = app.models_mut().insert(false);

    let script_text = app.models_mut().insert(String::new());
    let script_last_stage = app.models_mut().insert(None::<UiScriptStageV1>);
    let script_last_step_index = app.models_mut().insert(None::<u32>);
    let script_last_reason = app.models_mut().insert(None::<Arc<str>>);
    let script_last_bundle_dir = app.models_mut().insert(None::<Arc<str>>);
    let script_pack_after_run = app.models_mut().insert(false);

    let target_out_dir = match cfg.transport {
        DiagTransportKind::FileSystem => app.models_mut().insert(Some(cfg.fs_out_dir.clone())),
        DiagTransportKind::WebSocket => app.models_mut().insert(None::<Arc<str>>),
    };
    let last_bundle_dir_abs = app.models_mut().insert(None::<Arc<str>>);
    let last_bundle_dump_exported_unix_ms = app.models_mut().insert(None::<u64>);
    let last_bundle_dump_bundle_json = app.models_mut().insert(None::<Arc<str>>);
    let last_pack_path = app.models_mut().insert(None::<Arc<str>>);
    let pack_in_flight = app.models_mut().insert(false);
    let pack_last_error = app.models_mut().insert(None::<Arc<str>>);
    let viewer_url = app.models_mut().insert("http://localhost:5173".to_string());
    let last_pick_json = app.models_mut().insert(String::new());
    let last_inspect_hover_json = app.models_mut().insert(String::new());
    let last_inspect_focus_json = app.models_mut().insert(String::new());
    let last_script_result_json = app.models_mut().insert(String::new());
    let last_bundle_json = app.models_mut().insert(String::new());
    let last_screenshot_json = app.models_mut().insert(String::new());
    let log_lines = match cfg.transport {
        DiagTransportKind::FileSystem => app.models_mut().insert(vec![Arc::<str>::from(format!(
            "filesystem transport: polling FRET_DIAG_DIR={}",
            cfg.fs_out_dir
        ))]),
        DiagTransportKind::WebSocket => app.models_mut().insert(Vec::<Arc<str>>::new()),
    };

    let semantics_cache = app
        .models_mut()
        .insert(None::<Arc<semantics::SemanticsIndex>>);
    let semantics_source_hash = app.models_mut().insert(None::<u64>);
    let semantics_error = app.models_mut().insert(None::<Arc<str>>);
    let semantics_search = app.models_mut().insert(String::new());
    let semantics_expanded = app.models_mut().insert(HashSet::<u64>::new());
    let semantics_selected_id = app.models_mut().insert(None::<u64>);
    let semantics_selected_node_json = app.models_mut().insert(String::new());
    let semantics_selected_node_live_json = app.models_mut().insert(String::new());
    let semantics_selected_node_live_status = app.models_mut().insert(None::<Arc<str>>);
    let semantics_selected_node_live_updated_unix_ms = app.models_mut().insert(None::<u64>);
    let semantics_selected_node_live_children = app.models_mut().insert(Vec::<u64>::new());
    let semantics_live_enabled = app.models_mut().insert(true);
    let semantics_live_force_nonce = app.models_mut().insert(0u64);

    let client = match cfg.transport {
        DiagTransportKind::WebSocket => {
            let mut client_cfg = DevtoolsWsClientConfig::with_defaults(
                cfg.ws_url.to_string(),
                cfg.token.to_string(),
            );
            client_cfg.client_kind = ClientKindV1::Tooling;
            client_cfg.capabilities = vec![
                "inspect".to_string(),
                "pick".to_string(),
                "scripts".to_string(),
                "bundles".to_string(),
            ];
            ToolingDiagClient::connect_ws(WsDiagTransportConfig::native(client_cfg))
                .expect("devtools ws client connect must succeed")
        }
        DiagTransportKind::FileSystem => {
            let fs_cfg =
                FsDiagTransportConfig::from_out_dir(PathBuf::from(cfg.fs_out_dir.as_ref()));
            ToolingDiagClient::connect_fs(fs_cfg).expect("devtools fs client connect must succeed")
        }
    };
    let devtools = DevtoolsOps::new(client);

    let (pack_tx, pack_rx) = pack::new_pack_channel();

    let mut st = State {
        cfg,
        panel_fractions,
        left_tab,
        details_tab,
        sessions,
        selected_session_id,
        selected_session_open,
        inspect_consume_clicks,
        script_paths,
        script_library,
        loaded_script_origin,
        loaded_script_path,
        script_apply_pointer,
        script_text,
        script_studio_helper_tab,
        script_step_insert_index,
        script_selector_kind,
        script_selector_kind_open,
        script_selector_test_id,
        script_selector_role,
        script_selector_name,
        script_selector_ancestors,
        script_selector_node_id,
        script_selector_element_id,
        script_predicate_kind,
        script_predicate_kind_open,
        script_predicate_other_selector_json,
        script_predicate_role,
        script_predicate_checked,
        script_predicate_padding_px,
        script_predicate_eps_px,
        script_predicate_min_w_px,
        script_predicate_min_h_px,
        script_predicate_barrier_root,
        script_predicate_barrier_root_open,
        script_predicate_focus_barrier_root,
        script_predicate_focus_barrier_root_open,
        script_predicate_require_equal,
        script_predicate_require_equal_open,
        script_last_stage,
        script_last_step_index,
        script_last_reason,
        script_last_bundle_dir,
        script_pack_after_run,
        target_out_dir,
        last_bundle_dir_abs,
        last_bundle_dump_exported_unix_ms,
        last_bundle_dump_bundle_json,
        last_pack_path,
        pack_in_flight,
        pack_last_error,
        viewer_url,
        last_pick_json,
        last_inspect_hover_json,
        last_inspect_focus_json,
        last_script_result_json,
        last_bundle_json,
        last_screenshot_json,
        log_lines,
        semantics_cache,
        semantics_source_hash,
        semantics_error,
        semantics_search,
        semantics_expanded,
        semantics_selected_id,
        semantics_selected_node_json,
        semantics_selected_node_live_json,
        semantics_selected_node_live_status,
        semantics_selected_node_live_updated_unix_ms,
        semantics_selected_node_live_children,
        semantics_live_enabled,
        semantics_live_force_nonce,
        devtools,
        applied_session_id: None,
        live_semantics_last_target: None,
        live_semantics_last_sent_unix_ms: None,
        live_semantics_last_force_nonce: 0,
        pack_tx,
        pack_rx,
    };

    refresh_script_library(app, &mut st);
    st
}

fn view(cx: &mut ElementContext<'_, App>, st: &mut State) -> ViewElements {
    pack::poll_pack_jobs(cx.app, st);
    ws::drain_ws_messages(cx.app, st);
    ws::sync_selected_session_to_client(cx.app, st);
    semantics::refresh_semantics_cache_if_needed(cx.app, st);
    ws::maybe_request_semantics_node_details(cx.app, st);

    let mut needs_frames = false;
    cx.with_state(
        || None::<ContinuousFrames>,
        |lease: &mut Option<ContinuousFrames>| {
            if lease.is_none() {
                needs_frames = true;
            }
        },
    );
    if needs_frames {
        let lease = cx.begin_continuous_frames();
        cx.with_state(
            || None::<ContinuousFrames>,
            |slot: &mut Option<ContinuousFrames>| {
                *slot = Some(lease);
            },
        );
    }

    cx.observe_model(&st.panel_fractions, Invalidation::Layout);
    cx.observe_model(&st.left_tab, Invalidation::Paint);
    cx.observe_model(&st.details_tab, Invalidation::Paint);
    cx.observe_model(&st.sessions, Invalidation::Paint);
    cx.observe_model(&st.selected_session_id, Invalidation::Paint);
    cx.observe_model(&st.selected_session_open, Invalidation::Paint);
    cx.observe_model(&st.inspect_consume_clicks, Invalidation::Paint);
    cx.observe_model(&st.script_library, Invalidation::Paint);
    cx.observe_model(&st.loaded_script_origin, Invalidation::Paint);
    cx.observe_model(&st.loaded_script_path, Invalidation::Paint);
    cx.observe_model(&st.script_apply_pointer, Invalidation::Paint);
    cx.observe_model(&st.script_text, Invalidation::Paint);
    cx.observe_model(&st.script_studio_helper_tab, Invalidation::Paint);
    cx.observe_model(&st.script_step_insert_index, Invalidation::Paint);
    cx.observe_model(&st.script_selector_kind, Invalidation::Paint);
    cx.observe_model(&st.script_selector_kind_open, Invalidation::Paint);
    cx.observe_model(&st.script_selector_test_id, Invalidation::Paint);
    cx.observe_model(&st.script_selector_role, Invalidation::Paint);
    cx.observe_model(&st.script_selector_name, Invalidation::Paint);
    cx.observe_model(&st.script_selector_ancestors, Invalidation::Paint);
    cx.observe_model(&st.script_selector_node_id, Invalidation::Paint);
    cx.observe_model(&st.script_selector_element_id, Invalidation::Paint);
    cx.observe_model(&st.script_predicate_kind, Invalidation::Paint);
    cx.observe_model(&st.script_predicate_kind_open, Invalidation::Paint);
    cx.observe_model(
        &st.script_predicate_other_selector_json,
        Invalidation::Paint,
    );
    cx.observe_model(&st.script_predicate_role, Invalidation::Paint);
    cx.observe_model(&st.script_predicate_checked, Invalidation::Paint);
    cx.observe_model(&st.script_predicate_padding_px, Invalidation::Paint);
    cx.observe_model(&st.script_predicate_eps_px, Invalidation::Paint);
    cx.observe_model(&st.script_predicate_min_w_px, Invalidation::Paint);
    cx.observe_model(&st.script_predicate_min_h_px, Invalidation::Paint);
    cx.observe_model(&st.script_predicate_barrier_root, Invalidation::Paint);
    cx.observe_model(&st.script_predicate_barrier_root_open, Invalidation::Paint);
    cx.observe_model(&st.script_predicate_focus_barrier_root, Invalidation::Paint);
    cx.observe_model(
        &st.script_predicate_focus_barrier_root_open,
        Invalidation::Paint,
    );
    cx.observe_model(&st.script_predicate_require_equal, Invalidation::Paint);
    cx.observe_model(&st.script_predicate_require_equal_open, Invalidation::Paint);
    cx.observe_model(&st.script_last_stage, Invalidation::Paint);
    cx.observe_model(&st.script_last_step_index, Invalidation::Paint);
    cx.observe_model(&st.script_last_reason, Invalidation::Paint);
    cx.observe_model(&st.script_last_bundle_dir, Invalidation::Paint);
    cx.observe_model(&st.script_pack_after_run, Invalidation::Paint);
    cx.observe_model(&st.target_out_dir, Invalidation::Paint);
    cx.observe_model(&st.last_bundle_dir_abs, Invalidation::Paint);
    cx.observe_model(&st.last_bundle_dump_exported_unix_ms, Invalidation::Paint);
    cx.observe_model(&st.last_bundle_dump_bundle_json, Invalidation::Paint);
    cx.observe_model(&st.last_pack_path, Invalidation::Paint);
    cx.observe_model(&st.pack_in_flight, Invalidation::Paint);
    cx.observe_model(&st.pack_last_error, Invalidation::Paint);
    cx.observe_model(&st.viewer_url, Invalidation::Paint);
    cx.observe_model(&st.last_pick_json, Invalidation::Paint);
    cx.observe_model(&st.last_inspect_hover_json, Invalidation::Paint);
    cx.observe_model(&st.last_inspect_focus_json, Invalidation::Paint);
    cx.observe_model(&st.last_script_result_json, Invalidation::Paint);
    cx.observe_model(&st.last_bundle_json, Invalidation::Paint);
    cx.observe_model(&st.last_screenshot_json, Invalidation::Paint);
    cx.observe_model(&st.log_lines, Invalidation::Paint);
    cx.observe_model(&st.semantics_cache, Invalidation::Paint);
    cx.observe_model(&st.semantics_error, Invalidation::Paint);
    cx.observe_model(&st.semantics_search, Invalidation::Paint);
    cx.observe_model(&st.semantics_expanded, Invalidation::Paint);
    cx.observe_model(&st.semantics_selected_id, Invalidation::Paint);
    cx.observe_model(&st.semantics_selected_node_json, Invalidation::Paint);
    cx.observe_model(&st.semantics_selected_node_live_json, Invalidation::Paint);
    cx.observe_model(&st.semantics_selected_node_live_status, Invalidation::Paint);
    cx.observe_model(
        &st.semantics_selected_node_live_updated_unix_ms,
        Invalidation::Paint,
    );
    cx.observe_model(
        &st.semantics_selected_node_live_children,
        Invalidation::Paint,
    );
    cx.observe_model(&st.semantics_live_enabled, Invalidation::Paint);
    cx.observe_model(&st.semantics_live_force_nonce, Invalidation::Paint);

    let theme = Theme::global(&*cx.app).clone();

    let header = header_bar(cx, &theme, st);
    let body = resizable_body(cx, &theme, st);

    let wrap = fret_ui_kit::declarative::style::container_props(
        &theme,
        fret_ui_kit::ChromeRefinement::default()
            .bg(fret_ui_kit::ColorRef::Color(
                theme.color_required("background"),
            ))
            .p(fret_ui_kit::Space::N2),
        fret_ui_kit::LayoutRefinement::default().w_full().h_full(),
    );

    vec![cx.container(wrap, |_cx| [header, body])].into()
}

fn header_bar(cx: &mut ElementContext<'_, App>, theme: &Theme, st: &State) -> AnyElement {
    let ws_url_with_token = format!(
        "{}?fret_devtools_token={}",
        st.cfg.ws_url.as_ref(),
        st.cfg.token.as_ref()
    );
    let title = cx.text("Fret DevTools (WS)");
    let subtitle = cx.text(format!(
        "url={}  token={}  port={}",
        ws_url_with_token, st.cfg.token, st.cfg.ws_port
    ));

    let actions = fret_ui_kit::declarative::stack::hstack(
        cx,
        fret_ui_kit::declarative::stack::HStackProps::default()
            .gap_x(fret_ui_kit::Space::N2)
            .layout(fret_ui_kit::LayoutRefinement::default().w_full())
            .items_center()
            .justify_between(),
        |cx| {
            let has_session = cx
                .app
                .models()
                .read(&st.selected_session_id, |v| v.is_some())
                .unwrap_or(false);

            let session_items = cx
                .app
                .models()
                .read(&st.sessions, |sessions| {
                    sessions
                        .iter()
                        .map(|s| {
                            let label = if s.client_version.trim().is_empty() {
                                format!("{} ({})", s.session_id, s.client_kind)
                            } else {
                                format!("{} ({} {})", s.session_id, s.client_kind, s.client_version)
                            };
                            shadcn::SelectItem::new(s.session_id.clone(), label)
                        })
                        .collect::<Vec<_>>()
                })
                .unwrap_or_default();

            let session_select = shadcn::Select::new(
                st.selected_session_id.clone(),
                st.selected_session_open.clone(),
            )
            .placeholder("Session")
            .items(session_items)
            .refine_layout(fret_ui_kit::LayoutRefinement::default().w_px(Px(220.0)))
            .into_element(cx);

            let left = fret_ui_kit::declarative::stack::hstack(
                cx,
                fret_ui_kit::declarative::stack::HStackProps::default()
                    .gap_x(fret_ui_kit::Space::N2)
                    .items_center(),
                |_cx| [title, subtitle],
            );

            let right = fret_ui_kit::declarative::stack::hstack(
                cx,
                fret_ui_kit::declarative::stack::HStackProps::default()
                    .gap_x(fret_ui_kit::Space::N2)
                    .items_center(),
                |cx| {
                    [
                        session_select,
                        shadcn::Button::new("Copy WS URL")
                            .variant(shadcn::ButtonVariant::Secondary)
                            .size(shadcn::ButtonSize::Sm)
                            .on_click(CMD_COPY_WS_URL)
                            .into_element(cx),
                        shadcn::Button::new("Copy Token")
                            .variant(shadcn::ButtonVariant::Secondary)
                            .size(shadcn::ButtonSize::Sm)
                            .on_click(CMD_COPY_TOKEN)
                            .into_element(cx),
                        shadcn::Button::new("Inspect On")
                            .variant(shadcn::ButtonVariant::Outline)
                            .size(shadcn::ButtonSize::Sm)
                            .disabled(!has_session)
                            .on_click(CMD_INSPECT_ENABLE)
                            .into_element(cx),
                        shadcn::Button::new("Inspect Off")
                            .variant(shadcn::ButtonVariant::Outline)
                            .size(shadcn::ButtonSize::Sm)
                            .disabled(!has_session)
                            .on_click(CMD_INSPECT_DISABLE)
                            .into_element(cx),
                        shadcn::Button::new("Pick")
                            .variant(shadcn::ButtonVariant::Outline)
                            .size(shadcn::ButtonSize::Sm)
                            .disabled(!has_session)
                            .on_click(CMD_PICK_ARM)
                            .into_element(cx),
                        shadcn::Button::new("Dump Bundle")
                            .variant(shadcn::ButtonVariant::Outline)
                            .size(shadcn::ButtonSize::Sm)
                            .disabled(!has_session)
                            .on_click(CMD_BUNDLE_DUMP)
                            .into_element(cx),
                        shadcn::Button::new("Screenshot")
                            .variant(shadcn::ButtonVariant::Outline)
                            .size(shadcn::ButtonSize::Sm)
                            .disabled(!has_session)
                            .on_click(CMD_SCREENSHOT_REQUEST)
                            .into_element(cx),
                    ]
                },
            );

            [left, right]
        },
    );

    let bg = theme.color_required("muted");
    let chrome = fret_ui_kit::ChromeRefinement::default()
        .bg(fret_ui_kit::ColorRef::Color(bg))
        .px(fret_ui_kit::Space::N3)
        .py(fret_ui_kit::Space::N2)
        .border_1()
        .border_color(fret_ui_kit::ColorRef::Color(theme.color_required("border")));

    cx.container(
        fret_ui_kit::declarative::style::container_props(
            theme,
            chrome,
            fret_ui_kit::LayoutRefinement::default().w_full(),
        ),
        |_cx| [actions],
    )
}

fn resizable_body(cx: &mut ElementContext<'_, App>, theme: &Theme, st: &State) -> AnyElement {
    let group = shadcn::ResizablePanelGroup::new(st.panel_fractions.clone())
        .axis(fret_core::Axis::Horizontal)
        .entries([
            shadcn::ResizablePanel::new([left_panel(cx, theme, st)]).into(),
            shadcn::ResizableHandle::new().into(),
            shadcn::ResizablePanel::new([center_panel(cx, theme, st)]).into(),
            shadcn::ResizableHandle::new().into(),
            shadcn::ResizablePanel::new([right_panel(cx, theme, st)]).into(),
        ])
        .into_element(cx);

    cx.container(
        fret_ui_kit::declarative::style::container_props(
            theme,
            fret_ui_kit::ChromeRefinement::default(),
            fret_ui_kit::LayoutRefinement::default().w_full().h_full(),
        ),
        |_cx| [group],
    )
}

fn left_panel(cx: &mut ElementContext<'_, App>, _theme: &Theme, st: &State) -> AnyElement {
    let semantics = semantics_panel(cx, st);
    let lines = cx
        .app
        .models()
        .read(&st.log_lines, |v| v.clone())
        .unwrap_or_default();

    let mut rows: Vec<AnyElement> = Vec::new();
    rows.reserve(lines.len().min(500));
    for (i, line) in lines.iter().rev().take(500).enumerate() {
        rows.push(cx.keyed(i as u64, |cx| cx.text(line.as_ref())));
    }

    let list = shadcn::ScrollArea::new([fret_ui_kit::declarative::stack::vstack(
        cx,
        fret_ui_kit::declarative::stack::VStackProps::default()
            .gap_y(fret_ui_kit::Space::N1)
            .layout(fret_ui_kit::LayoutRefinement::default().w_full()),
        |_cx| rows,
    )])
    .into_element(cx);

    let tabs = shadcn::Tabs::new(st.left_tab.clone())
        .refine_layout(fret_ui_kit::LayoutRefinement::default().w_full())
        .items([
            shadcn::TabsItem::new("semantics", "Semantics", [semantics]),
            shadcn::TabsItem::new("events", "Events", [list]),
        ])
        .into_element(cx);

    shadcn::Card::new([
        shadcn::CardHeader::new([
            shadcn::CardTitle::new("Left").into_element(cx),
            shadcn::CardDescription::new("Semantics tree and WS message tail.").into_element(cx),
        ])
        .into_element(cx),
        shadcn::CardContent::new([tabs]).into_element(cx),
    ])
    .into_element(cx)
}

fn semantics_panel(cx: &mut ElementContext<'_, App>, st: &State) -> AnyElement {
    let index = cx
        .app
        .models()
        .read(&st.semantics_cache, |v| v.clone())
        .ok()
        .flatten();
    let error = cx
        .app
        .models()
        .read(&st.semantics_error, |v| v.clone())
        .ok()
        .flatten();
    let search = cx
        .app
        .models()
        .read(&st.semantics_search, |v| v.clone())
        .unwrap_or_default();
    let expanded = cx
        .app
        .models()
        .read(&st.semantics_expanded, |v| v.clone())
        .unwrap_or_default();
    let selected_id = cx
        .app
        .models()
        .read(&st.semantics_selected_id, |v| *v)
        .ok()
        .flatten();
    let source_hash = cx
        .app
        .models()
        .read(&st.semantics_source_hash, |v| *v)
        .ok()
        .flatten()
        .unwrap_or(0);

    let search_input = shadcn::Input::new(st.semantics_search.clone())
        .a11y_label("Semantics search")
        .placeholder("Search role/test_id/label/value...")
        .into_element(cx);

    let header = fret_ui_kit::declarative::stack::hstack(
        cx,
        fret_ui_kit::declarative::stack::HStackProps::default()
            .gap_x(fret_ui_kit::Space::N2)
            .items_center(),
        |_cx| [search_input],
    );

    let content: AnyElement = match (index, error) {
        (_index, Some(err)) => cx.text(format!("semantics error: {err}")),
        (None, None) => {
            cx.text("No semantics yet. Use 'Dump Bundle' or run a script that dumps a bundle.")
        }
        (Some(index), None) => {
            #[derive(Debug, Default)]
            struct RowsCache {
                key: u64,
                rows: Arc<Vec<semantics::SemanticsRow>>,
            }

            #[derive(Debug, Default)]
            struct SelectionScrollSync {
                last: Option<(u64, u64)>,
            }

            let rows_key = {
                use std::hash::{Hash, Hasher};
                let mut hasher = std::collections::hash_map::DefaultHasher::new();
                source_hash.hash(&mut hasher);
                search.trim().to_lowercase().hash(&mut hasher);
                let mut expanded_sorted: Vec<u64> = expanded.iter().copied().collect();
                expanded_sorted.sort_unstable();
                expanded_sorted.hash(&mut hasher);
                hasher.finish()
            };

            let rows = cx.with_state(RowsCache::default, |cache| {
                if cache.key != rows_key {
                    let next = semantics::compute_rows(&index, &expanded, &search);
                    cache.key = rows_key;
                    cache.rows = Arc::new(next);
                }
                Arc::clone(&cache.rows)
            });

            let scroll_handle = cx.with_state(VirtualListScrollHandle::new, |h| h.clone());

            if let Some(sel) = selected_id {
                let rows_for_scroll = Arc::clone(&rows);
                let handle_for_scroll = scroll_handle.clone();
                cx.with_state(SelectionScrollSync::default, |sync| {
                    let next = (rows_key, sel);
                    if sync.last == Some(next) {
                        return;
                    }
                    sync.last = Some(next);

                    if let Some(idx) = rows_for_scroll.iter().position(|r| r.id == sel) {
                        handle_for_scroll.scroll_to_item(idx, ScrollStrategy::Nearest);
                    }
                });
            } else {
                cx.with_state(SelectionScrollSync::default, |sync| sync.last = None);
            }

            let mut layout = LayoutStyle::default();
            layout.size.width = Length::Fill;
            layout.size.height = Length::Fill;
            layout.flex.grow = 1.0;

            let mut options = VirtualListOptions::fixed(Px(28.0), 8).keep_alive(16);
            options.items_revision = rows_key;

            let stats = cx.text(format!(
                "window={} roots={} nodes={} rows={}",
                index.window,
                index.roots.len(),
                index.nodes_by_id.len(),
                rows.len()
            ));

            let rows_for_key = Arc::clone(&rows);
            let rows_for_row = Arc::clone(&rows);
            let index_for_list = Arc::clone(&index);
            let selected_id_for_list = selected_id;
            let has_search = !search.trim().is_empty();

            let list = cx.virtual_list_keyed_with_layout(
                layout,
                rows_for_key.len(),
                options,
                &scroll_handle,
                |i| rows_for_key[i].id,
                move |cx, i| {
                    let row = &rows_for_row[i];
                    let id = row.id;

                    let variant = if selected_id_for_list == Some(id) {
                        shadcn::ButtonVariant::Secondary
                    } else {
                        shadcn::ButtonVariant::Ghost
                    };

                    let toggle: AnyElement = if row.has_children {
                        let glyph = if row.is_expanded { "▾" } else { "▸" };
                        if has_search {
                            cx.text(glyph.to_string())
                        } else {
                            let expanded_model = st.semantics_expanded.clone();
                            let on_toggle: fret_ui::action::OnActivate =
                                Arc::new(move |host, action_cx, _reason| {
                                    let _ = host.models_mut().update(&expanded_model, |set| {
                                        if set.contains(&id) {
                                            set.remove(&id);
                                        } else {
                                            set.insert(id);
                                        }
                                    });
                                    host.request_redraw(action_cx.window);
                                });
                            shadcn::Button::new(glyph)
                                .variant(shadcn::ButtonVariant::Ghost)
                                .size(shadcn::ButtonSize::Sm)
                                .on_activate(on_toggle)
                                .into_element(cx)
                        }
                    } else {
                        cx.text(" ")
                    };

                    let label = index_for_list
                        .node(id)
                        .map(semantics::node_label)
                        .unwrap_or_else(|| format!("<missing semantics node id={id}>"));

                    let selected_id_model = st.semantics_selected_id.clone();
                    let selected_json_model = st.semantics_selected_node_json.clone();
                    let selected_live_json_model = st.semantics_selected_node_live_json.clone();
                    let selected_live_status_model = st.semantics_selected_node_live_status.clone();
                    let selected_live_updated_model =
                        st.semantics_selected_node_live_updated_unix_ms.clone();
                    let index_for_select = Arc::clone(&index_for_list);
                    let on_select: fret_ui::action::OnActivate =
                        Arc::new(move |host, action_cx, _reason| {
                            let _ = host
                                .models_mut()
                                .update(&selected_id_model, |v| *v = Some(id));
                            let text =
                                semantics::selected_node_json(index_for_select.as_ref(), Some(id));
                            let _ = host
                                .models_mut()
                                .update(&selected_json_model, |v| *v = text);
                            let _ = host
                                .models_mut()
                                .update(&selected_live_json_model, |v| v.clear());
                            let _ = host.models_mut().update(&selected_live_status_model, |v| {
                                *v = None;
                            });
                            let _ = host
                                .models_mut()
                                .update(&selected_live_updated_model, |v| *v = None);
                            host.request_redraw(action_cx.window);
                        });

                    let row_button = shadcn::Button::new(label)
                        .variant(variant)
                        .size(shadcn::ButtonSize::Sm)
                        .on_activate(on_select)
                        .refine_layout(
                            fret_ui_kit::LayoutRefinement::default()
                                .flex_1()
                                .min_w_0()
                                .ml_px(Px(12.0 * row.depth as f32)),
                        )
                        .into_element(cx);

                    fret_ui_kit::declarative::stack::hstack(
                        cx,
                        fret_ui_kit::declarative::stack::HStackProps::default()
                            .gap_x(fret_ui_kit::Space::N1)
                            .items_center()
                            .layout(fret_ui_kit::LayoutRefinement::default().w_full()),
                        |_cx| [toggle, row_button],
                    )
                },
            );

            fret_ui_kit::declarative::stack::vstack(
                cx,
                fret_ui_kit::declarative::stack::VStackProps::default()
                    .gap_y(fret_ui_kit::Space::N1)
                    .layout(fret_ui_kit::LayoutRefinement::default().w_full().h_full()),
                |_cx| [stats, list],
            )
        }
    };

    fret_ui_kit::declarative::stack::vstack(
        cx,
        fret_ui_kit::declarative::stack::VStackProps::default()
            .gap_y(fret_ui_kit::Space::N2)
            .layout(fret_ui_kit::LayoutRefinement::default().w_full().h_full()),
        |_cx| [header, content],
    )
}

fn center_panel(cx: &mut ElementContext<'_, App>, theme: &Theme, st: &State) -> AnyElement {
    let script_text = cx
        .app
        .models()
        .read(&st.script_text, |v| v.clone())
        .unwrap_or_default();
    let pick_text = cx
        .app
        .models()
        .read(&st.last_pick_json, |v| v.clone())
        .unwrap_or_default();
    let apply_pointer = cx
        .app
        .models()
        .read(&st.script_apply_pointer, |v| v.clone())
        .unwrap_or_default();
    let scripts = cx
        .app
        .models()
        .read(&st.script_library, |v| v.clone())
        .unwrap_or_default();
    let loaded_origin = cx
        .app
        .models()
        .read(&st.loaded_script_origin, |v| *v)
        .ok()
        .flatten();
    let loaded_path = cx
        .app
        .models()
        .read(&st.loaded_script_path, |v| v.clone())
        .ok()
        .flatten();
    let script_last_stage = cx
        .app
        .models()
        .read(&st.script_last_stage, |v| v.clone())
        .ok()
        .flatten();
    let script_last_step_index = cx
        .app
        .models()
        .read(&st.script_last_step_index, |v| *v)
        .ok()
        .flatten();
    let script_last_reason = cx
        .app
        .models()
        .read(&st.script_last_reason, |v| v.clone())
        .ok()
        .flatten();
    let script_last_bundle_dir = cx
        .app
        .models()
        .read(&st.script_last_bundle_dir, |v| v.clone())
        .ok()
        .flatten();
    let pack_after_run = cx
        .app
        .models()
        .read(&st.script_pack_after_run, |v| *v)
        .unwrap_or(false);

    let target_out_dir = cx
        .app
        .models()
        .read(&st.target_out_dir, |v| v.clone())
        .ok()
        .flatten();
    let last_bundle_dir_abs = cx
        .app
        .models()
        .read(&st.last_bundle_dir_abs, |v| v.clone())
        .ok()
        .flatten();
    let last_bundle_dump_bundle_json = cx
        .app
        .models()
        .read(&st.last_bundle_dump_bundle_json, |v| v.clone())
        .ok()
        .flatten();
    let last_pack_path = cx
        .app
        .models()
        .read(&st.last_pack_path, |v| v.clone())
        .ok()
        .flatten();
    let pack_in_flight = cx
        .app
        .models()
        .read(&st.pack_in_flight, |v| *v)
        .unwrap_or(false);
    let pack_last_error = cx
        .app
        .models()
        .read(&st.pack_last_error, |v| v.clone())
        .ok()
        .flatten();
    let viewer_url = cx
        .app
        .models()
        .read(&st.viewer_url, |v| v.clone())
        .unwrap_or_default();

    let consume_clicks = cx
        .app
        .models()
        .read(&st.inspect_consume_clicks, |v| *v)
        .unwrap_or(false);

    let consume_toggle = shadcn::Checkbox::new(st.inspect_consume_clicks.clone())
        .a11y_label("Consume clicks while inspecting")
        .into_element(cx);

    let has_session = cx
        .app
        .models()
        .read(&st.selected_session_id, |v| v.is_some())
        .unwrap_or(false);

    let can_fork = loaded_origin == Some(script_studio::ScriptOrigin::WorkspaceTools);
    let can_save = loaded_origin == Some(script_studio::ScriptOrigin::UserLocal);
    let can_apply_pick = !pick_text.trim().is_empty() && !apply_pointer.trim().is_empty();
    let can_pack = last_bundle_dir_abs.is_some() || last_bundle_dump_bundle_json.is_some();

    let pointer_input = shadcn::Input::new(st.script_apply_pointer.clone())
        .a11y_label("JSON pointer")
        .placeholder("/steps/0/target")
        .into_element(cx);

    let viewer_url_input = shadcn::Input::new(st.viewer_url.clone())
        .a11y_label("Bundle viewer URL")
        .placeholder("http://localhost:5173")
        .into_element(cx);

    let textarea = shadcn::Textarea::new(st.script_text.clone())
        .a11y_label("Script JSON")
        .min_height(Px(360.0))
        .refine_layout(fret_ui_kit::LayoutRefinement::default().w_full().h_full())
        .into_element(cx);

    let (script_summary, script_is_valid) = script_summary_line(&script_text);
    let script_steps = script_steps_len(&script_text).unwrap_or(0);
    let status_line = {
        let stage = script_last_stage
            .as_ref()
            .map(|s| format!("{s:?}"))
            .unwrap_or_else(|| "None".to_string());
        let step = script_last_step_index
            .map(|s| s.to_string())
            .unwrap_or_else(|| "-".to_string());
        let reason = script_last_reason
            .as_deref()
            .map(|s| s.to_string())
            .unwrap_or_else(|| "-".to_string());
        let bundle = script_last_bundle_dir
            .as_deref()
            .map(|s| s.to_string())
            .unwrap_or_else(|| "-".to_string());
        format!(
            "status={stage} step={step}/{script_steps} reason={reason} last_bundle_dir={bundle}"
        )
    };
    let pack_status_line = {
        let err = pack_last_error
            .as_deref()
            .map(|s| s.to_string())
            .unwrap_or_else(|| "-".to_string());
        format!(
            "pack_in_flight={} pack_last_error={err}",
            if pack_in_flight { "true" } else { "false" }
        )
    };

    let buttons = fret_ui_kit::declarative::stack::hstack(
        cx,
        fret_ui_kit::declarative::stack::HStackProps::default()
            .gap_x(fret_ui_kit::Space::N2)
            .items_center(),
        |cx| {
            [
                shadcn::Button::new("Push Script")
                    .variant(shadcn::ButtonVariant::Secondary)
                    .size(shadcn::ButtonSize::Sm)
                    .disabled(!has_session || !script_is_valid)
                    .on_click(CMD_SCRIPT_PUSH)
                    .into_element(cx),
                shadcn::Button::new("Run Script")
                    .variant(shadcn::ButtonVariant::Default)
                    .size(shadcn::ButtonSize::Sm)
                    .disabled(!has_session || !script_is_valid)
                    .on_click(CMD_SCRIPT_RUN)
                    .into_element(cx),
                shadcn::Button::new("Run & Pack")
                    .variant(shadcn::ButtonVariant::Secondary)
                    .size(shadcn::ButtonSize::Sm)
                    .disabled(!has_session || !script_is_valid)
                    .on_click(CMD_SCRIPT_RUN_AND_PACK)
                    .into_element(cx),
                shadcn::Button::new("Refresh Scripts")
                    .variant(shadcn::ButtonVariant::Outline)
                    .size(shadcn::ButtonSize::Sm)
                    .on_click(CMD_SCRIPTS_REFRESH)
                    .into_element(cx),
                shadcn::Button::new("Fork")
                    .variant(shadcn::ButtonVariant::Outline)
                    .size(shadcn::ButtonSize::Sm)
                    .disabled(!can_fork)
                    .on_click(CMD_SCRIPT_FORK)
                    .into_element(cx),
                shadcn::Button::new("Save")
                    .variant(shadcn::ButtonVariant::Outline)
                    .size(shadcn::ButtonSize::Sm)
                    .disabled(!can_save)
                    .on_click(CMD_SCRIPT_SAVE)
                    .into_element(cx),
                consume_toggle,
                cx.text(format!(
                    "consume_clicks={}",
                    if consume_clicks { "true" } else { "false" }
                )),
            ]
        },
    );

    let pack_row = fret_ui_kit::declarative::stack::hstack(
        cx,
        fret_ui_kit::declarative::stack::HStackProps::default()
            .gap_x(fret_ui_kit::Space::N2)
            .items_center(),
        |cx| {
            let copy_enabled = last_pack_path.is_some();
            [
                cx.text("Artifacts:"),
                shadcn::Button::new("Pack last bundle")
                    .variant(shadcn::ButtonVariant::Outline)
                    .size(shadcn::ButtonSize::Sm)
                    .disabled(!can_pack || pack_in_flight)
                    .on_click(CMD_PACK_LAST_BUNDLE)
                    .into_element(cx),
                shadcn::Button::new("Copy pack path")
                    .variant(shadcn::ButtonVariant::Outline)
                    .size(shadcn::ButtonSize::Sm)
                    .disabled(!copy_enabled)
                    .on_click(CMD_COPY_PACK_PATH)
                    .into_element(cx),
                viewer_url_input,
                shadcn::Button::new("Open viewer")
                    .variant(shadcn::ButtonVariant::Outline)
                    .size(shadcn::ButtonSize::Sm)
                    .disabled(viewer_url.trim().is_empty())
                    .on_click(CMD_OPEN_VIEWER_URL)
                    .into_element(cx),
            ]
        },
    );

    let apply_row = fret_ui_kit::declarative::stack::hstack(
        cx,
        fret_ui_kit::declarative::stack::HStackProps::default()
            .gap_x(fret_ui_kit::Space::N2)
            .items_center(),
        |cx| {
            [
                cx.text("Pick-to-fill:"),
                pointer_input,
                shadcn::Button::new("Apply Pick")
                    .variant(shadcn::ButtonVariant::Secondary)
                    .size(shadcn::ButtonSize::Sm)
                    .disabled(!can_apply_pick)
                    .on_click(CMD_SCRIPT_APPLY_PICK)
                    .into_element(cx),
            ]
        },
    );

    let loaded_line = match (loaded_origin, loaded_path.as_deref()) {
        (Some(origin), Some(path)) => format!("Loaded: [{}] {path}", origin.label()),
        _ => "Loaded: <none>".to_string(),
    };
    let out_dir_line = match target_out_dir.as_deref() {
        Some(dir) => format!("Target diag out_dir: {dir}"),
        None => "Target diag out_dir: <unknown>".to_string(),
    };
    let pack_line = match last_pack_path.as_deref() {
        Some(p) => format!("Last pack: {p}"),
        None => "Last pack: <none>".to_string(),
    };

    let mut script_rows: Vec<AnyElement> = Vec::new();
    for item in scripts.iter() {
        let label = format!("[{}] {}", item.origin.label(), item.file_name);
        let is_loaded = loaded_path
            .as_deref()
            .is_some_and(|p| PathBuf::from(p) == item.path);

        let variant = if is_loaded {
            shadcn::ButtonVariant::Secondary
        } else {
            shadcn::ButtonVariant::Ghost
        };

        let origin_for_activate = item.origin;
        let path_for_activate = item.path.clone();
        let script_text_for_activate = st.script_text.clone();
        let loaded_origin_for_activate = st.loaded_script_origin.clone();
        let loaded_path_for_activate = st.loaded_script_path.clone();
        let log_lines_for_activate = st.log_lines.clone();

        let on_activate: fret_ui::action::OnActivate = Arc::new(move |host, action_cx, _reason| {
            let text = match std::fs::read_to_string(&path_for_activate) {
                Ok(text) => text,
                Err(err) => {
                    let line = format!("script load failed: {err}");
                    let _ = host.models_mut().update(&log_lines_for_activate, |v| {
                        v.push(Arc::<str>::from(line));
                        if v.len() > 2000 {
                            let drain = v.len().saturating_sub(2000);
                            v.drain(0..drain);
                        }
                    });
                    host.request_redraw(action_cx.window);
                    return;
                }
            };

            let _ = host.models_mut().update(&script_text_for_activate, |v| {
                *v = text;
            });
            let _ = host.models_mut().update(&loaded_origin_for_activate, |v| {
                *v = Some(origin_for_activate)
            });
            let _ = host.models_mut().update(&loaded_path_for_activate, |v| {
                *v = Some(Arc::<str>::from(
                    path_for_activate.to_string_lossy().to_string(),
                ))
            });

            host.request_redraw(action_cx.window);
            host.push_effect(fret_runtime::Effect::RequestAnimationFrame(
                action_cx.window,
            ));
        });

        script_rows.push(
            shadcn::Button::new(label)
                .variant(variant)
                .size(shadcn::ButtonSize::Sm)
                .on_activate(on_activate)
                .refine_layout(fret_ui_kit::LayoutRefinement::default().w_full())
                .into_element(cx),
        );
    }

    let scripts_list = shadcn::ScrollArea::new([fret_ui_kit::declarative::stack::vstack(
        cx,
        fret_ui_kit::declarative::stack::VStackProps::default()
            .gap_y(fret_ui_kit::Space::N1)
            .layout(fret_ui_kit::LayoutRefinement::default().w_full()),
        |_cx| script_rows,
    )])
    .into_element(cx);

    let script_schema_version = infer_script_schema_version(&script_text).unwrap_or(1);
    let pointer_candidates = script_studio::collect_common_json_pointers(&script_text);

    let step_index_input = shadcn::Input::new(st.script_step_insert_index.clone())
        .a11y_label("Step insert index")
        .placeholder("(append)")
        .into_element(cx);

    let mut step_buttons: Vec<AnyElement> = Vec::new();
    for t in step_templates_for_schema(script_schema_version) {
        let script_text_model = st.script_text.clone();
        let insert_index_model = st.script_step_insert_index.clone();
        let pointer_model = st.script_apply_pointer.clone();
        let log_lines = st.log_lines.clone();
        let step_value = t.step.clone();
        let label = t.label;

        let on_activate: fret_ui::action::OnActivate = Arc::new(move |host, action_cx, _reason| {
            let index_text = host
                .models_mut()
                .read(&insert_index_model, |v: &String| v.clone())
                .ok()
                .unwrap_or_default();
            let index = index_text.trim().parse::<usize>().ok();

            let current = host
                .models_mut()
                .read(&script_text_model, |v: &String| v.clone())
                .ok()
                .unwrap_or_default();

            let len_before = script_steps_len(&current).unwrap_or(0);
            let insert_at = index.unwrap_or(len_before);
            let inserted_index = insert_at.min(len_before);

            let updated = match index {
                Some(i) => script_studio::insert_step_json(&current, i, step_value.clone()),
                None => script_studio::append_step_json(&current, step_value.clone()),
            };

            match updated {
                Ok(text) => {
                    let _ = host.models_mut().update(&script_text_model, |v| *v = text);
                    if let Some(suffix) = primary_pointer_suffix_for_step_json(&step_value) {
                        let ptr = format!("/steps/{inserted_index}{suffix}");
                        let _ = host.models_mut().update(&pointer_model, |v| *v = ptr);
                    }
                }
                Err(err) => {
                    let _ = host.models_mut().update(&log_lines, |v| {
                        v.push(Arc::<str>::from(format!(
                            "insert step failed ({label}): {err}"
                        )));
                        if v.len() > 2000 {
                            let drain = v.len().saturating_sub(2000);
                            v.drain(0..drain);
                        }
                    });
                }
            }

            host.request_redraw(action_cx.window);
            host.push_effect(Effect::RequestAnimationFrame(action_cx.window));
        });

        step_buttons.push(
            shadcn::Button::new(t.label)
                .variant(shadcn::ButtonVariant::Secondary)
                .size(shadcn::ButtonSize::Sm)
                .on_activate(on_activate)
                .refine_layout(fret_ui_kit::LayoutRefinement::default().w_full())
                .into_element(cx),
        );
    }

    let steps_tab = shadcn::ScrollArea::new([fret_ui_kit::declarative::stack::vstack(
        cx,
        fret_ui_kit::declarative::stack::VStackProps::default()
            .gap_y(fret_ui_kit::Space::N2)
            .layout(fret_ui_kit::LayoutRefinement::default().w_full()),
        |cx| {
            let mut out: Vec<AnyElement> = Vec::new();
            out.push(cx.text(format!("Schema v{script_schema_version} step palette")));
            out.push(step_index_input);
            out.extend(step_buttons);
            if !pointer_candidates.is_empty() {
                out.push(cx.text("Pointer candidates:"));
                for p in pointer_candidates.iter().take(64) {
                    let pointer_model = st.script_apply_pointer.clone();
                    let p_value = p.clone();
                    let p_label = p.clone();
                    let on_activate: fret_ui::action::OnActivate =
                        Arc::new(move |host, action_cx, _reason| {
                            let _ = host
                                .models_mut()
                                .update(&pointer_model, |v| *v = p_value.clone());
                            host.request_redraw(action_cx.window);
                            host.push_effect(Effect::RequestAnimationFrame(action_cx.window));
                        });
                    out.push(
                        shadcn::Button::new(p_label)
                            .variant(shadcn::ButtonVariant::Ghost)
                            .size(shadcn::ButtonSize::Sm)
                            .on_activate(on_activate)
                            .refine_layout(fret_ui_kit::LayoutRefinement::default().w_full())
                            .into_element(cx),
                    );
                }
            }
            out
        },
    )])
    .into_element(cx);

    let selector_kind_items = [
        shadcn::SelectItem::new("test_id", "test_id"),
        shadcn::SelectItem::new("role_and_name", "role_and_name"),
        shadcn::SelectItem::new("role_and_path", "role_and_path"),
        shadcn::SelectItem::new("node_id", "node_id"),
        shadcn::SelectItem::new("global_element_id", "global_element_id"),
    ];
    let selector_kind_select = shadcn::Select::new(
        st.script_selector_kind.clone(),
        st.script_selector_kind_open.clone(),
    )
    .placeholder("selector kind")
    .items(selector_kind_items)
    .refine_layout(fret_ui_kit::LayoutRefinement::default().w_full())
    .into_element(cx);

    let selector_kind = cx
        .app
        .models()
        .read(&st.script_selector_kind, |v| v.clone())
        .ok()
        .flatten()
        .unwrap_or_else(|| Arc::<str>::from("test_id"));
    let selector_value = selector_value_from_models(cx, st, selector_kind.as_ref());
    let selector_json =
        serde_json::to_string_pretty(&selector_value).unwrap_or_else(|_| "{}".to_string());

    let selector_apply = {
        let script_text_model = st.script_text.clone();
        let pointer_model = st.script_apply_pointer.clone();
        let log_lines = st.log_lines.clone();
        let selector_value = selector_value.clone();
        let on_activate: fret_ui::action::OnActivate = Arc::new(move |host, action_cx, _reason| {
            let pointer = host
                .models_mut()
                .read(&pointer_model, |v: &String| v.clone())
                .ok()
                .unwrap_or_default();
            if pointer.trim().is_empty() {
                let _ = host.models_mut().update(&log_lines, |v| {
                    v.push(Arc::<str>::from(
                        "apply selector refused (empty json pointer)",
                    ));
                });
                host.request_redraw(action_cx.window);
                return;
            }

            let current = host
                .models_mut()
                .read(&script_text_model, |v: &String| v.clone())
                .ok()
                .unwrap_or_default();
            match script_studio::apply_json_value_to_json_pointer(
                &current,
                &pointer,
                selector_value.clone(),
            ) {
                Ok(updated) => {
                    let _ = host
                        .models_mut()
                        .update(&script_text_model, |v| *v = updated);
                }
                Err(err) => {
                    let _ = host.models_mut().update(&log_lines, |v| {
                        v.push(Arc::<str>::from(format!("apply selector failed: {err}")));
                    });
                }
            }
            host.request_redraw(action_cx.window);
            host.push_effect(Effect::RequestAnimationFrame(action_cx.window));
        });
        on_activate
    };

    let selector_copy = {
        let selector_json = selector_json.clone();
        let on_activate: fret_ui::action::OnActivate = Arc::new(move |host, action_cx, _reason| {
            host.push_effect(Effect::ClipboardSetText {
                text: selector_json.clone(),
            });
            host.request_redraw(action_cx.window);
        });
        on_activate
    };

    let selector_tab = fret_ui_kit::declarative::stack::vstack(
        cx,
        fret_ui_kit::declarative::stack::VStackProps::default()
            .gap_y(fret_ui_kit::Space::N2)
            .layout(fret_ui_kit::LayoutRefinement::default().w_full()),
        |cx| {
            let fields = selector_fields(cx, st, selector_kind.as_ref());
            let preview = text_blob(cx, selector_json.clone());
            [
                selector_kind_select,
                fields,
                fret_ui_kit::declarative::stack::hstack(
                    cx,
                    fret_ui_kit::declarative::stack::HStackProps::default()
                        .gap_x(fret_ui_kit::Space::N2)
                        .items_center(),
                    |cx| {
                        [
                            shadcn::Button::new("Apply to pointer")
                                .variant(shadcn::ButtonVariant::Secondary)
                                .size(shadcn::ButtonSize::Sm)
                                .on_activate(selector_apply)
                                .into_element(cx),
                            shadcn::Button::new("Copy JSON")
                                .variant(shadcn::ButtonVariant::Outline)
                                .size(shadcn::ButtonSize::Sm)
                                .on_activate(selector_copy)
                                .into_element(cx),
                        ]
                    },
                ),
                preview,
            ]
        },
    );

    let predicate_kind_items = [
        shadcn::SelectItem::new("exists", "exists"),
        shadcn::SelectItem::new("not_exists", "not_exists"),
        shadcn::SelectItem::new("focus_is", "focus_is"),
        shadcn::SelectItem::new("role_is", "role_is"),
        shadcn::SelectItem::new("checked_is", "checked_is"),
        shadcn::SelectItem::new("checked_is_none", "checked_is_none"),
        shadcn::SelectItem::new("barrier_roots", "barrier_roots"),
        shadcn::SelectItem::new("visible_in_window", "visible_in_window"),
        shadcn::SelectItem::new("bounds_within_window", "bounds_within_window"),
        shadcn::SelectItem::new("bounds_min_size", "bounds_min_size"),
        shadcn::SelectItem::new("bounds_non_overlapping", "bounds_non_overlapping"),
        shadcn::SelectItem::new("bounds_overlapping", "bounds_overlapping"),
        shadcn::SelectItem::new("bounds_overlapping_x", "bounds_overlapping_x"),
        shadcn::SelectItem::new("bounds_overlapping_y", "bounds_overlapping_y"),
    ];
    let predicate_kind_select = shadcn::Select::new(
        st.script_predicate_kind.clone(),
        st.script_predicate_kind_open.clone(),
    )
    .placeholder("predicate kind")
    .items(predicate_kind_items)
    .refine_layout(fret_ui_kit::LayoutRefinement::default().w_full())
    .into_element(cx);

    let predicate_kind = cx
        .app
        .models()
        .read(&st.script_predicate_kind, |v| v.clone())
        .ok()
        .flatten()
        .unwrap_or_else(|| Arc::<str>::from("exists"));
    let predicate_value =
        predicate_value_from_models(cx, st, predicate_kind.as_ref(), selector_value.clone());
    let predicate_json =
        serde_json::to_string_pretty(&predicate_value).unwrap_or_else(|_| "{}".to_string());

    let predicate_apply = {
        let script_text_model = st.script_text.clone();
        let pointer_model = st.script_apply_pointer.clone();
        let log_lines = st.log_lines.clone();
        let predicate_value = predicate_value.clone();
        let on_activate: fret_ui::action::OnActivate = Arc::new(move |host, action_cx, _reason| {
            let pointer = host
                .models_mut()
                .read(&pointer_model, |v: &String| v.clone())
                .ok()
                .unwrap_or_default();
            if pointer.trim().is_empty() {
                let _ = host.models_mut().update(&log_lines, |v| {
                    v.push(Arc::<str>::from(
                        "apply predicate refused (empty json pointer)",
                    ));
                });
                host.request_redraw(action_cx.window);
                return;
            }

            let current = host
                .models_mut()
                .read(&script_text_model, |v: &String| v.clone())
                .ok()
                .unwrap_or_default();
            match script_studio::apply_json_value_to_json_pointer(
                &current,
                &pointer,
                predicate_value.clone(),
            ) {
                Ok(updated) => {
                    let _ = host
                        .models_mut()
                        .update(&script_text_model, |v| *v = updated);
                }
                Err(err) => {
                    let _ = host.models_mut().update(&log_lines, |v| {
                        v.push(Arc::<str>::from(format!("apply predicate failed: {err}")));
                    });
                }
            }
            host.request_redraw(action_cx.window);
            host.push_effect(Effect::RequestAnimationFrame(action_cx.window));
        });
        on_activate
    };

    let predicate_copy = {
        let predicate_json = predicate_json.clone();
        let on_activate: fret_ui::action::OnActivate = Arc::new(move |host, action_cx, _reason| {
            host.push_effect(Effect::ClipboardSetText {
                text: predicate_json.clone(),
            });
            host.request_redraw(action_cx.window);
        });
        on_activate
    };

    let predicate_tab = fret_ui_kit::declarative::stack::vstack(
        cx,
        fret_ui_kit::declarative::stack::VStackProps::default()
            .gap_y(fret_ui_kit::Space::N2)
            .layout(fret_ui_kit::LayoutRefinement::default().w_full()),
        |cx| {
            let fields = predicate_fields(cx, st, predicate_kind.as_ref());
            let preview = text_blob(cx, predicate_json.clone());
            [
                predicate_kind_select,
                fields,
                fret_ui_kit::declarative::stack::hstack(
                    cx,
                    fret_ui_kit::declarative::stack::HStackProps::default()
                        .gap_x(fret_ui_kit::Space::N2)
                        .items_center(),
                    |cx| {
                        [
                            shadcn::Button::new("Apply to pointer")
                                .variant(shadcn::ButtonVariant::Secondary)
                                .size(shadcn::ButtonSize::Sm)
                                .on_activate(predicate_apply)
                                .into_element(cx),
                            shadcn::Button::new("Copy JSON")
                                .variant(shadcn::ButtonVariant::Outline)
                                .size(shadcn::ButtonSize::Sm)
                                .on_activate(predicate_copy)
                                .into_element(cx),
                        ]
                    },
                ),
                preview,
            ]
        },
    );

    let helpers_tabs = shadcn::Tabs::new(st.script_studio_helper_tab.clone())
        .refine_layout(fret_ui_kit::LayoutRefinement::default().w_full())
        .items([
            shadcn::TabsItem::new("steps", "Steps", [steps_tab]),
            shadcn::TabsItem::new("selector", "Selector", [selector_tab]),
            shadcn::TabsItem::new("predicate", "Predicate", [predicate_tab]),
        ])
        .into_element(cx);

    let split = fret_ui_kit::declarative::stack::hstack(
        cx,
        fret_ui_kit::declarative::stack::HStackProps::default()
            .gap_x(fret_ui_kit::Space::N2)
            .layout(fret_ui_kit::LayoutRefinement::default().w_full().h_full())
            .items_start(),
        |cx| {
            [
                cx.container(
                    fret_ui_kit::declarative::style::container_props(
                        theme,
                        fret_ui_kit::ChromeRefinement::default(),
                        fret_ui_kit::LayoutRefinement::default()
                            .w_px(Px(240.0))
                            .h_full(),
                    ),
                    |_cx| [scripts_list],
                ),
                cx.container(
                    fret_ui_kit::declarative::style::container_props(
                        theme,
                        fret_ui_kit::ChromeRefinement::default(),
                        fret_ui_kit::LayoutRefinement::default()
                            .flex_1()
                            .min_w_0()
                            .h_full(),
                    ),
                    |_cx| [textarea],
                ),
                cx.container(
                    fret_ui_kit::declarative::style::container_props(
                        theme,
                        fret_ui_kit::ChromeRefinement::default(),
                        fret_ui_kit::LayoutRefinement::default()
                            .w_px(Px(320.0))
                            .h_full(),
                    ),
                    |_cx| [helpers_tabs],
                ),
            ]
        },
    );

    shadcn::Card::new([
        shadcn::CardHeader::new([
            shadcn::CardTitle::new("Script Studio").into_element(cx),
            shadcn::CardDescription::new("Browse, fork, edit, and run diagnostics scripts.")
                .into_element(cx),
        ])
        .into_element(cx),
        shadcn::CardContent::new([
            buttons,
            cx.text(format!("validate: {script_summary}")),
            cx.text(status_line),
            cx.text(pack_status_line),
            cx.text(format!(
                "pack_after_run={}",
                if pack_after_run { "true" } else { "false" }
            )),
            apply_row,
            pack_row,
            cx.text(loaded_line),
            cx.text(out_dir_line),
            cx.text(pack_line),
            cx.text(format!("Library: {} scripts", scripts.len())),
            split,
            cx.text(format!("Script text bytes={}", script_text.len())),
        ])
        .into_element(cx),
    ])
    .into_element(cx)
}

fn right_panel(cx: &mut ElementContext<'_, App>, _theme: &Theme, st: &State) -> AnyElement {
    let pick = cx
        .app
        .models()
        .read(&st.last_pick_json, |v| v.clone())
        .unwrap_or_default();
    let inspect_hover = cx
        .app
        .models()
        .read(&st.last_inspect_hover_json, |v| v.clone())
        .unwrap_or_default();
    let inspect_focus = cx
        .app
        .models()
        .read(&st.last_inspect_focus_json, |v| v.clone())
        .unwrap_or_default();
    let script = cx
        .app
        .models()
        .read(&st.last_script_result_json, |v| v.clone())
        .unwrap_or_default();
    let bundle = cx
        .app
        .models()
        .read(&st.last_bundle_json, |v| v.clone())
        .unwrap_or_default();
    let screenshot = cx
        .app
        .models()
        .read(&st.last_screenshot_json, |v| v.clone())
        .unwrap_or_default();
    let semantics_node = sem_node_panel(cx, st);

    let inspect = if inspect_hover.trim().is_empty() && inspect_focus.trim().is_empty() {
        String::new()
    } else {
        format!("hover:\n{inspect_hover}\n\nfocus:\n{inspect_focus}")
    };

    let tabs = shadcn::Tabs::new(st.details_tab.clone())
        .refine_layout(fret_ui_kit::LayoutRefinement::default().w_full())
        .items([
            shadcn::TabsItem::new("inspect", "Inspect", [text_blob(cx, inspect)]),
            shadcn::TabsItem::new("pick", "Pick", [text_blob(cx, pick)]),
            shadcn::TabsItem::new("script", "Script", [text_blob(cx, script)]),
            shadcn::TabsItem::new("bundle", "Bundle", [text_blob(cx, bundle)]),
            shadcn::TabsItem::new("screenshot", "Screenshot", [text_blob(cx, screenshot)]),
            shadcn::TabsItem::new("sem_node", "Sem Node", [semantics_node]),
        ])
        .into_element(cx);

    shadcn::Card::new([
        shadcn::CardHeader::new([
            shadcn::CardTitle::new("Latest").into_element(cx),
            shadcn::CardDescription::new("Latest pick/script/bundle payloads.").into_element(cx),
        ])
        .into_element(cx),
        shadcn::CardContent::new([tabs]).into_element(cx),
    ])
    .into_element(cx)
}

fn text_blob(cx: &mut ElementContext<'_, App>, text: String) -> AnyElement {
    let text = if text.is_empty() {
        "<empty>".to_string()
    } else {
        text
    };

    let pre = cx.text(text);
    shadcn::ScrollArea::new([pre]).into_element(cx)
}

fn sem_node_panel(cx: &mut ElementContext<'_, App>, st: &State) -> AnyElement {
    let fallback = cx
        .app
        .models()
        .read(&st.semantics_selected_node_json, |v| v.clone())
        .unwrap_or_default();
    let live = cx
        .app
        .models()
        .read(&st.semantics_selected_node_live_json, |v| v.clone())
        .unwrap_or_default();
    let live_status = cx
        .app
        .models()
        .read(&st.semantics_selected_node_live_status, |v| v.clone())
        .ok()
        .flatten()
        .unwrap_or_else(|| Arc::<str>::from("unknown"));
    let live_updated = cx
        .app
        .models()
        .read(&st.semantics_selected_node_live_updated_unix_ms, |v| *v)
        .ok()
        .flatten();
    let children = cx
        .app
        .models()
        .read(&st.semantics_selected_node_live_children, |v| v.clone())
        .unwrap_or_default();
    let live_enabled = cx
        .app
        .models()
        .read(&st.semantics_live_enabled, |v| *v)
        .unwrap_or(true);
    let selected_id = cx
        .app
        .models()
        .read(&st.semantics_selected_id, |v| *v)
        .ok()
        .flatten();
    let index = cx
        .app
        .models()
        .read(&st.semantics_cache, |v| v.clone())
        .ok()
        .flatten();

    let status_line = {
        let mut line = format!(
            "live_enabled={live_enabled} status={}",
            live_status.as_ref()
        );
        if let Some(ts) = live_updated {
            line.push_str(&format!(" updated_unix_ms={ts}"));
        }
        line
    };

    let live_toggle_label = if live_enabled {
        "Live: On"
    } else {
        "Live: Off"
    };
    let live_enabled_model = st.semantics_live_enabled.clone();
    let force_nonce_model = st.semantics_live_force_nonce.clone();
    let on_toggle: fret_ui::action::OnActivate = Arc::new(move |host, action_cx, _reason| {
        let _ = host.models_mut().update(&live_enabled_model, |v| *v = !*v);
        let _ = host
            .models_mut()
            .update(&force_nonce_model, |v| *v = v.saturating_add(1));
        host.request_redraw(action_cx.window);
    });

    let on_refresh: fret_ui::action::OnActivate = {
        let force_nonce_model = st.semantics_live_force_nonce.clone();
        Arc::new(move |host, action_cx, _reason| {
            let _ = host
                .models_mut()
                .update(&force_nonce_model, |v| *v = v.saturating_add(1));
            host.request_redraw(action_cx.window);
        })
    };

    let live_toggle_btn = shadcn::Button::new(live_toggle_label)
        .variant(shadcn::ButtonVariant::Outline)
        .size(shadcn::ButtonSize::Sm)
        .on_activate(on_toggle)
        .into_element(cx);
    let refresh_btn = shadcn::Button::new("Refresh")
        .variant(shadcn::ButtonVariant::Outline)
        .size(shadcn::ButtonSize::Sm)
        .on_activate(on_refresh)
        .into_element(cx);
    let status_elem = cx.text(status_line);

    let header = fret_ui_kit::declarative::stack::hstack(
        cx,
        fret_ui_kit::declarative::stack::HStackProps::default()
            .gap_x(fret_ui_kit::Space::N2)
            .items_center()
            .layout(fret_ui_kit::LayoutRefinement::default().w_full()),
        |_cx| [live_toggle_btn, refresh_btn, status_elem],
    );

    let mut child_buttons: Vec<AnyElement> = Vec::new();
    child_buttons.reserve(children.len().min(64));
    if let (Some(index), Some(_selected)) = (index, selected_id) {
        for child in children.iter().take(200) {
            let id = *child;
            let label = index
                .node(id)
                .map(semantics::node_label)
                .unwrap_or_else(|| format!("id={id}"));

            let selected_id_model = st.semantics_selected_id.clone();
            let selected_json_model = st.semantics_selected_node_json.clone();
            let selected_live_json_model = st.semantics_selected_node_live_json.clone();
            let selected_live_status_model = st.semantics_selected_node_live_status.clone();
            let selected_live_updated_model =
                st.semantics_selected_node_live_updated_unix_ms.clone();
            let selected_live_children_model = st.semantics_selected_node_live_children.clone();
            let index_for_select = Arc::clone(&index);
            let on_child: fret_ui::action::OnActivate =
                Arc::new(move |host, action_cx, _reason| {
                    let _ = host
                        .models_mut()
                        .update(&selected_id_model, |v| *v = Some(id));
                    let text = semantics::selected_node_json(index_for_select.as_ref(), Some(id));
                    let _ = host
                        .models_mut()
                        .update(&selected_json_model, |v| *v = text);
                    let _ = host
                        .models_mut()
                        .update(&selected_live_json_model, |v| v.clear());
                    let _ = host.models_mut().update(&selected_live_status_model, |v| {
                        *v = None;
                    });
                    let _ = host
                        .models_mut()
                        .update(&selected_live_updated_model, |v| *v = None);
                    let _ = host
                        .models_mut()
                        .update(&selected_live_children_model, |v| v.clear());
                    host.request_redraw(action_cx.window);
                });

            child_buttons.push(
                shadcn::Button::new(label)
                    .variant(shadcn::ButtonVariant::Ghost)
                    .size(shadcn::ButtonSize::Sm)
                    .on_activate(on_child)
                    .into_element(cx),
            );
        }
    }

    let children_panel = if child_buttons.is_empty() {
        cx.text("children: <none>")
    } else {
        shadcn::ScrollArea::new([fret_ui_kit::declarative::stack::vstack(
            cx,
            fret_ui_kit::declarative::stack::VStackProps::default()
                .gap_y(fret_ui_kit::Space::N1)
                .layout(fret_ui_kit::LayoutRefinement::default().w_full()),
            |_cx| child_buttons,
        )])
        .refine_layout(
            fret_ui_kit::LayoutRefinement::default()
                .w_full()
                .h_px(Px(160.0)),
        )
        .into_element(cx)
    };

    let json_text = if !live.is_empty() { live } else { fallback };
    let body = text_blob(cx, json_text);

    fret_ui_kit::declarative::stack::vstack(
        cx,
        fret_ui_kit::declarative::stack::VStackProps::default()
            .gap_y(fret_ui_kit::Space::N2)
            .layout(fret_ui_kit::LayoutRefinement::default().w_full().h_full()),
        |_cx| [header, children_panel, body],
    )
}

fn on_command(
    app: &mut App,
    _services: &mut dyn UiServices,
    window: AppWindowId,
    _ui: &mut fret_ui::UiTree<App>,
    st: &mut State,
    cmd: &CommandId,
) {
    ws::sync_selected_session_to_client(app, st);

    match cmd.as_str() {
        CMD_COPY_WS_URL => {
            let text = format!(
                "{}?fret_devtools_token={}",
                st.cfg.ws_url.as_ref(),
                st.cfg.token.as_ref()
            );
            app.push_effect(Effect::ClipboardSetText { text });
        }
        CMD_COPY_TOKEN => {
            app.push_effect(Effect::ClipboardSetText {
                text: st.cfg.token.to_string(),
            });
        }
        CMD_INSPECT_ENABLE | CMD_INSPECT_DISABLE => {
            if !ws::require_session_selected(app, st) {
                app.request_redraw(window);
                return;
            }
            let enabled = cmd.as_str() == CMD_INSPECT_ENABLE;
            let consume_clicks = app
                .models()
                .read(&st.inspect_consume_clicks, |v| *v)
                .unwrap_or(false);
            st.devtools.inspect_set(None, enabled, consume_clicks);
            app.push_effect(Effect::RequestAnimationFrame(window));
        }
        CMD_PICK_ARM => {
            if !ws::require_session_selected(app, st) {
                app.request_redraw(window);
                return;
            }
            st.devtools.pick_arm(None);
            app.push_effect(Effect::RequestAnimationFrame(window));
        }
        CMD_BUNDLE_DUMP => {
            if !ws::require_session_selected(app, st) {
                app.request_redraw(window);
                return;
            }
            st.devtools.bundle_dump(None, Some("devtools"));
            app.push_effect(Effect::RequestAnimationFrame(window));
        }
        CMD_SCREENSHOT_REQUEST => {
            if !ws::require_session_selected(app, st) {
                app.request_redraw(window);
                return;
            }
            if st.devtools.client().kind() != DiagTransportKind::WebSocket {
                push_log(
                    app,
                    &st.log_lines,
                    "screenshot.request requires WebSocket transport (filesystem mode cannot request runner-owned screenshots)",
                );
                app.request_redraw(window);
                return;
            }
            let _ = st
                .devtools
                .screenshot_request(None, Some("devtools"), 300, None);
            app.push_effect(Effect::RequestAnimationFrame(window));
        }
        CMD_SCRIPTS_REFRESH => {
            refresh_script_library(app, st);
            app.request_redraw(window);
        }
        CMD_SCRIPT_FORK => {
            fork_loaded_script(app, window, st);
            app.request_redraw(window);
        }
        CMD_SCRIPT_SAVE => {
            save_loaded_script(app, window, st);
            app.request_redraw(window);
        }
        CMD_SCRIPT_APPLY_PICK => {
            apply_pick_to_loaded_script(app, window, st);
            app.request_redraw(window);
        }
        CMD_OPEN_VIEWER_URL => {
            let url = app
                .models()
                .read(&st.viewer_url, |v| v.clone())
                .unwrap_or_default();
            if url.trim().is_empty() {
                push_log(app, &st.log_lines, "open viewer refused (empty url)");
                return;
            }
            app.push_effect(Effect::OpenUrl {
                url,
                target: None,
                rel: None,
            });
        }
        CMD_COPY_PACK_PATH => {
            let Some(path) = app
                .models()
                .read(&st.last_pack_path, |v| v.clone())
                .ok()
                .flatten()
            else {
                push_log(app, &st.log_lines, "copy pack path refused (no pack yet)");
                return;
            };
            app.push_effect(Effect::ClipboardSetText {
                text: path.to_string(),
            });
        }
        CMD_PACK_LAST_BUNDLE => {
            if let Err(err) = pack::start_pack_last_bundle(app, st) {
                push_log(app, &st.log_lines, &format!("pack refused: {err}"));
            }
            app.request_redraw(window);
        }
        CMD_SCRIPT_PUSH | CMD_SCRIPT_RUN | CMD_SCRIPT_RUN_AND_PACK => {
            if !ws::require_session_selected(app, st) {
                app.request_redraw(window);
                return;
            }
            let script_text = app
                .models()
                .read(&st.script_text, |v| v.clone())
                .unwrap_or_default();
            let Ok(script_value) = serde_json::from_str::<serde_json::Value>(&script_text) else {
                push_log(app, &st.log_lines, "script json parse failed");
                app.request_redraw(window);
                return;
            };
            if let Err(err) = validate_script_json_value(&script_value) {
                push_log(app, &st.log_lines, &format!("script invalid: {err}"));
                app.request_redraw(window);
                return;
            }

            let ty = match cmd.as_str() {
                CMD_SCRIPT_RUN | CMD_SCRIPT_RUN_AND_PACK => "script.run",
                _ => "script.push",
            };

            if cmd.as_str() == CMD_SCRIPT_RUN_AND_PACK {
                let _ = app
                    .models_mut()
                    .update(&st.script_pack_after_run, |v| *v = true);
            } else {
                let _ = app
                    .models_mut()
                    .update(&st.script_pack_after_run, |v| *v = false);
            }
            let _ = app.models_mut().update(&st.script_last_stage, |v| {
                *v = Some(UiScriptStageV1::Queued)
            });
            let _ = app
                .models_mut()
                .update(&st.script_last_step_index, |v| *v = None);
            let _ = app
                .models_mut()
                .update(&st.script_last_reason, |v| *v = None);
            let _ = app
                .models_mut()
                .update(&st.script_last_bundle_dir, |v| *v = None);
            match ty {
                "script.run" => st.devtools.script_run_value(None, script_value),
                _ => st.devtools.script_push_value(None, script_value),
            }
            app.push_effect(Effect::RequestAnimationFrame(window));
        }
        _ => {}
    }
}

fn refresh_script_library(app: &mut App, st: &mut State) {
    let scripts = script_studio::scan_script_library(&st.script_paths);
    let _ = app
        .models_mut()
        .update(&st.script_library, |v| *v = scripts.clone());

    let loaded_path = app
        .models()
        .read(&st.loaded_script_path, |v| v.clone())
        .ok()
        .flatten()
        .map(|s| PathBuf::from(s.as_ref()));

    let loaded_origin = loaded_path
        .as_ref()
        .and_then(|p| scripts.iter().find(|i| &i.path == p).map(|i| i.origin));
    let _ = app
        .models_mut()
        .update(&st.loaded_script_origin, |v| *v = loaded_origin);
}

fn fork_loaded_script(app: &mut App, window: AppWindowId, st: &mut State) {
    let origin = app
        .models()
        .read(&st.loaded_script_origin, |v| *v)
        .ok()
        .flatten();
    let path = app
        .models()
        .read(&st.loaded_script_path, |v| v.clone())
        .ok()
        .flatten()
        .map(|s| PathBuf::from(s.as_ref()));

    if origin != Some(script_studio::ScriptOrigin::WorkspaceTools) {
        push_log(
            app,
            &st.log_lines,
            "fork refused (load a tools/* script first)",
        );
        return;
    }
    let Some(path) = path else {
        push_log(app, &st.log_lines, "fork refused (no script loaded)");
        return;
    };
    let Some(file_name) = path.file_name().and_then(|s| s.to_str()) else {
        push_log(app, &st.log_lines, "fork refused (invalid file name)");
        return;
    };

    let item = script_studio::ScriptItem {
        origin: script_studio::ScriptOrigin::WorkspaceTools,
        file_name: Arc::from(file_name),
        path,
    };

    let forked = match script_studio::fork_script_to_user(&st.script_paths, &item) {
        Ok(item) => item,
        Err(err) => {
            push_log(app, &st.log_lines, &format!("fork failed: {err}"));
            return;
        }
    };

    refresh_script_library(app, st);
    let _ = app.models_mut().update(&st.script_text, |v| {
        *v = script_studio::load_script_text(&forked.path).unwrap_or_default()
    });
    let _ = app
        .models_mut()
        .update(&st.loaded_script_origin, |v| *v = Some(forked.origin));
    let _ = app.models_mut().update(&st.loaded_script_path, |v| {
        *v = Some(Arc::<str>::from(forked.path.to_string_lossy().to_string()))
    });

    app.push_effect(Effect::RequestAnimationFrame(window));
}

fn save_loaded_script(app: &mut App, window: AppWindowId, st: &mut State) {
    let origin = app
        .models()
        .read(&st.loaded_script_origin, |v| *v)
        .ok()
        .flatten();
    if origin != Some(script_studio::ScriptOrigin::UserLocal) {
        push_log(
            app,
            &st.log_lines,
            "save refused (fork into .fret/diag/scripts first)",
        );
        return;
    }

    let Some(path) = app
        .models()
        .read(&st.loaded_script_path, |v| v.clone())
        .ok()
        .flatten()
        .map(|s| PathBuf::from(s.as_ref()))
    else {
        push_log(app, &st.log_lines, "save refused (no script loaded)");
        return;
    };

    let text = app
        .models()
        .read(&st.script_text, |v| v.clone())
        .unwrap_or_default();
    if let Err(err) = script_studio::save_user_script(&st.script_paths, &path, &text) {
        push_log(app, &st.log_lines, &format!("save failed: {err}"));
        return;
    }

    refresh_script_library(app, st);
    app.push_effect(Effect::RequestAnimationFrame(window));
}

fn apply_pick_to_loaded_script(app: &mut App, window: AppWindowId, st: &mut State) {
    let pointer = app
        .models()
        .read(&st.script_apply_pointer, |v| v.clone())
        .unwrap_or_default();
    let script = app
        .models()
        .read(&st.script_text, |v| v.clone())
        .unwrap_or_default();
    let pick = app
        .models()
        .read(&st.last_pick_json, |v| v.clone())
        .unwrap_or_default();
    if pick.trim().is_empty() {
        push_log(
            app,
            &st.log_lines,
            "apply pick refused (no pick.result yet)",
        );
        return;
    }

    match script_studio::apply_pick_to_json_pointer(&script, &pointer, &pick) {
        Ok(updated) => {
            let _ = app.models_mut().update(&st.script_text, |v| *v = updated);
            app.push_effect(Effect::RequestAnimationFrame(window));
        }
        Err(err) => push_log(app, &st.log_lines, &format!("apply pick failed: {err}")),
    }
}

fn script_steps_len(script_text: &str) -> Option<usize> {
    let v: serde_json::Value = serde_json::from_str(script_text).ok()?;
    v.get("steps").and_then(|v| v.as_array()).map(|a| a.len())
}

fn script_summary_line(script_text: &str) -> (String, bool) {
    let v: serde_json::Value = match serde_json::from_str(script_text) {
        Ok(v) => v,
        Err(err) => return (format!("parse_error: {err}"), false),
    };

    let schema = match validate_script_json_value(&v) {
        Ok(schema) => schema,
        Err(err) => return (format!("invalid: {err}"), false),
    };

    let steps = v.get("steps").and_then(|v| v.as_array()).map(|a| a.len());
    let steps = steps
        .map(|n| n.to_string())
        .unwrap_or_else(|| "<missing>".to_string());
    (format!("ok schema_version={schema} steps={steps}"), true)
}

fn validate_script_json_value(script: &serde_json::Value) -> Result<u32, String> {
    let schema_version = script
        .get("schema_version")
        .and_then(|v| v.as_u64())
        .ok_or_else(|| "missing schema_version".to_string())?;
    let schema_version = schema_version.min(u32::MAX as u64) as u32;

    match schema_version {
        1 => {
            let parsed: UiActionScriptV1 =
                serde_json::from_value(script.clone()).map_err(|e| e.to_string())?;
            if parsed.schema_version != 1 {
                return Err("schema_version mismatch".to_string());
            }
            Ok(1)
        }
        2 => {
            let parsed: UiActionScriptV2 =
                serde_json::from_value(script.clone()).map_err(|e| e.to_string())?;
            if parsed.schema_version != 2 {
                return Err("schema_version mismatch".to_string());
            }
            Ok(2)
        }
        other => Err(format!("unsupported schema_version: {other}")),
    }
}

#[derive(Clone)]
struct StepTemplate {
    label: &'static str,
    step: serde_json::Value,
}

fn infer_script_schema_version(script_text: &str) -> Option<u32> {
    let v: serde_json::Value = serde_json::from_str(script_text).ok()?;
    let schema = v.get("schema_version").and_then(|v| v.as_u64())?;
    Some(schema.min(u32::MAX as u64) as u32)
}

fn placeholder_selector_value() -> serde_json::Value {
    serde_json::json!({
        "kind": "test_id",
        "id": "TODO",
    })
}

fn placeholder_predicate_value() -> serde_json::Value {
    serde_json::json!({
        "kind": "exists",
        "target": placeholder_selector_value(),
    })
}

fn primary_pointer_suffix_for_step_json(step: &serde_json::Value) -> Option<&'static str> {
    let obj = step.as_object()?;
    let has = |k: &str| obj.contains_key(k);
    if has("target") {
        return Some("/target");
    }
    if has("predicate") {
        return Some("/predicate");
    }
    if has("container") {
        return Some("/container");
    }
    if has("from") {
        return Some("/from");
    }
    if has("to") {
        return Some("/to");
    }
    if has("menu") {
        return Some("/menu");
    }
    if has("item") {
        return Some("/item");
    }
    if has("path") {
        return Some("/path/0");
    }
    None
}

fn step_templates_for_schema(schema_version: u32) -> Vec<StepTemplate> {
    let target = placeholder_selector_value();
    let predicate = placeholder_predicate_value();

    let v1 = vec![
        StepTemplate {
            label: "click",
            step: serde_json::json!({
                "type": "click",
                "target": target,
                "button": "left",
            }),
        },
        StepTemplate {
            label: "move_pointer",
            step: serde_json::json!({
                "type": "move_pointer",
                "target": placeholder_selector_value(),
            }),
        },
        StepTemplate {
            label: "wheel",
            step: serde_json::json!({
                "type": "wheel",
                "target": placeholder_selector_value(),
                "delta_x": 0.0,
                "delta_y": -120.0,
            }),
        },
        StepTemplate {
            label: "press_key",
            step: serde_json::json!({
                "type": "press_key",
                "key": "Enter",
                "modifiers": { "shift": false, "ctrl": false, "alt": false, "meta": false },
                "repeat": false,
            }),
        },
        StepTemplate {
            label: "type_text",
            step: serde_json::json!({
                "type": "type_text",
                "text": "TODO",
            }),
        },
        StepTemplate {
            label: "wait_frames",
            step: serde_json::json!({
                "type": "wait_frames",
                "n": 30,
            }),
        },
        StepTemplate {
            label: "wait_until",
            step: serde_json::json!({
                "type": "wait_until",
                "predicate": predicate,
                "timeout_frames": 180,
            }),
        },
        StepTemplate {
            label: "assert",
            step: serde_json::json!({
                "type": "assert",
                "predicate": placeholder_predicate_value(),
            }),
        },
        StepTemplate {
            label: "capture_bundle",
            step: serde_json::json!({
                "type": "capture_bundle",
                "label": "devtools",
            }),
        },
        StepTemplate {
            label: "capture_screenshot",
            step: serde_json::json!({
                "type": "capture_screenshot",
                "label": "devtools",
                "timeout_frames": 300,
            }),
        },
        StepTemplate {
            label: "reset_diagnostics",
            step: serde_json::json!({
                "type": "reset_diagnostics",
            }),
        },
    ];

    if schema_version <= 1 {
        return v1;
    }

    let mut v2 = Vec::new();
    v2.extend(v1);
    v2.push(StepTemplate {
        label: "press_shortcut",
        step: serde_json::json!({
            "type": "press_shortcut",
            "shortcut": "Ctrl+P",
            "repeat": false,
        }),
    });
    v2.push(StepTemplate {
        label: "move_pointer_sweep",
        step: serde_json::json!({
            "type": "move_pointer_sweep",
            "target": placeholder_selector_value(),
            "delta_x": 0.0,
            "delta_y": 40.0,
            "steps": 8,
            "frames_per_step": 1,
        }),
    });
    v2.push(StepTemplate {
        label: "click_stable",
        step: serde_json::json!({
            "type": "click_stable",
            "target": placeholder_selector_value(),
            "button": "left",
            "stable_frames": 2,
            "max_move_px": 1.0,
            "timeout_frames": 180,
        }),
    });
    v2.push(StepTemplate {
        label: "ensure_visible",
        step: serde_json::json!({
            "type": "ensure_visible",
            "target": placeholder_selector_value(),
            "within_window": true,
            "padding_px": 0.0,
            "timeout_frames": 180,
        }),
    });
    v2.push(StepTemplate {
        label: "scroll_into_view",
        step: serde_json::json!({
            "type": "scroll_into_view",
            "container": placeholder_selector_value(),
            "target": placeholder_selector_value(),
            "delta_x": 0.0,
            "delta_y": -120.0,
            "require_fully_within_container": false,
            "require_fully_within_window": false,
            "padding_px": 0.0,
            "padding_insets_px": null,
            "timeout_frames": 180,
        }),
    });
    v2.push(StepTemplate {
        label: "type_text_into",
        step: serde_json::json!({
            "type": "type_text_into",
            "target": placeholder_selector_value(),
            "text": "TODO",
            "timeout_frames": 180,
        }),
    });
    v2.push(StepTemplate {
        label: "menu_select",
        step: serde_json::json!({
            "type": "menu_select",
            "menu": placeholder_selector_value(),
            "item": placeholder_selector_value(),
            "timeout_frames": 180,
        }),
    });
    v2.push(StepTemplate {
        label: "menu_select_path",
        step: serde_json::json!({
            "type": "menu_select_path",
            "path": [placeholder_selector_value()],
            "timeout_frames": 180,
        }),
    });
    v2.push(StepTemplate {
        label: "drag_to",
        step: serde_json::json!({
            "type": "drag_to",
            "from": placeholder_selector_value(),
            "to": placeholder_selector_value(),
            "button": "left",
            "steps": 8,
            "timeout_frames": 180,
        }),
    });
    v2.push(StepTemplate {
        label: "set_slider_value",
        step: serde_json::json!({
            "type": "set_slider_value",
            "target": placeholder_selector_value(),
            "value": 50.0,
            "min": 0.0,
            "max": 100.0,
            "epsilon": 0.5,
            "timeout_frames": 180,
            "drag_steps": 8,
        }),
    });
    v2.push(StepTemplate {
        label: "set_window_inner_size",
        step: serde_json::json!({
            "type": "set_window_inner_size",
            "width_px": 1280.0,
            "height_px": 720.0,
        }),
    });

    v2
}

fn selector_fields(cx: &mut ElementContext<'_, App>, st: &State, kind: &str) -> AnyElement {
    let test_id = shadcn::Input::new(st.script_selector_test_id.clone())
        .a11y_label("test_id")
        .placeholder("button.ok")
        .into_element(cx);
    let role = shadcn::Input::new(st.script_selector_role.clone())
        .a11y_label("role")
        .placeholder("button")
        .into_element(cx);
    let name = shadcn::Input::new(st.script_selector_name.clone())
        .a11y_label("name")
        .placeholder("OK")
        .into_element(cx);
    let ancestors = shadcn::Textarea::new(st.script_selector_ancestors.clone())
        .a11y_label("ancestors (role:name per line)")
        .min_height(Px(120.0))
        .into_element(cx);
    let node_id = shadcn::Input::new(st.script_selector_node_id.clone())
        .a11y_label("node_id")
        .placeholder("123")
        .into_element(cx);
    let element_id = shadcn::Input::new(st.script_selector_element_id.clone())
        .a11y_label("global_element_id")
        .placeholder("123")
        .into_element(cx);

    match kind {
        "test_id" => fret_ui_kit::declarative::stack::vstack(
            cx,
            fret_ui_kit::declarative::stack::VStackProps::default()
                .gap_y(fret_ui_kit::Space::N1)
                .layout(fret_ui_kit::LayoutRefinement::default().w_full()),
            |_cx| [test_id],
        ),
        "role_and_name" => fret_ui_kit::declarative::stack::vstack(
            cx,
            fret_ui_kit::declarative::stack::VStackProps::default()
                .gap_y(fret_ui_kit::Space::N1)
                .layout(fret_ui_kit::LayoutRefinement::default().w_full()),
            |_cx| [role, name],
        ),
        "role_and_path" => fret_ui_kit::declarative::stack::vstack(
            cx,
            fret_ui_kit::declarative::stack::VStackProps::default()
                .gap_y(fret_ui_kit::Space::N1)
                .layout(fret_ui_kit::LayoutRefinement::default().w_full()),
            |_cx| [role, name, ancestors],
        ),
        "node_id" => fret_ui_kit::declarative::stack::vstack(
            cx,
            fret_ui_kit::declarative::stack::VStackProps::default()
                .gap_y(fret_ui_kit::Space::N1)
                .layout(fret_ui_kit::LayoutRefinement::default().w_full()),
            |_cx| [node_id],
        ),
        "global_element_id" => fret_ui_kit::declarative::stack::vstack(
            cx,
            fret_ui_kit::declarative::stack::VStackProps::default()
                .gap_y(fret_ui_kit::Space::N1)
                .layout(fret_ui_kit::LayoutRefinement::default().w_full()),
            |_cx| [element_id],
        ),
        _ => cx.text("unknown selector kind"),
    }
}

fn selector_value_from_models(
    cx: &mut ElementContext<'_, App>,
    st: &State,
    kind: &str,
) -> serde_json::Value {
    let test_id = cx
        .app
        .models()
        .read(&st.script_selector_test_id, |v| v.clone())
        .unwrap_or_default();
    let role = cx
        .app
        .models()
        .read(&st.script_selector_role, |v| v.clone())
        .unwrap_or_default();
    let name = cx
        .app
        .models()
        .read(&st.script_selector_name, |v| v.clone())
        .unwrap_or_default();
    let ancestors_text = cx
        .app
        .models()
        .read(&st.script_selector_ancestors, |v| v.clone())
        .unwrap_or_default();
    let node_id = cx
        .app
        .models()
        .read(&st.script_selector_node_id, |v| v.clone())
        .unwrap_or_default();
    let element_id = cx
        .app
        .models()
        .read(&st.script_selector_element_id, |v| v.clone())
        .unwrap_or_default();

    match kind {
        "test_id" => serde_json::json!({
            "kind": "test_id",
            "id": test_id.trim(),
        }),
        "role_and_name" => serde_json::json!({
            "kind": "role_and_name",
            "role": role.trim(),
            "name": name.trim(),
        }),
        "role_and_path" => serde_json::json!({
            "kind": "role_and_path",
            "role": role.trim(),
            "name": name.trim(),
            "ancestors": parse_ancestors_lines(&ancestors_text),
        }),
        "node_id" => serde_json::json!({
            "kind": "node_id",
            "node": node_id.trim().parse::<u64>().unwrap_or(0),
        }),
        "global_element_id" => serde_json::json!({
            "kind": "global_element_id",
            "element": element_id.trim().parse::<u64>().unwrap_or(0),
        }),
        _ => placeholder_selector_value(),
    }
}

fn predicate_fields(cx: &mut ElementContext<'_, App>, st: &State, kind: &str) -> AnyElement {
    let role = shadcn::Input::new(st.script_predicate_role.clone())
        .a11y_label("role")
        .placeholder("button")
        .into_element(cx);
    let checked = shadcn::Checkbox::new(st.script_predicate_checked.clone())
        .a11y_label("checked")
        .into_element(cx);
    let padding = shadcn::Input::new(st.script_predicate_padding_px.clone())
        .a11y_label("padding_px")
        .placeholder("0")
        .into_element(cx);
    let eps = shadcn::Input::new(st.script_predicate_eps_px.clone())
        .a11y_label("eps_px")
        .placeholder("0")
        .into_element(cx);
    let min_w = shadcn::Input::new(st.script_predicate_min_w_px.clone())
        .a11y_label("min_w_px")
        .placeholder("0")
        .into_element(cx);
    let min_h = shadcn::Input::new(st.script_predicate_min_h_px.clone())
        .a11y_label("min_h_px")
        .placeholder("0")
        .into_element(cx);

    match kind {
        "role_is" => role,
        "checked_is" => checked,
        "barrier_roots" => {
            let barrier_root_items = [
                shadcn::SelectItem::new("any", "any"),
                shadcn::SelectItem::new("none", "none"),
                shadcn::SelectItem::new("some", "some"),
            ];
            let focus_root_items = [
                shadcn::SelectItem::new("any", "any"),
                shadcn::SelectItem::new("none", "none"),
                shadcn::SelectItem::new("some", "some"),
            ];
            let require_items = [
                shadcn::SelectItem::new("unset", "unset"),
                shadcn::SelectItem::new("true", "true"),
                shadcn::SelectItem::new("false", "false"),
            ];

            let barrier_root = shadcn::Select::new(
                st.script_predicate_barrier_root.clone(),
                st.script_predicate_barrier_root_open.clone(),
            )
            .placeholder("barrier_root")
            .items(barrier_root_items)
            .refine_layout(fret_ui_kit::LayoutRefinement::default().w_full())
            .into_element(cx);

            let focus_root = shadcn::Select::new(
                st.script_predicate_focus_barrier_root.clone(),
                st.script_predicate_focus_barrier_root_open.clone(),
            )
            .placeholder("focus_barrier_root")
            .items(focus_root_items)
            .refine_layout(fret_ui_kit::LayoutRefinement::default().w_full())
            .into_element(cx);

            let require_equal = shadcn::Select::new(
                st.script_predicate_require_equal.clone(),
                st.script_predicate_require_equal_open.clone(),
            )
            .placeholder("require_equal")
            .items(require_items)
            .refine_layout(fret_ui_kit::LayoutRefinement::default().w_full())
            .into_element(cx);

            let other_selector =
                shadcn::Textarea::new(st.script_predicate_other_selector_json.clone())
                    .a11y_label("other selector (optional)")
                    .min_height(Px(96.0))
                    .into_element(cx);

            fret_ui_kit::declarative::stack::vstack(
                cx,
                fret_ui_kit::declarative::stack::VStackProps::default()
                    .gap_y(fret_ui_kit::Space::N1)
                    .layout(fret_ui_kit::LayoutRefinement::default().w_full()),
                |_cx| [barrier_root, focus_root, require_equal, other_selector],
            )
        }
        "bounds_within_window" => fret_ui_kit::declarative::stack::vstack(
            cx,
            fret_ui_kit::declarative::stack::VStackProps::default()
                .gap_y(fret_ui_kit::Space::N1)
                .layout(fret_ui_kit::LayoutRefinement::default().w_full()),
            |_cx| [padding, eps],
        ),
        "bounds_min_size" => fret_ui_kit::declarative::stack::vstack(
            cx,
            fret_ui_kit::declarative::stack::VStackProps::default()
                .gap_y(fret_ui_kit::Space::N1)
                .layout(fret_ui_kit::LayoutRefinement::default().w_full()),
            |_cx| [min_w, min_h, eps],
        ),
        "bounds_non_overlapping"
        | "bounds_overlapping"
        | "bounds_overlapping_x"
        | "bounds_overlapping_y" => {
            let other_selector =
                shadcn::Textarea::new(st.script_predicate_other_selector_json.clone())
                    .a11y_label("selector B (JSON)")
                    .min_height(Px(120.0))
                    .into_element(cx);
            fret_ui_kit::declarative::stack::vstack(
                cx,
                fret_ui_kit::declarative::stack::VStackProps::default()
                    .gap_y(fret_ui_kit::Space::N1)
                    .layout(fret_ui_kit::LayoutRefinement::default().w_full()),
                |_cx| [eps, other_selector],
            )
        }
        _ => cx.text(""),
    }
}

fn predicate_value_from_models(
    cx: &mut ElementContext<'_, App>,
    st: &State,
    kind: &str,
    selector: serde_json::Value,
) -> serde_json::Value {
    let role = cx
        .app
        .models()
        .read(&st.script_predicate_role, |v| v.clone())
        .unwrap_or_default();
    let other_selector_json = cx
        .app
        .models()
        .read(&st.script_predicate_other_selector_json, |v| v.clone())
        .unwrap_or_default();
    let checked = cx
        .app
        .models()
        .read(&st.script_predicate_checked, |v| *v)
        .unwrap_or(false);
    let padding_px = parse_f32_model(cx, &st.script_predicate_padding_px);
    let eps_px = parse_f32_model(cx, &st.script_predicate_eps_px);
    let min_w_px = parse_f32_model(cx, &st.script_predicate_min_w_px);
    let min_h_px = parse_f32_model(cx, &st.script_predicate_min_h_px);

    let other_selector = serde_json::from_str::<serde_json::Value>(&other_selector_json)
        .ok()
        .unwrap_or_else(placeholder_selector_value);

    match kind {
        "exists" => serde_json::json!({
            "kind": "exists",
            "target": selector,
        }),
        "not_exists" => serde_json::json!({
            "kind": "not_exists",
            "target": selector,
        }),
        "focus_is" => serde_json::json!({
            "kind": "focus_is",
            "target": selector,
        }),
        "role_is" => serde_json::json!({
            "kind": "role_is",
            "target": selector,
            "role": role.trim(),
        }),
        "checked_is" => serde_json::json!({
            "kind": "checked_is",
            "target": selector,
            "checked": checked,
        }),
        "checked_is_none" => serde_json::json!({
            "kind": "checked_is_none",
            "target": selector,
        }),
        "barrier_roots" => {
            let barrier_root = cx
                .app
                .models()
                .read(&st.script_predicate_barrier_root, |v| v.clone())
                .ok()
                .flatten()
                .unwrap_or_else(|| Arc::<str>::from("any"));
            let focus_barrier_root = cx
                .app
                .models()
                .read(&st.script_predicate_focus_barrier_root, |v| v.clone())
                .ok()
                .flatten()
                .unwrap_or_else(|| Arc::<str>::from("any"));
            let require_equal = cx
                .app
                .models()
                .read(&st.script_predicate_require_equal, |v| v.clone())
                .ok()
                .flatten()
                .unwrap_or_else(|| Arc::<str>::from("unset"));

            let mut obj = serde_json::Map::new();
            obj.insert(
                "kind".to_string(),
                serde_json::Value::String("barrier_roots".to_string()),
            );
            obj.insert(
                "barrier_root".to_string(),
                serde_json::Value::String(barrier_root.to_string()),
            );
            obj.insert(
                "focus_barrier_root".to_string(),
                serde_json::Value::String(focus_barrier_root.to_string()),
            );
            if require_equal.as_ref() == "true" {
                obj.insert("require_equal".to_string(), serde_json::Value::Bool(true));
            } else if require_equal.as_ref() == "false" {
                obj.insert("require_equal".to_string(), serde_json::Value::Bool(false));
            }
            serde_json::Value::Object(obj)
        }
        "visible_in_window" => serde_json::json!({
            "kind": "visible_in_window",
            "target": selector,
        }),
        "bounds_within_window" => serde_json::json!({
            "kind": "bounds_within_window",
            "target": selector,
            "padding_px": padding_px,
            "eps_px": eps_px,
        }),
        "bounds_min_size" => serde_json::json!({
            "kind": "bounds_min_size",
            "target": selector,
            "min_w_px": min_w_px,
            "min_h_px": min_h_px,
            "eps_px": eps_px,
        }),
        "bounds_non_overlapping" => serde_json::json!({
            "kind": "bounds_non_overlapping",
            "a": selector,
            "b": other_selector,
            "eps_px": eps_px,
        }),
        "bounds_overlapping" => serde_json::json!({
            "kind": "bounds_overlapping",
            "a": selector,
            "b": other_selector,
            "eps_px": eps_px,
        }),
        "bounds_overlapping_x" => serde_json::json!({
            "kind": "bounds_overlapping_x",
            "a": selector,
            "b": other_selector,
            "eps_px": eps_px,
        }),
        "bounds_overlapping_y" => serde_json::json!({
            "kind": "bounds_overlapping_y",
            "a": selector,
            "b": other_selector,
            "eps_px": eps_px,
        }),
        _ => placeholder_predicate_value(),
    }
}

fn parse_f32_model(cx: &mut ElementContext<'_, App>, m: &Model<String>) -> f32 {
    cx.app
        .models()
        .read(m, |v| v.trim().parse::<f32>().ok())
        .ok()
        .flatten()
        .unwrap_or(0.0)
}

fn parse_ancestors_lines(text: &str) -> Vec<serde_json::Value> {
    let mut out = Vec::new();
    for line in text.lines() {
        let line = line.trim();
        if line.is_empty() {
            continue;
        }
        let Some((role, name)) = line.split_once(':') else {
            continue;
        };
        let role = role.trim();
        let name = name.trim();
        if role.is_empty() || name.is_empty() {
            continue;
        }
        out.push(serde_json::json!({
            "role": role,
            "name": name,
        }));
    }
    out
}

fn is_abs_path(s: &str) -> bool {
    if s.starts_with('/') || s.starts_with('\\') {
        return true;
    }
    let bytes = s.as_bytes();
    bytes.len() >= 3 && bytes[1] == b':' && (bytes[2] == b'\\' || bytes[2] == b'/')
}

fn repo_root_from_script_paths(paths: &script_studio::ScriptPaths) -> PathBuf {
    paths
        .workspace_tools_dir
        .parent()
        .and_then(|p| p.parent())
        .map(|p| p.to_path_buf())
        .unwrap_or_else(|| PathBuf::from("."))
}

fn now_unix_ms() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .ok()
        .map(|d| d.as_millis().min(u128::from(u64::MAX)) as u64)
        .unwrap_or(0)
}

fn push_log(app: &mut App, model: &Model<Vec<Arc<str>>>, line: &str) {
    let line = Arc::<str>::from(line);
    let _ = app.models_mut().update(model, |v| {
        v.push(line);
        if v.len() > 2000 {
            let drain = v.len().saturating_sub(2000);
            v.drain(0..drain);
        }
    });
}

fn env_u16(key: &str) -> Option<u16> {
    std::env::var(key).ok().and_then(|v| v.parse().ok())
}

fn env_transport_kind(key: &str) -> Option<DiagTransportKind> {
    let v = std::env::var(key).ok()?;
    let v = v.trim().to_lowercase();
    match v.as_str() {
        "ws" | "websocket" => Some(DiagTransportKind::WebSocket),
        "fs" | "filesystem" => Some(DiagTransportKind::FileSystem),
        _ => None,
    }
}
