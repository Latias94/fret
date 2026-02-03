use std::collections::HashMap;
use std::sync::Arc;

use fret_app::{CreateWindowKind, CreateWindowRequest, WindowRequest};
use fret_core::Color;
use fret_docking::{
    DockManager, DockPanel, DockPanelRegistry, DockPanelRegistryService, ViewportPanel,
    runtime as dock_runtime,
};
use fret_kit::interop::embedded_viewport as embedded;
use fret_kit::prelude::*;
use fret_render::{RenderTargetColorSpace, Renderer, WgpuContext};
use fret_runtime::{
    FrameId, PlatformCapabilities, TickId, WindowHoverDetectionQuality, WindowRole,
};
use fret_ui::element::LayoutStyle;

const VIEWPORT_PX_SIZE: (u32, u32) = (960, 540);
const AUX_LOGICAL_WINDOW_ID: &str = "aux";
const ENV_SINGLE_WINDOW: &str = "FRET_IMUI_EDITOR_PROOF_SINGLE_WINDOW";

struct ImUiEditorProofState {
    embedded: embedded::EmbeddedViewportSurface,
}

impl embedded::EmbeddedViewportRecord for ImUiEditorProofState {
    fn embedded_viewport_surface(&mut self) -> &mut embedded::EmbeddedViewportSurface {
        &mut self.embedded
    }

    fn embedded_viewport_label(&self) -> Option<&'static str> {
        Some("imui-editor-proof viewport")
    }

    fn record_embedded_viewport(
        &mut self,
        _app: &mut App,
        _window: AppWindowId,
        _context: &WgpuContext,
        _renderer: &mut Renderer,
        _scale_factor: f32,
        _tick_id: TickId,
        frame_id: FrameId,
        view: &wgpu::TextureView,
        encoder: &mut wgpu::CommandEncoder,
    ) {
        let t = (frame_id.0 as f32 * 0.02).sin() * 0.5 + 0.5;
        let clear = wgpu::Color {
            r: (0.08 + 0.30 * t) as f64,
            g: (0.08 + 0.25 * (1.0 - t)) as f64,
            b: (0.10 + 0.35 * (0.5 - (t - 0.5).abs())) as f64,
            a: 1.0,
        };
        embedded::clear_pass(encoder, view, Some("imui-editor-proof clear"), clear);
    }
}

pub fn run() -> anyhow::Result<()> {
    fret_kit::app_with_hooks("imui-editor-proof-demo", init_window, view, |d| {
        d.drive_embedded_viewport()
            .dock_op(on_dock_op)
            .window_create_spec(window_create_spec)
            .window_created(window_created)
    })?
    .with_main_window("imui_editor_proof_demo", (1120.0, 720.0))
    .init_app(|app| {
        configure_single_window_caps_if_requested(app);
        shadcn::shadcn_themes::apply_shadcn_new_york_v4(
            app,
            shadcn::shadcn_themes::ShadcnBaseColor::Slate,
            shadcn::shadcn_themes::ShadcnColorScheme::Dark,
        );
        install_dock_panel_registry(app);
    })
    .run()?;
    Ok(())
}

fn single_window_mode_enabled() -> bool {
    std::env::var_os(ENV_SINGLE_WINDOW).is_some_and(|v| !v.is_empty() && v != "0")
}

fn configure_single_window_caps_if_requested(app: &mut App) {
    if !single_window_mode_enabled() {
        return;
    }

    // Simulate wasm/mobile-like constraints:
    // - no OS multi-window tear-off
    // - no reliable hover detection across windows
    app.with_global_mut(PlatformCapabilities::default, |caps, _app| {
        caps.ui.multi_window = false;
        caps.ui.window_tear_off = false;
        caps.ui.window_hover_detection = WindowHoverDetectionQuality::None;
    });
}

fn init_window(app: &mut App, window: AppWindowId) -> ImUiEditorProofState {
    embedded::ensure_models(app, window);
    if !single_window_mode_enabled() {
        ensure_aux_window_requested(app, window);
    }

    ImUiEditorProofState {
        embedded: embedded::EmbeddedViewportSurface::new(
            wgpu::TextureFormat::Bgra8UnormSrgb,
            RenderTargetColorSpace::Srgb,
            VIEWPORT_PX_SIZE,
        ),
    }
}

fn view(cx: &mut ElementContext<'_, App>, _st: &mut ImUiEditorProofState) -> ViewElements {
    let window = cx.window;
    let last_input: Arc<str> = embedded::models(&*cx.app, window)
        .and_then(|models| cx.watch_model(&models.last_input).paint().cloned())
        .unwrap_or_else(|| Arc::from("<embedded viewport models missing>"));

    let caps = cx
        .app
        .global::<PlatformCapabilities>()
        .cloned()
        .unwrap_or_default();
    let window_size = cx
        .app
        .global::<fret_core::WindowMetricsService>()
        .and_then(|svc| svc.inner_size(window));
    let single = single_window_mode_enabled();

    fret_imui::imui(cx, |ui| {
        use fret_ui_kit::imui::UiWriterUiKitExt as _;

        let root = fret_ui_kit::ui::v_flex_build(ui.cx_mut(), move |cx, out| {
            fret_imui::imui_build(cx, out, |ui| {
                let headline = fret_ui_kit::ui::text(
                    ui.cx_mut(),
                    format!(
                        "imui editor-grade proof (M7): docking + multi-window + viewport surfaces (window={window:?})"
                    ),
                )
                .font_semibold();
                ui.add_ui(headline);

                if single {
                    let hint = fret_ui_kit::ui::text(
                        ui.cx_mut(),
                        format!(
                            "single-window mode enabled ({ENV_SINGLE_WINDOW}=1): dock tear-off should degrade to in-window floating"
                        ),
                    )
                    .text_xs();
                    ui.add_ui(hint);
                }

                let caps_line = fret_ui_kit::ui::text(
                    ui.cx_mut(),
                    format!(
                        "caps: multi_window={} window_tear_off={} window_hover_detection={:?} window_inner_size={window_size:?}",
                        caps.ui.multi_window, caps.ui.window_tear_off, caps.ui.window_hover_detection,
                    ),
                )
                .text_xs();
                ui.add_ui(caps_line);

                let controls = fret_ui_kit::ui::h_flex_build(ui.cx_mut(), move |cx, out| {
                    fret_imui::imui_build(cx, out, |ui| {
                        if ui.button("Reset layout").clicked() {
                            reset_dock_graph(ui.cx_mut().app, window);
                            dock_runtime::request_dock_invalidation(ui.cx_mut().app, [window]);
                        }
                        if ui.button("Center floatings").clicked() {
                            dock_runtime::recenter_in_window_floatings(ui.cx_mut().app, window);
                        }
                    });
                })
                .gap(fret_ui_kit::Space::N2);
                ui.add_ui(controls);

                let last_input_line = fret_ui_kit::ui::text(
                    ui.cx_mut(),
                    format!("last viewport input: {last_input}"),
                )
                .text_xs();
                ui.add_ui(last_input_line);
                ui.separator();

                let mut layout = LayoutStyle::default();
                layout.size.width = Length::Fill;
                layout.size.height = Length::Fill;
                layout.flex.grow = 1.0;

                fret_docking::imui::dock_space_with(
                    ui,
                    fret_docking::imui::DockSpaceImUiOptions {
                        layout,
                        test_id: Some("imui-editor-proof-dock"),
                    },
                    move |app, window| ensure_dock_graph(app, window),
                );
            });
        })
        .size_full();
        ui.add_ui(root);
    })
}

fn install_dock_panel_registry(app: &mut App) {
    let registry: Arc<dyn DockPanelRegistry<App>> = Arc::new(ImUiEditorProofPanelRegistry);
    app.with_global_mut(DockPanelRegistryService::<App>::default, |svc, _app| {
        svc.set(registry);
    });
}

struct ImUiEditorProofPanelRegistry;

impl DockPanelRegistry<App> for ImUiEditorProofPanelRegistry {
    fn render_panel(
        &self,
        ui: &mut UiTree<App>,
        app: &mut App,
        services: &mut dyn UiServices,
        window: AppWindowId,
        bounds: fret_core::Rect,
        panel: &fret_core::PanelKey,
    ) -> Option<fret_core::NodeId> {
        if panel.kind.0 != "demo.controls" {
            return None;
        }

        let root_name = match panel.instance.as_deref() {
            Some(instance) => format!("imui_editor_proof.panel.{}:{}", panel.kind.0, instance),
            None => format!("imui_editor_proof.panel.{}", panel.kind.0),
        };
        let panel_key = panel.clone();
        Some(fret_docking::render_cached_panel_root(
            ui,
            app,
            services,
            window,
            bounds,
            &root_name,
            move |cx| {
                let target = embedded::models(&*cx.app, window)
                    .and_then(|m| cx.watch_model(&m.target).paint().copied())
                    .unwrap_or_default();

                vec![
                    fret_ui_kit::ui::container_build(cx, move |cx, out| {
                        out.extend(
                            fret_imui::imui_vstack(cx, move |ui| {
                                ui.id(&panel_key, |ui| {
                                    ui.text("Controls panel (declarative root inside docking)");
                                    ui.text(format!("embedded viewport target: {target:?}"));
                                    ui.text(
                                        "Wasm/mobile note: multi-window should degrade to in-window floatings.",
                                    );
                                });
                            })
                            .into_vec(),
                        );
                    })
                    .size_full()
                    .p_3()
                    .bg(fret_ui_kit::ColorRef::Token {
                        key: "background",
                        fallback: fret_ui_kit::ColorFallback::ThemeSurfaceBackground,
                    })
                    .into_element(cx),
                ]
            },
        ))
    }
}

fn ensure_dock_graph(app: &mut App, window: AppWindowId) {
    ensure_dock_graph_inner(app, window, false);
}

fn reset_dock_graph(app: &mut App, window: AppWindowId) {
    app.with_global_mut(DockManager::default, |dock, _app| {
        dock.graph.remove_window_root(window);
        dock.graph.floating_windows_mut(window).clear();
    });
    ensure_dock_graph_inner(app, window, true);
}

fn ensure_dock_graph_inner(app: &mut App, window: AppWindowId, force: bool) {
    app.with_global_mut(DockManager::default, |dock, app| {
        let logical_window_id = app
            .global::<WindowBootstrapService>()
            .and_then(|svc| svc.logical_by_window.get(&window).cloned())
            .unwrap_or_else(|| format!("{window:?}"));

        let viewport_panel =
            fret_core::PanelKey::with_instance("demo.viewport", logical_window_id.clone());
        let controls_panel = fret_core::PanelKey::with_instance("demo.controls", logical_window_id);

        let target = embedded::models(app, window)
            .and_then(|m| app.models().read(&m.target, |v| *v).ok())
            .unwrap_or_default();

        dock.ensure_panel(&viewport_panel, || DockPanel {
            title: "Viewport".to_string(),
            color: Color::TRANSPARENT,
            viewport: None,
        });
        dock.ensure_panel(&controls_panel, || DockPanel {
            title: "Controls".to_string(),
            color: Color::TRANSPARENT,
            viewport: None,
        });

        if let Some(panel) = dock.panels.get_mut(&viewport_panel) {
            panel.viewport = if target == fret_core::RenderTargetId::default() {
                None
            } else {
                Some(ViewportPanel {
                    target,
                    target_px_size: VIEWPORT_PX_SIZE,
                    fit: fret_core::ViewportFit::Stretch,
                    context_menu_enabled: true,
                })
            };
        }

        if !force && dock.graph.window_root(window).is_some() {
            return;
        }

        use fret_core::{Axis, DockFloatingWindow, DockNode, Point, Px, Rect, Size};

        if single_window_mode_enabled() {
            // In single-window mode we want the "floating window" affordance to be immediately
            // visible without requiring the user to discover the float zone gesture.
            let tabs_viewport = dock.graph.insert_node(DockNode::Tabs {
                tabs: vec![viewport_panel],
                active: 0,
            });
            dock.graph.set_window_root(window, tabs_viewport);

            let tabs_controls = dock.graph.insert_node(DockNode::Tabs {
                tabs: vec![controls_panel],
                active: 0,
            });
            let floating = dock.graph.insert_node(DockNode::Floating {
                child: tabs_controls,
            });
            dock.graph
                .floating_windows_mut(window)
                .push(DockFloatingWindow {
                    floating,
                    rect: Rect::new(
                        Point::new(Px(24.0), Px(48.0)),
                        Size::new(Px(420.0), Px(240.0)),
                    ),
                });
        } else {
            let tabs_viewport = dock.graph.insert_node(DockNode::Tabs {
                tabs: vec![viewport_panel],
                active: 0,
            });
            let tabs_controls = dock.graph.insert_node(DockNode::Tabs {
                tabs: vec![controls_panel],
                active: 0,
            });
            let root = dock.graph.insert_node(DockNode::Split {
                axis: Axis::Vertical,
                children: vec![tabs_viewport, tabs_controls],
                fractions: vec![0.7, 0.3],
            });
            dock.graph.set_window_root(window, root);
        }

        dock_runtime::request_dock_invalidation(app, [window]);
    });
}

#[derive(Default)]
struct WindowBootstrapService {
    main_window: Option<AppWindowId>,
    aux_requested: bool,
    logical_by_window: HashMap<AppWindowId, String>,
}

fn ensure_aux_window_requested(app: &mut App, window: AppWindowId) {
    app.with_global_mut(WindowBootstrapService::default, |svc, app| {
        if svc.main_window.is_none() {
            svc.main_window = Some(window);
            svc.logical_by_window.insert(window, "main".to_string());
        }
        if svc.main_window != Some(window) {
            svc.logical_by_window
                .entry(window)
                .or_insert_with(|| AUX_LOGICAL_WINDOW_ID.to_string());
        }
        if svc.aux_requested {
            return;
        }
        if svc.main_window != Some(window) {
            return;
        }

        svc.aux_requested = true;
        app.push_effect(Effect::Window(WindowRequest::Create(CreateWindowRequest {
            kind: CreateWindowKind::DockRestore {
                logical_window_id: AUX_LOGICAL_WINDOW_ID.to_string(),
            },
            anchor: None,
            role: WindowRole::Auxiliary,
            style: Default::default(),
        })));
    });
}

fn on_dock_op(app: &mut App, op: fret_core::DockOp) {
    let _ = dock_runtime::handle_dock_op(app, op);
}

fn window_create_spec(
    _app: &mut App,
    request: &fret_app::CreateWindowRequest,
) -> Option<fret_launch::WindowCreateSpec> {
    match &request.kind {
        CreateWindowKind::DockFloating { panel, .. } => Some(fret_launch::WindowCreateSpec::new(
            format!("fret-demo imui_editor_proof_demo — {}", panel.kind.0),
            winit::dpi::LogicalSize::new(720.0, 520.0),
        )),
        CreateWindowKind::DockRestore { logical_window_id } => {
            Some(fret_launch::WindowCreateSpec::new(
                format!("fret-demo imui_editor_proof_demo — {logical_window_id}"),
                winit::dpi::LogicalSize::new(980.0, 720.0),
            ))
        }
    }
}

fn window_created(app: &mut App, request: &fret_app::CreateWindowRequest, new_window: AppWindowId) {
    if let CreateWindowKind::DockRestore { logical_window_id } = &request.kind {
        app.with_global_mut(WindowBootstrapService::default, |svc, _app| {
            svc.logical_by_window
                .insert(new_window, logical_window_id.clone());
        });
    }
    let _ = dock_runtime::handle_dock_window_created(app, request, new_window);
}
