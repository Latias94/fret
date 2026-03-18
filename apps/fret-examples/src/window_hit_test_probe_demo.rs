use std::collections::HashMap;
use std::sync::Arc;

use fret::advanced::prelude::*;
use fret::component::prelude::*;
use fret_app::{CreateWindowKind, CreateWindowRequest, WindowRequest};
use fret_bootstrap::ui_app_driver;
use fret_core::{AppWindowId, Px};
use fret_launch::{WindowCreateSpec, WindowLogicalSize, WinitRunnerConfig};
use fret_runtime::{
    ActivationPolicy, WindowDecorationsRequest, WindowRole, WindowStyleRequest, WindowZLevel,
};
use fret_ui::ElementContext;
use fret_ui::element::{LayoutStyle, Length, SizeStyle};
use fret_ui_kit::{ColorRef, LayoutRefinement, Space, ui};
use fret_ui_shadcn::facade as shadcn;

const OVERLAY_LOGICAL_WINDOW_ID: &str = "overlay";

const TEST_ID_BASE_ROOT: &str = "window-hit-test-probe.base.root";
const TEST_ID_OVERLAY_ROOT: &str = "window-hit-test-probe.overlay.root";

pub fn run() -> anyhow::Result<()> {
    let driver = ui_app_driver::UiAppDriver::new("window-hit-test-probe-demo", init_window, view)
        .on_preferences(ui_app_driver::default_on_preferences::<WindowState>)
        .window_create_spec(window_create_spec)
        .window_created(window_created)
        .into_fn_driver();

    let mut config = WinitRunnerConfig::default();
    config.main_window_title = "window_hit_test_probe_demo".to_string();
    config.main_window_size = WindowLogicalSize::new(720.0, 460.0);
    // Keep deterministic overlap: place restored windows relative to the anchor point.
    config.new_window_anchor_offset = (0.0, 0.0);

    fret::advanced::interop::run_native_with_compat_driver(config, KernelApp::new(), driver)?;
    Ok(())
}

#[derive(Default)]
struct WindowBootstrapService {
    main_window: Option<AppWindowId>,
    overlay_requested: bool,
    logical_by_window: HashMap<AppWindowId, String>,
}

fn ensure_overlay_window_requested(app: &mut KernelApp, window: AppWindowId) {
    app.with_global_mut(WindowBootstrapService::default, |svc, app| {
        if svc.main_window.is_none() {
            svc.main_window = Some(window);
            svc.logical_by_window.insert(window, "main".to_string());
        }
        if svc.overlay_requested {
            return;
        }
        if svc.main_window != Some(window) {
            return;
        }

        svc.overlay_requested = true;
        let anchor = Some(fret_core::WindowAnchor {
            window,
            position: fret_core::Point::new(Px(48.0), Px(48.0)),
        });
        app.push_effect(Effect::Window(WindowRequest::Create(CreateWindowRequest {
            kind: CreateWindowKind::DockRestore {
                logical_window_id: OVERLAY_LOGICAL_WINDOW_ID.to_string(),
            },
            anchor,
            role: WindowRole::Auxiliary,
            style: WindowStyleRequest {
                decorations: Some(WindowDecorationsRequest::None),
                activation: Some(ActivationPolicy::NonActivating),
                z_level: Some(WindowZLevel::AlwaysOnTop),
                ..Default::default()
            },
        })));
    });
}

struct WindowState {
    window: AppWindowId,
    status: Model<Arc<str>>,
}

fn init_window(app: &mut KernelApp, window: AppWindowId) -> WindowState {
    ensure_overlay_window_requested(app, window);
    WindowState {
        window,
        status: app.models_mut().insert(Arc::from("Ready")),
    }
}

fn window_create_spec(
    _app: &mut KernelApp,
    request: &CreateWindowRequest,
) -> Option<WindowCreateSpec> {
    match &request.kind {
        CreateWindowKind::DockRestore { logical_window_id } => Some(WindowCreateSpec::new(
            format!("fret-demo window_hit_test_probe_demo — {logical_window_id}"),
            WindowLogicalSize::new(560.0, 360.0),
        )),
        CreateWindowKind::DockFloating { .. } => None,
    }
}

fn window_created(app: &mut KernelApp, request: &CreateWindowRequest, new_window: AppWindowId) {
    if let CreateWindowKind::DockRestore { logical_window_id } = &request.kind {
        app.with_global_mut(WindowBootstrapService::default, |svc, _app| {
            svc.logical_by_window
                .insert(new_window, logical_window_id.clone());
        });

        if logical_window_id == OVERLAY_LOGICAL_WINDOW_ID {
            let sender = app
                .global::<WindowBootstrapService>()
                .and_then(|svc| svc.main_window);
            app.push_effect(Effect::Window(WindowRequest::Raise {
                window: new_window,
                sender,
            }));
        }
    }
}

fn view(cx: &mut ElementContext<'_, KernelApp>, st: &mut WindowState) -> ViewElements {
    let theme = cx.theme().snapshot();
    let color_muted_foreground = theme.color_token("muted-foreground");
    let color_secondary = theme.color_token("secondary");
    let color_background = theme.color_token("background");

    let logical = cx
        .app
        .global::<WindowBootstrapService>()
        .and_then(|svc| svc.logical_by_window.get(&st.window))
        .cloned()
        .unwrap_or_else(|| "<unknown>".to_string());
    let is_overlay = logical == OVERLAY_LOGICAL_WINDOW_ID;

    let status = st.status.layout_in(cx).value_or_else(|| Arc::from("Idle"));

    let root_test_id = if is_overlay {
        TEST_ID_OVERLAY_ROOT
    } else {
        TEST_ID_BASE_ROOT
    };

    let root = cx.container(
        fret_ui::element::ContainerProps {
            layout: LayoutStyle {
                size: SizeStyle {
                    width: Length::Fill,
                    height: Length::Fill,
                    ..Default::default()
                },
                ..Default::default()
            },
            padding: fret_core::Edges::all(Px(16.0)).into(),
            background: Some(color_background),
            ..Default::default()
        },
        move |cx| {
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
                    vec![
                        ui::text("Hit-test passthrough probe")
                            .font_semibold()
                            .text_sm()
                            .into_element(cx),
                    ]
                },
            );

            let content = shadcn::Card::new([
                shadcn::CardHeader::new([
                    shadcn::CardTitle::new("Hit-test passthrough probe").into_element(cx),
                    shadcn::CardDescription::new(
                        "Use diag scripts to patch hit_test regions and assert OS-level receiver selection.",
                    )
                    .into_element(cx),
                ])
                .into_element(cx),
                shadcn::CardContent::new([
                    ui::v_flex(move |cx| {
                        let logical_line =
                            ui::text(format!("logical_window_id={logical}"))
                                .font_monospace()
                                .text_sm()
                                .into_element(cx);
                        let status_line = ui::text(status)
                            .text_sm()
                            .text_color(ColorRef::Color(color_muted_foreground))
                            .into_element(cx);
                        [logical_line, status_line]
                    })
                    .gap(Space::N3)
                    .layout(LayoutRefinement::default().w_full())
                    .into_element(cx),
                ])
                .into_element(cx),
            ])
            .into_element(cx)
            .test_id(root_test_id);

            vec![header, content]
        },
    );

    ViewElements::new([root])
}
