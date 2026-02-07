use std::net::{IpAddr, Ipv4Addr, SocketAddr};
use std::path::PathBuf;
use std::sync::Arc;

use fret_app::{App, CommandId, Effect};
use fret_bootstrap::BootstrapBuilder;
use fret_bootstrap::ui_app_driver::{UiAppDriver, ViewElements};
use fret_core::{AppWindowId, Px, UiServices};
use fret_diag_protocol::{
    DevtoolsSessionAddedV1, DevtoolsSessionDescriptorV1, DevtoolsSessionListV1,
    DevtoolsSessionRemovedV1, DiagTransportMessageV1,
};
use fret_diag_ws::client::{ClientKindV1, DevtoolsWsClient, DevtoolsWsClientConfig};
use fret_diag_ws::server::{DevtoolsWsServer, DevtoolsWsServerConfig};
use fret_runtime::Model;
use fret_ui::element::AnyElement;
use fret_ui::elements::ContinuousFrames;
use fret_ui::{ElementContext, Invalidation, Theme};
use fret_ui_shadcn as shadcn;

mod script_studio;

const CMD_COPY_WS_URL: &str = "fret.devtools.copy_ws_url";
const CMD_COPY_TOKEN: &str = "fret.devtools.copy_token";
const CMD_INSPECT_ENABLE: &str = "fret.devtools.inspect_enable";
const CMD_INSPECT_DISABLE: &str = "fret.devtools.inspect_disable";
const CMD_PICK_ARM: &str = "fret.devtools.pick_arm";
const CMD_BUNDLE_DUMP: &str = "fret.devtools.bundle_dump";
const CMD_SCREENSHOT_REQUEST: &str = "fret.devtools.screenshot_request";
const CMD_SCRIPT_PUSH: &str = "fret.devtools.script_push";
const CMD_SCRIPT_RUN: &str = "fret.devtools.script_run";
const CMD_SCRIPTS_REFRESH: &str = "fret.devtools.scripts.refresh";
const CMD_SCRIPT_FORK: &str = "fret.devtools.script.fork";
const CMD_SCRIPT_SAVE: &str = "fret.devtools.script.save";
const CMD_SCRIPT_APPLY_PICK: &str = "fret.devtools.script.apply_pick";

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

    last_pick_json: Model<String>,
    last_script_result_json: Model<String>,
    last_bundle_json: Model<String>,
    last_screenshot_json: Model<String>,
    log_lines: Model<Vec<Arc<str>>>,

    client: DevtoolsWsClient,
    applied_session_id: Option<Arc<str>>,
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
        last_pick_json,
        last_script_result_json,
        last_bundle_json,
        last_screenshot_json,
        log_lines,
        client,
        applied_session_id: None,
    };

    refresh_script_library(app, &mut st);
    st
}

fn view(cx: &mut ElementContext<'_, App>, st: &mut State) -> ViewElements {
    drain_ws_messages(cx.app, st);
    sync_selected_session_to_client(cx.app, st);

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

    let pointer_input = shadcn::Input::new(st.script_apply_pointer.clone())
        .a11y_label("JSON pointer")
        .placeholder("/steps/0/target")
        .into_element(cx);

    let textarea = shadcn::Textarea::new(st.script_text.clone())
        .a11y_label("Script JSON")
        .min_height(Px(360.0))
        .refine_layout(fret_ui_kit::LayoutRefinement::default().w_full().h_full())
        .into_element(cx);

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
                    .disabled(!has_session)
                    .on_click(CMD_SCRIPT_PUSH)
                    .into_element(cx),
                shadcn::Button::new("Run Script")
                    .variant(shadcn::ButtonVariant::Default)
                    .size(shadcn::ButtonSize::Sm)
                    .disabled(!has_session)
                    .on_click(CMD_SCRIPT_RUN)
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
            apply_row,
            cx.text(loaded_line),
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
    sync_selected_session_to_client(app, st);

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
            if !require_session_selected(app, st) {
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
            if !require_session_selected(app, st) {
                app.request_redraw(window);
                return;
            }
            st.client
                .send_type_payload("pick.arm", serde_json::json!({}));
            app.push_effect(Effect::RequestAnimationFrame(window));
        }
        CMD_BUNDLE_DUMP => {
            if !require_session_selected(app, st) {
                app.request_redraw(window);
                return;
            }
            st.client
                .send_type_payload("bundle.dump", serde_json::json!({ "label": "devtools" }));
            app.push_effect(Effect::RequestAnimationFrame(window));
        }
        CMD_SCREENSHOT_REQUEST => {
            if !require_session_selected(app, st) {
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
        CMD_SCRIPT_PUSH | CMD_SCRIPT_RUN => {
            if !require_session_selected(app, st) {
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
            let ty = if cmd.as_str() == CMD_SCRIPT_RUN {
                "script.run"
            } else {
                "script.push"
            };
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

fn require_session_selected(app: &mut App, st: &State) -> bool {
    let selected = app
        .models()
        .read(&st.selected_session_id, |v| v.clone())
        .ok()
        .flatten();
    if selected.is_some() {
        return true;
    }
    push_log(
        app,
        &st.log_lines,
        "no session selected (connect an app or pick a session)",
    );
    false
}

fn drain_ws_messages(app: &mut App, st: &mut State) {
    let mut drained_any = false;
    while let Some(msg) = st.client.try_recv() {
        drained_any = true;

        let ty = msg.r#type.clone();
        let compact = match msg.session_id.as_deref() {
            Some(s) => format!("type={ty} session_id={s}"),
            None => format!("type={ty}"),
        };
        push_log(app, &st.log_lines, &compact);

        match ty.as_str() {
            "session.list" => {
                if let Ok(parsed) = serde_json::from_value::<DevtoolsSessionListV1>(msg.payload) {
                    let sessions = parsed.sessions;
                    let _ = app.models_mut().update(&st.sessions, |v| *v = sessions);
                    ensure_session_selection_is_valid(app, st);
                }
            }
            "session.added" => {
                if let Ok(parsed) = serde_json::from_value::<DevtoolsSessionAddedV1>(msg.payload) {
                    let _ = app.models_mut().update(&st.sessions, |v| {
                        if let Some(pos) = v
                            .iter()
                            .position(|s| s.session_id == parsed.session.session_id)
                        {
                            v[pos] = parsed.session;
                        } else {
                            v.push(parsed.session);
                        }
                    });
                    ensure_session_selection_is_valid(app, st);
                }
            }
            "session.removed" => {
                if let Ok(parsed) = serde_json::from_value::<DevtoolsSessionRemovedV1>(msg.payload)
                {
                    let _ = app.models_mut().update(&st.sessions, |v| {
                        v.retain(|s| s.session_id != parsed.session_id);
                    });
                    ensure_session_selection_is_valid(app, st);
                }
            }
            "pick.result" => {
                if !message_matches_selected_session(app, st, &msg) {
                    continue;
                }
                if let Ok(text) = serde_json::to_string_pretty(&msg.payload) {
                    let _ = app.models_mut().update(&st.last_pick_json, |v| *v = text);
                }
            }
            "script.result" => {
                if !message_matches_selected_session(app, st, &msg) {
                    continue;
                }
                if let Ok(text) = serde_json::to_string_pretty(&msg.payload) {
                    let _ = app
                        .models_mut()
                        .update(&st.last_script_result_json, |v| *v = text);
                }
            }
            "bundle.dumped" => {
                if !message_matches_selected_session(app, st, &msg) {
                    continue;
                }
                if let Ok(text) = serde_json::to_string_pretty(&msg.payload) {
                    let _ = app.models_mut().update(&st.last_bundle_json, |v| *v = text);
                }
            }
            "screenshot.result" => {
                if !message_matches_selected_session(app, st, &msg) {
                    continue;
                }
                if let Ok(text) = serde_json::to_string_pretty(&msg.payload) {
                    let _ = app
                        .models_mut()
                        .update(&st.last_screenshot_json, |v| *v = text);
                }
            }
            _ => {}
        }
    }

    if drained_any {
        // Keep a small heartbeat while messages are flowing.
        // The driver can stop requesting frames once the UI is idle.
        // (We do not have a dedicated background-to-UI wakeup path yet.)
    }
}

fn ensure_session_selection_is_valid(app: &mut App, st: &mut State) {
    let selected = app
        .models()
        .read(&st.selected_session_id, |v| v.clone())
        .ok()
        .flatten();
    let sessions = app
        .models()
        .read(&st.sessions, |v| v.clone())
        .unwrap_or_default();

    if let Some(selected) = selected.as_deref() {
        if sessions.iter().any(|s| s.session_id == selected) {
            return;
        }
    }

    let new_selected = sessions
        .first()
        .map(|s| Arc::<str>::from(s.session_id.clone()));
    let _ = app
        .models_mut()
        .update(&st.selected_session_id, |v| *v = new_selected);
}

fn message_matches_selected_session(
    app: &mut App,
    st: &State,
    msg: &DiagTransportMessageV1,
) -> bool {
    let selected = app
        .models()
        .read(&st.selected_session_id, |v| v.clone())
        .ok()
        .flatten();
    let Some(selected) = selected else {
        return true;
    };
    msg.session_id.as_deref() == Some(selected.as_ref())
}

fn sync_selected_session_to_client(app: &mut App, st: &mut State) {
    let selected = app
        .models()
        .read(&st.selected_session_id, |v| v.clone())
        .ok()
        .flatten();

    if selected.as_deref() == st.applied_session_id.as_deref() {
        return;
    }

    st.client
        .set_default_session_id(selected.as_ref().map(|s| s.to_string()));
    st.applied_session_id = selected;
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
