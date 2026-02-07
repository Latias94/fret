use std::net::{IpAddr, Ipv4Addr, SocketAddr};
use std::path::PathBuf;
use std::sync::Arc;

use fret_app::{App, CommandId, Effect};
use fret_bootstrap::BootstrapBuilder;
use fret_bootstrap::ui_app_driver::{UiAppDriver, ViewElements};
use fret_core::{AppWindowId, Px, UiServices};
use fret_diag_protocol::{
    DevtoolsSessionDescriptorV1, DiagTransportMessageV1, UiActionScriptV1, UiActionScriptV2,
    UiScriptStageV1,
};
use fret_diag_ws::client::{ClientKindV1, DevtoolsWsClient, DevtoolsWsClientConfig};
use fret_diag_ws::server::{DevtoolsWsServer, DevtoolsWsServerConfig};
use fret_runtime::Model;
use fret_ui::element::AnyElement;
use fret_ui::elements::ContinuousFrames;
use fret_ui::{ElementContext, Invalidation, Theme};
use fret_ui_shadcn as shadcn;

mod pack;
mod script_studio;
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
    ws_port: u16,
    ws_url: Arc<str>,
    token: Arc<str>,
}

struct State {
    cfg: DevtoolsConfig,

    panel_fractions: Model<Vec<f32>>,
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
    last_script_result_json: Model<String>,
    last_bundle_json: Model<String>,
    last_screenshot_json: Model<String>,
    log_lines: Model<Vec<Arc<str>>>,

    client: DevtoolsWsClient,
    applied_session_id: Option<Arc<str>>,

    pack_tx: std::sync::mpsc::Sender<pack::PackJobResult>,
    pack_rx: std::sync::mpsc::Receiver<pack::PackJobResult>,
}

fn main() -> anyhow::Result<()> {
    let port = env_u16("FRET_DEVTOOLS_WS_PORT").unwrap_or(7331);
    let token =
        std::env::var("FRET_DEVTOOLS_TOKEN").unwrap_or_else(|_| uuid::Uuid::new_v4().to_string());
    let bind = SocketAddr::new(IpAddr::V4(Ipv4Addr::LOCALHOST), port);

    eprintln!("fret-devtools: bind={bind} token={token}");
    eprintln!("fret-devtools: url=ws://127.0.0.1:{port}/?fret_devtools_token={token}");

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

    let script_text = app.models_mut().insert(String::new());
    let script_last_stage = app.models_mut().insert(None::<UiScriptStageV1>);
    let script_last_step_index = app.models_mut().insert(None::<u32>);
    let script_last_reason = app.models_mut().insert(None::<Arc<str>>);
    let script_last_bundle_dir = app.models_mut().insert(None::<Arc<str>>);
    let script_pack_after_run = app.models_mut().insert(false);

    let target_out_dir = app.models_mut().insert(None::<Arc<str>>);
    let last_bundle_dir_abs = app.models_mut().insert(None::<Arc<str>>);
    let last_bundle_dump_exported_unix_ms = app.models_mut().insert(None::<u64>);
    let last_bundle_dump_bundle_json = app.models_mut().insert(None::<Arc<str>>);
    let last_pack_path = app.models_mut().insert(None::<Arc<str>>);
    let pack_in_flight = app.models_mut().insert(false);
    let pack_last_error = app.models_mut().insert(None::<Arc<str>>);
    let viewer_url = app.models_mut().insert("http://localhost:5173".to_string());
    let last_pick_json = app.models_mut().insert(String::new());
    let last_script_result_json = app.models_mut().insert(String::new());
    let last_bundle_json = app.models_mut().insert(String::new());
    let last_screenshot_json = app.models_mut().insert(String::new());
    let log_lines = app.models_mut().insert(Vec::<Arc<str>>::new());

    let mut client_cfg =
        DevtoolsWsClientConfig::with_defaults(cfg.ws_url.to_string(), cfg.token.to_string());
    client_cfg.client_kind = ClientKindV1::Tooling;
    client_cfg.capabilities = vec![
        "inspect".to_string(),
        "pick".to_string(),
        "scripts".to_string(),
        "bundles".to_string(),
    ];
    let client = DevtoolsWsClient::connect_native(client_cfg)
        .expect("devtools ws client connect must succeed");

    let (pack_tx, pack_rx) = pack::new_pack_channel();

    let mut st = State {
        cfg,
        panel_fractions,
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
        last_script_result_json,
        last_bundle_json,
        last_screenshot_json,
        log_lines,
        client,
        applied_session_id: None,
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
    cx.observe_model(&st.last_script_result_json, Invalidation::Paint);
    cx.observe_model(&st.last_bundle_json, Invalidation::Paint);
    cx.observe_model(&st.last_screenshot_json, Invalidation::Paint);
    cx.observe_model(&st.log_lines, Invalidation::Paint);

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

    shadcn::Card::new([
        shadcn::CardHeader::new([
            shadcn::CardTitle::new("Events").into_element(cx),
            shadcn::CardDescription::new("Latest WS messages (tail)").into_element(cx),
        ])
        .into_element(cx),
        shadcn::CardContent::new([list]).into_element(cx),
    ])
    .into_element(cx)
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

    let tabs = shadcn::Tabs::new(st.details_tab.clone())
        .refine_layout(fret_ui_kit::LayoutRefinement::default().w_full())
        .items([
            shadcn::TabsItem::new("pick", "Pick", [text_blob(cx, pick)]),
            shadcn::TabsItem::new("script", "Script", [text_blob(cx, script)]),
            shadcn::TabsItem::new("bundle", "Bundle", [text_blob(cx, bundle)]),
            shadcn::TabsItem::new("screenshot", "Screenshot", [text_blob(cx, screenshot)]),
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
            st.client.send(DiagTransportMessageV1 {
                schema_version: 1,
                r#type: "inspect.set".to_string(),
                session_id: None,
                request_id: None,
                payload: serde_json::json!({
                    "enabled": enabled,
                    "consume_clicks": consume_clicks,
                }),
            });
            app.push_effect(Effect::RequestAnimationFrame(window));
        }
        CMD_PICK_ARM => {
            if !ws::require_session_selected(app, st) {
                app.request_redraw(window);
                return;
            }
            st.client
                .send_type_payload("pick.arm", serde_json::json!({}));
            app.push_effect(Effect::RequestAnimationFrame(window));
        }
        CMD_BUNDLE_DUMP => {
            if !ws::require_session_selected(app, st) {
                app.request_redraw(window);
                return;
            }
            st.client
                .send_type_payload("bundle.dump", serde_json::json!({ "label": "devtools" }));
            app.push_effect(Effect::RequestAnimationFrame(window));
        }
        CMD_SCREENSHOT_REQUEST => {
            if !ws::require_session_selected(app, st) {
                app.request_redraw(window);
                return;
            }
            st.client.send_type_payload(
                "screenshot.request",
                serde_json::json!({ "label": "devtools", "timeout_frames": 300 }),
            );
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
            app.push_effect(Effect::OpenUrl { url });
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
            st.client.send_type_payload(
                ty,
                serde_json::json!({
                    "script": script_value,
                }),
            );
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
