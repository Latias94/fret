use std::collections::HashMap;
use std::sync::Arc;

use fret::advanced::KernelApp;
use fret::advanced::interop::embedded_viewport as embedded;
use fret::app::AppRenderDataExt as _;
use fret::imui::prelude::*;
use fret_app::{CreateWindowKind, CreateWindowRequest, Effect, WindowRequest};
use fret_core::{AppWindowId, Color, DockFloatingWindow, DockNode, Point, Px, Rect, Size};
use fret_docking::{
    DockManager, DockPanel, DockPanelElementRegistry, DockPanelElementRegistryService,
    ViewportPanel, runtime as dock_runtime,
};
use fret_runtime::{ActivationPolicy, WindowRole, WindowStyleRequest};
use fret_ui::ElementContext;
use fret_ui::element::AnyElement;
use fret_ui_kit::{IntoUiElement as _, StyledExt as _, UiExt as _};

use super::{AUX_LOGICAL_WINDOW_ID, VIEWPORT_PX_SIZE, diag_enabled, single_window_mode_enabled};

pub(super) fn install_dock_panel_registry(app: &mut KernelApp) {
    app.with_global_mut(
        DockPanelElementRegistryService::<KernelApp>::default,
        |svc, _app| {
            svc.set(Arc::new(ImUiEditorProofControlsPanelRegistry));
        },
    );
}

struct ImUiEditorProofControlsPanelRegistry;

impl DockPanelElementRegistry<KernelApp> for ImUiEditorProofControlsPanelRegistry {
    fn render_panel(
        &self,
        cx: &mut ElementContext<'_, KernelApp>,
        _window: AppWindowId,
        panel: &fret_core::PanelKey,
    ) -> Option<AnyElement> {
        if panel.kind.0.as_str() != "demo.controls" {
            return None;
        }
        let panel_key = panel.clone();
        let target = embedded::models(&*cx.app, cx.window)
            .map(|m| cx.data().selector_model_paint(&m.target, |target| target))
            .unwrap_or_default();

        Some(
            fret_ui_kit::ui::container_build(move |cx, out| {
                out.extend(
                    imui(cx, move |ui| {
                        // Dock panels can move across roots and windows, so the immediate
                        // content keeps an explicit stable identity instead of relying on
                        // callsite position alone.
                        ui.id(&panel_key, |ui| {
                            ui.text("Controls panel (declarative root inside docking)");
                            ui.text(format!("embedded viewport target: {target:?}"));
                            ui.text_wrapped(
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
        )
    }
}

pub(super) fn dock_test_id_for_window(
    app: &KernelApp,
    window: AppWindowId,
) -> Option<&'static str> {
    let logical_window_id = app
        .global::<WindowBootstrapService>()
        .and_then(|svc| svc.logical_by_window.get(&window).cloned());

    if logical_window_id.as_deref() == Some("main") {
        Some("imui-editor-proof.main.dock")
    } else if logical_window_id.as_deref() == Some(AUX_LOGICAL_WINDOW_ID) {
        Some("imui-editor-proof.aux.dock")
    } else {
        None
    }
}

pub(super) fn ensure_dock_graph(app: &mut KernelApp, window: AppWindowId) {
    ensure_dock_graph_inner(app, window, false);
}

pub(super) fn reset_dock_graph(app: &mut KernelApp, window: AppWindowId) {
    app.with_global_mut(DockManager::default, |dock, _app| {
        dock.graph.remove_window_root(window);
        dock.graph.floating_windows_mut(window).clear();
    });
    ensure_dock_graph_inner(app, window, true);
}

fn embedded_target_for_window(app: &KernelApp, window: AppWindowId) -> fret_core::RenderTargetId {
    embedded::models(app, window)
        .and_then(|m| app.models().read(&m.target, |v| *v).ok())
        .unwrap_or_default()
}

fn ensure_dock_graph_inner(app: &mut KernelApp, window: AppWindowId, force: bool) {
    app.with_global_mut(DockManager::default, |dock, app| {
        let logical_window_id = app
            .global::<WindowBootstrapService>()
            .and_then(|svc| svc.logical_by_window.get(&window).cloned())
            .unwrap_or_else(|| format!("{window:?}"));

        let viewport_panel =
            fret_core::PanelKey::with_instance("demo.viewport", logical_window_id.clone());
        let controls_panel = fret_core::PanelKey::with_instance("demo.controls", logical_window_id);

        let target = embedded_target_for_window(app, window);

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
                axis: fret_core::Axis::Vertical,
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

pub(super) fn ensure_aux_window_requested(app: &mut KernelApp, window: AppWindowId) {
    app.with_global_mut(WindowBootstrapService::default, |svc, app| {
        if svc.main_window.is_none() {
            svc.main_window = Some(window);
            svc.logical_by_window.insert(window, "main".to_string());
        }
        if svc.aux_requested {
            return;
        }
        if svc.main_window != Some(window) {
            return;
        }

        svc.aux_requested = true;
        let anchor = diag_enabled().then_some(fret_core::WindowAnchor {
            window,
            position: fret_core::Point::new(fret_core::Px(120.0), fret_core::Px(24.0)),
        });
        app.push_effect(Effect::Window(WindowRequest::Create(CreateWindowRequest {
            kind: CreateWindowKind::DockRestore {
                logical_window_id: AUX_LOGICAL_WINDOW_ID.to_string(),
            },
            anchor,
            role: WindowRole::Auxiliary,
            style: WindowStyleRequest {
                activation: diag_enabled().then_some(ActivationPolicy::NonActivating),
                ..Default::default()
            },
        })));
    });
}

pub(super) fn on_dock_op(app: &mut KernelApp, op: fret_core::DockOp) {
    let _ = dock_runtime::handle_dock_op(app, op);
}

pub(super) fn window_create_spec(
    _app: &mut KernelApp,
    request: &fret_app::CreateWindowRequest,
) -> Option<fret_launch::WindowCreateSpec> {
    match &request.kind {
        CreateWindowKind::DockFloating { panel, .. } => Some(fret_launch::WindowCreateSpec::new(
            format!("fret-demo imui_editor_proof_demo — {}", panel.kind.0),
            fret_launch::WindowLogicalSize::new(720.0, 520.0),
        )),
        CreateWindowKind::DockRestore { logical_window_id } => {
            Some(fret_launch::WindowCreateSpec::new(
                format!("fret-demo imui_editor_proof_demo — {logical_window_id}"),
                fret_launch::WindowLogicalSize::new(980.0, 720.0),
            ))
        }
    }
}

pub(super) fn window_created(
    app: &mut KernelApp,
    request: &fret_app::CreateWindowRequest,
    new_window: AppWindowId,
) {
    if let CreateWindowKind::DockRestore { logical_window_id } = &request.kind {
        app.with_global_mut(WindowBootstrapService::default, |svc, _app| {
            svc.logical_by_window
                .insert(new_window, logical_window_id.clone());
        });
        if diag_enabled() && logical_window_id == AUX_LOGICAL_WINDOW_ID {
            let sender = app
                .global::<WindowBootstrapService>()
                .and_then(|svc| svc.main_window);
            app.push_effect(Effect::Window(WindowRequest::Raise {
                window: new_window,
                sender,
            }));
        }
        if diag_enabled() {
            app.request_redraw(new_window);
            app.push_effect(Effect::RequestAnimationFrame(new_window));
        }
    }
    let _ = dock_runtime::handle_dock_window_created(app, request, new_window);
}

pub(super) fn before_close_window(app: &mut KernelApp, closing_window: AppWindowId) -> bool {
    let target_window = app
        .global::<WindowBootstrapService>()
        .and_then(|svc| svc.main_window)
        .unwrap_or(closing_window);
    let _ = dock_runtime::handle_dock_before_close_window(app, closing_window, target_window);
    true
}
