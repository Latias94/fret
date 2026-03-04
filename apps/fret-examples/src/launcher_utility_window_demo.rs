use std::sync::Arc;
use std::time::Duration;

use fret_app::{App, CommandId, Effect, WindowRequest};
use fret_bootstrap::ui_app_with_hooks;
use fret_core::{AppWindowId, MouseButton, Px, SemanticsRole};
use fret_runtime::DefaultAction;
use fret_runtime::{
    RunnerWindowStyleDiagnosticsStore, TimerToken, WindowDecorationsRequest, WindowResizeDirection,
    WindowStyleRequest, WindowZLevel,
};
use fret_ui::ElementContext;
use fret_ui::element::{LayoutStyle, Length, PointerRegionProps, SemanticsDecoration, SizeStyle};
use fret_ui_kit::declarative::ModelWatchExt as _;
use fret_ui_kit::declarative::stack;
use fret_ui_kit::{ColorRef, LayoutRefinement, Space, ui};
use fret_ui_shadcn::button::{Button, ButtonSize, ButtonVariant};
use fret_ui_shadcn::{Card, CardContent, CardDescription, CardHeader, CardTitle};

const CMD_BLINK: &str = "launcher_utility_window_demo.blink";
const CMD_TOGGLE_ALWAYS_ON_TOP: &str = "launcher_utility_window_demo.toggle_always_on_top";
const CMD_QUIT: &str = "launcher_utility_window_demo.quit";

const TEST_ID_ROOT: &str = "utility-window.root";
const TEST_ID_DRAG_REGION: &str = "utility-window.chrome.drag_region";
const TEST_ID_RESIZE_SE: &str = "utility-window.chrome.resize_se";
const TEST_ID_BLINK: &str = "utility-window.blink";
const TEST_ID_TOGGLE_ALWAYS_ON_TOP: &str = "utility-window.always_on_top";
const TEST_ID_QUIT: &str = "utility-window.quit";
const TEST_ID_STYLE_TEXT: &str = "utility-window.style_effective";

pub fn run() -> anyhow::Result<()> {
    ui_app_with_hooks(
        "launcher-utility-window-demo",
        init_window,
        view,
        configure_driver,
    )
    .with_default_diagnostics()
    .with_main_window("launcher_utility_window_demo", (640.0, 420.0))
    .configure(|config| {
        config.main_window_style = WindowStyleRequest {
            decorations: Some(WindowDecorationsRequest::None),
            resizable: Some(true),
            transparent: Some(false),
            ..Default::default()
        };
    })
    .init_app(|app| {
        app.commands_mut().register(
            CommandId::from(CMD_BLINK),
            fret_app::CommandMeta::new("Blink (hide + show)"),
        );
        app.commands_mut().register(
            CommandId::from(CMD_TOGGLE_ALWAYS_ON_TOP),
            fret_app::CommandMeta::new("Toggle always on top"),
        );
        app.commands_mut().register(
            CommandId::from(CMD_QUIT),
            fret_app::CommandMeta::new("Quit"),
        );
    })
    .run()
    .map_err(anyhow::Error::from)
}

struct LauncherUtilityWindowState {
    window: AppWindowId,
    always_on_top: fret_runtime::Model<bool>,
    blink_timer: Option<TimerToken>,
    status: fret_runtime::Model<Arc<str>>,
}

fn init_window(app: &mut App, window: AppWindowId) -> LauncherUtilityWindowState {
    LauncherUtilityWindowState {
        window,
        always_on_top: app.models_mut().insert(false),
        blink_timer: None,
        status: app.models_mut().insert(Arc::from("Idle")),
    }
}

fn configure_driver(
    driver: fret_bootstrap::ui_app_driver::UiAppDriver<LauncherUtilityWindowState>,
) -> fret_bootstrap::ui_app_driver::UiAppDriver<LauncherUtilityWindowState> {
    driver.on_command(on_command).on_event(on_event)
}

fn on_command(
    app: &mut App,
    _services: &mut dyn fret_core::UiServices,
    window: AppWindowId,
    _ui: &mut fret_ui::UiTree<App>,
    st: &mut LauncherUtilityWindowState,
    command: &CommandId,
) {
    match command.as_str() {
        CMD_BLINK => {
            if st.blink_timer.is_some() {
                return;
            }

            let token = app.next_timer_token();
            st.blink_timer = Some(token);
            let _ = app
                .models_mut()
                .update(&st.status, |v| *v = Arc::from("Blink: hide"));

            app.push_effect(Effect::Window(WindowRequest::SetVisible {
                window,
                visible: false,
            }));
            app.push_effect(Effect::SetTimer {
                window: Some(window),
                token,
                after: Duration::from_millis(250),
                repeat: None,
            });
        }
        CMD_TOGGLE_ALWAYS_ON_TOP => {
            let next = app
                .models_mut()
                .update(&st.always_on_top, |v| {
                    *v = !*v;
                    *v
                })
                .ok()
                .unwrap_or(false);

            let z_level = if next {
                WindowZLevel::AlwaysOnTop
            } else {
                WindowZLevel::Normal
            };
            app.push_effect(Effect::Window(WindowRequest::SetStyle {
                window,
                style: WindowStyleRequest {
                    z_level: Some(z_level),
                    ..Default::default()
                },
            }));
            let _ = app.models_mut().update(&st.status, |v| {
                *v = Arc::from(if next {
                    "AlwaysOnTop: on"
                } else {
                    "AlwaysOnTop: off"
                })
            });
        }
        CMD_QUIT => {
            app.push_effect(Effect::QuitApp);
        }
        _ => {}
    }
}

fn on_event(
    app: &mut App,
    _services: &mut dyn fret_core::UiServices,
    window: AppWindowId,
    _ui: &mut fret_ui::UiTree<App>,
    st: &mut LauncherUtilityWindowState,
    event: &fret_core::Event,
) {
    let fret_core::Event::Timer { token } = event else {
        return;
    };
    if st.blink_timer != Some(*token) {
        return;
    }
    st.blink_timer = None;

    app.push_effect(Effect::Window(WindowRequest::SetVisible {
        window,
        visible: true,
    }));
    let _ = app
        .models_mut()
        .update(&st.status, |v| *v = Arc::from("Blink: show"));
    app.request_redraw(window);
}

fn view(
    cx: &mut ElementContext<'_, App>,
    st: &mut LauncherUtilityWindowState,
) -> fret_bootstrap::ui_app_driver::ViewElements {
    let theme = cx.theme().snapshot();
    let color_background = theme.color_token("background");
    let color_muted_foreground = theme.color_token("muted-foreground");
    let color_secondary = theme.color_token("secondary");

    let always_on_top = cx.watch_model(&st.always_on_top).layout().copied_or(false);
    let status = cx
        .watch_model(&st.status)
        .layout()
        .cloned_or_else(|| Arc::from("Idle"));

    let effective_style = cx
        .app
        .global::<RunnerWindowStyleDiagnosticsStore>()
        .and_then(|store| store.effective_snapshot(st.window));
    let style_text: Arc<str> = Arc::from(match effective_style {
        Some(s) => format!(
            "effective: decorations={:?} resizable={} visual_transparent={} z_level={:?}",
            s.decorations, s.resizable, s.visual_transparent, s.z_level
        ),
        None => "effective: <unavailable>".to_string(),
    });

    let header = cx.container(
        fret_ui::element::ContainerProps {
            layout: LayoutStyle {
                size: SizeStyle {
                    width: Length::Fill,
                    height: Length::Px(Px(44.0)),
                    ..Default::default()
                },
                ..Default::default()
            },
            padding: fret_core::Edges::all(Px(8.0)).into(),
            background: Some(color_secondary),
            corner_radii: fret_core::Corners::all(Px(10.0)),
            ..Default::default()
        },
        move |cx| {
            vec![stack::hstack(
                cx,
                stack::HStackProps::default()
                    .gap(Space::N3)
                    .items_center()
                    .layout(LayoutRefinement::default().w_full()),
                move |cx| {
                    let mut drag_region_props = PointerRegionProps::default();
                    drag_region_props.layout = LayoutStyle {
                        size: SizeStyle {
                            width: Length::Fill,
                            height: Length::Px(Px(40.0)),
                            ..Default::default()
                        },
                        ..Default::default()
                    };

                    let drag_region = cx
                        .pointer_region(drag_region_props, |cx| {
                            cx.pointer_region_on_pointer_down(Arc::new(|host, acx, down| {
                                if down.button != MouseButton::Left {
                                    return false;
                                }
                                host.prevent_default(DefaultAction::FocusOnPointerDown);
                                host.push_effect(Effect::Window(WindowRequest::BeginDrag {
                                    window: acx.window,
                                }));
                                true
                            }));

                            vec![
                                ui::text(cx, "Launcher Utility Window (drag here)")
                                    .font_semibold()
                                    .into_element(cx),
                            ]
                        })
                        .attach_semantics(
                            SemanticsDecoration::default()
                                .test_id(TEST_ID_DRAG_REGION)
                                .role(SemanticsRole::Button),
                        );

                    let header_controls = stack::hstack(
                        cx,
                        stack::HStackProps::default()
                            .gap(Space::N2)
                            .items_center()
                            .layout(LayoutRefinement::default().flex_shrink_0()),
                        move |cx| {
                            vec![
                                Button::new("Blink")
                                    .variant(ButtonVariant::Secondary)
                                    .size(ButtonSize::Sm)
                                    .on_click(CommandId::from(CMD_BLINK))
                                    .test_id(TEST_ID_BLINK)
                                    .into_element(cx),
                                Button::new(if always_on_top {
                                    "Always on top: on"
                                } else {
                                    "Always on top: off"
                                })
                                .variant(ButtonVariant::Outline)
                                .size(ButtonSize::Sm)
                                .on_click(CommandId::from(CMD_TOGGLE_ALWAYS_ON_TOP))
                                .test_id(TEST_ID_TOGGLE_ALWAYS_ON_TOP)
                                .into_element(cx),
                                Button::new("Quit")
                                    .variant(ButtonVariant::Destructive)
                                    .size(ButtonSize::Sm)
                                    .on_click(CommandId::from(CMD_QUIT))
                                    .test_id(TEST_ID_QUIT)
                                    .into_element(cx),
                            ]
                        },
                    );

                    vec![drag_region, header_controls]
                },
            )]
        },
    );

    let content = Card::new([
        CardHeader::new([
            CardTitle::new("MVP gates").into_element(cx),
            CardDescription::new(
                "Frameless main window + BeginDrag/BeginResize hooks + SetVisible (blink).",
            )
            .into_element(cx),
        ])
        .into_element(cx),
        CardContent::new([stack::vstack(
            cx,
            stack::VStackProps::default().gap_y(Space::N3),
            move |cx| {
                vec![
                    ui::text(cx, style_text)
                        .font_monospace()
                        .text_sm()
                        .into_element(cx)
                        .attach_semantics(
                            SemanticsDecoration::default().test_id(TEST_ID_STYLE_TEXT),
                        ),
                    ui::text(cx, status)
                        .text_sm()
                        .text_color(ColorRef::Color(color_muted_foreground))
                        .into_element(cx),
                ]
            },
        )])
        .into_element(cx),
    ])
    .into_element(cx);

    let resize_row = stack::hstack(
        cx,
        stack::HStackProps::default()
            .layout(LayoutRefinement::default().w_full())
            .justify_end(),
        move |cx| {
            let mut resize_props = PointerRegionProps::default();
            resize_props.layout = LayoutStyle {
                size: SizeStyle {
                    width: Length::Px(Px(24.0)),
                    height: Length::Px(Px(24.0)),
                    ..Default::default()
                },
                ..Default::default()
            };

            let resize_handle = cx
                .pointer_region(resize_props, |cx| {
                    cx.pointer_region_on_pointer_down(Arc::new(|host, acx, down| {
                        if down.button != MouseButton::Left {
                            return false;
                        }
                        host.push_effect(Effect::Window(WindowRequest::BeginResize {
                            window: acx.window,
                            direction: WindowResizeDirection::Se,
                        }));
                        true
                    }));

                    vec![ui::text(cx, "↘").font_semibold().into_element(cx)]
                })
                .attach_semantics(
                    SemanticsDecoration::default()
                        .test_id(TEST_ID_RESIZE_SE)
                        .role(SemanticsRole::Button),
                );

            vec![resize_handle]
        },
    );

    let root = cx
        .container(
            fret_ui::element::ContainerProps {
                layout: {
                    let mut layout = LayoutStyle::default();
                    layout.size.width = Length::Fill;
                    layout.size.height = Length::Fill;
                    layout
                },
                padding: fret_core::Edges::all(Px(16.0)).into(),
                background: Some(color_background),
                ..Default::default()
            },
            move |cx| {
                vec![stack::vstack(
                    cx,
                    stack::VStackProps::default()
                        .gap_y(Space::N4)
                        .layout(LayoutRefinement::default().w_full().h_full()),
                    move |_cx| vec![header, content, resize_row],
                )]
            },
        )
        .attach_semantics(SemanticsDecoration::default().test_id(TEST_ID_ROOT));

    vec![root].into()
}
