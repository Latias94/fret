use std::net::{IpAddr, Ipv4Addr, SocketAddr};
use std::sync::Arc;

use fret_app::{App, CommandId, Effect};
use fret_bootstrap::BootstrapBuilder;
use fret_bootstrap::ui_app_driver::{UiAppDriver, ViewElements};
use fret_core::{AppWindowId, UiServices};
use fret_diag_protocol::DiagTransportMessageV1;
use fret_diag_ws::client::{ClientKindV1, DevtoolsWsClient, DevtoolsWsClientConfig};
use fret_diag_ws::server::{DevtoolsWsServer, DevtoolsWsServerConfig};
use fret_runtime::Model;
use fret_ui::element::AnyElement;
use fret_ui::elements::ContinuousFrames;
use fret_ui::{ElementContext, Invalidation, Theme};
use fret_ui_shadcn as shadcn;

const CMD_COPY_WS_URL: &str = "fret.devtools.copy_ws_url";
const CMD_COPY_TOKEN: &str = "fret.devtools.copy_token";
const CMD_INSPECT_ENABLE: &str = "fret.devtools.inspect_enable";
const CMD_INSPECT_DISABLE: &str = "fret.devtools.inspect_disable";
const CMD_PICK_ARM: &str = "fret.devtools.pick_arm";
const CMD_BUNDLE_DUMP: &str = "fret.devtools.bundle_dump";
const CMD_SCRIPT_PUSH: &str = "fret.devtools.script_push";
const CMD_SCRIPT_RUN: &str = "fret.devtools.script_run";

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
    inspect_consume_clicks: Model<bool>,
    script_text: Model<String>,

    last_pick_json: Model<String>,
    last_script_result_json: Model<String>,
    last_bundle_json: Model<String>,
    log_lines: Model<Vec<Arc<str>>>,

    client: DevtoolsWsClient,
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
    let inspect_consume_clicks = app.models_mut().insert(false);
    let script_text = app.models_mut().insert(String::new());
    let last_pick_json = app.models_mut().insert(String::new());
    let last_script_result_json = app.models_mut().insert(String::new());
    let last_bundle_json = app.models_mut().insert(String::new());
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

    State {
        cfg,
        panel_fractions,
        details_tab,
        inspect_consume_clicks,
        script_text,
        last_pick_json,
        last_script_result_json,
        last_bundle_json,
        log_lines,
        client,
    }
}

fn view(cx: &mut ElementContext<'_, App>, st: &mut State) -> ViewElements {
    drain_ws_messages(cx.app, st);

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
    cx.observe_model(&st.inspect_consume_clicks, Invalidation::Paint);
    cx.observe_model(&st.script_text, Invalidation::Paint);
    cx.observe_model(&st.last_pick_json, Invalidation::Paint);
    cx.observe_model(&st.last_script_result_json, Invalidation::Paint);
    cx.observe_model(&st.last_bundle_json, Invalidation::Paint);
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
                            .on_click(CMD_INSPECT_ENABLE)
                            .into_element(cx),
                        shadcn::Button::new("Inspect Off")
                            .variant(shadcn::ButtonVariant::Outline)
                            .size(shadcn::ButtonSize::Sm)
                            .on_click(CMD_INSPECT_DISABLE)
                            .into_element(cx),
                        shadcn::Button::new("Pick")
                            .variant(shadcn::ButtonVariant::Outline)
                            .size(shadcn::ButtonSize::Sm)
                            .on_click(CMD_PICK_ARM)
                            .into_element(cx),
                        shadcn::Button::new("Dump Bundle")
                            .variant(shadcn::ButtonVariant::Outline)
                            .size(shadcn::ButtonSize::Sm)
                            .on_click(CMD_BUNDLE_DUMP)
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

fn center_panel(cx: &mut ElementContext<'_, App>, _theme: &Theme, st: &State) -> AnyElement {
    let script_text = cx
        .app
        .models()
        .read(&st.script_text, |v| v.clone())
        .unwrap_or_default();
    let consume_clicks = cx
        .app
        .models()
        .read(&st.inspect_consume_clicks, |v| *v)
        .unwrap_or(false);

    let consume_toggle = shadcn::Checkbox::new(st.inspect_consume_clicks.clone())
        .a11y_label("Consume clicks while inspecting")
        .into_element(cx);

    let textarea = shadcn::Textarea::new(st.script_text.clone())
        .a11y_label("Script JSON")
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
                    .on_click(CMD_SCRIPT_PUSH)
                    .into_element(cx),
                shadcn::Button::new("Run Script")
                    .variant(shadcn::ButtonVariant::Default)
                    .size(shadcn::ButtonSize::Sm)
                    .on_click(CMD_SCRIPT_RUN)
                    .into_element(cx),
                consume_toggle,
                cx.text(format!(
                    "consume_clicks={}",
                    if consume_clicks { "true" } else { "false" }
                )),
            ]
        },
    );

    shadcn::Card::new([
        shadcn::CardHeader::new([
            shadcn::CardTitle::new("Commands").into_element(cx),
            shadcn::CardDescription::new("Send protocol commands to connected apps.")
                .into_element(cx),
        ])
        .into_element(cx),
        shadcn::CardContent::new([
            buttons,
            cx.text("Script (paste JSON):"),
            textarea,
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

    let tabs = shadcn::Tabs::new(st.details_tab.clone())
        .refine_layout(fret_ui_kit::LayoutRefinement::default().w_full())
        .items([
            shadcn::TabsItem::new("pick", "Pick", [text_blob(cx, pick)]),
            shadcn::TabsItem::new("script", "Script", [text_blob(cx, script)]),
            shadcn::TabsItem::new("bundle", "Bundle", [text_blob(cx, bundle)]),
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
            st.client
                .send_type_payload("pick.arm", serde_json::json!({}));
            app.push_effect(Effect::RequestAnimationFrame(window));
        }
        CMD_BUNDLE_DUMP => {
            st.client
                .send_type_payload("bundle.dump", serde_json::json!({ "label": "devtools" }));
            app.push_effect(Effect::RequestAnimationFrame(window));
        }
        CMD_SCRIPT_PUSH | CMD_SCRIPT_RUN => {
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

fn drain_ws_messages(app: &mut App, st: &mut State) {
    let mut drained_any = false;
    while let Some(msg) = st.client.try_recv() {
        drained_any = true;

        let ty = msg.r#type.clone();
        let compact = format!("type={ty}");
        push_log(app, &st.log_lines, &compact);

        match ty.as_str() {
            "pick.result" => {
                if let Ok(text) = serde_json::to_string_pretty(&msg.payload) {
                    let _ = app.models_mut().update(&st.last_pick_json, |v| *v = text);
                }
            }
            "script.result" => {
                if let Ok(text) = serde_json::to_string_pretty(&msg.payload) {
                    let _ = app
                        .models_mut()
                        .update(&st.last_script_result_json, |v| *v = text);
                }
            }
            "bundle.dumped" => {
                if let Ok(text) = serde_json::to_string_pretty(&msg.payload) {
                    let _ = app.models_mut().update(&st.last_bundle_json, |v| *v = text);
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
