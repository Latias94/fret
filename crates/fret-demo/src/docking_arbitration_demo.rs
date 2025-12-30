use anyhow::Context as _;
use fret_app::CreateWindowKind;
use fret_app::{App, CommandId, Effect, Model, WindowRequest};
use fret_components_docking::dock::DockPanelContentService;
use fret_components_docking::{
    DockManager, DockPanel, DockViewportOverlayHooks, DockViewportOverlayHooksService,
    handle_dock_before_close_window, handle_dock_op, handle_dock_window_created,
};
use fret_components_icons::IconRegistry;
use fret_components_shadcn as shadcn;
use fret_core::{
    AppWindowId, Color, Corners, DrawOrder, Edges, Event, PlatformCapabilities, Rect, Scene,
    SceneOp, UiServices, ViewportInputEvent, geometry::Px,
};
use fret_runner_winit_wgpu::{WindowCreateSpec, WinitDriver, WinitRunner, WinitRunnerConfig};
use fret_ui::declarative;
use fret_ui::element::{ContainerProps, LayoutStyle, Length};
use fret_ui::{Invalidation, Theme, UiTree};
use std::collections::{HashMap, HashSet};
use std::sync::Arc;
use winit::event_loop::EventLoop;

#[derive(Default)]
struct ViewportDebugService {
    last_event: HashMap<AppWindowId, Model<Arc<str>>>,
}

struct DemoViewportOverlayHooks;

impl DockViewportOverlayHooks for DemoViewportOverlayHooks {
    fn paint(
        &self,
        theme: fret_ui::ThemeSnapshot,
        _window: AppWindowId,
        _panel: &fret_core::PanelKey,
        _viewport: fret_components_docking::ViewportPanel,
        _mapping: fret_core::ViewportMapping,
        draw_rect: Rect,
        scene: &mut Scene,
    ) {
        let border_color = Color {
            a: 0.80,
            ..theme.colors.accent
        };
        scene.push(SceneOp::Quad {
            order: DrawOrder(6),
            rect: draw_rect,
            background: Color::TRANSPARENT,
            border: Edges::all(Px(2.0)),
            border_color,
            corner_radii: Corners::all(Px(0.0)),
        });
    }
}

struct DockingArbitrationWindowState {
    ui: UiTree<App>,
    root: Option<fret_core::NodeId>,
    popover_open: Model<bool>,
    dialog_open: Model<bool>,
    last_viewport_input: Model<Arc<str>>,
}

#[derive(Default)]
struct DockingArbitrationDriver {
    main_window: Option<AppWindowId>,
    pending_layout: Option<fret_core::DockLayout>,
    restore: Option<DockLayoutRestoreState>,
    logical_windows: HashMap<AppWindowId, String>,
    next_logical_window_ix: u32,
}

struct DockLayoutRestoreState {
    layout: fret_core::DockLayout,
    pending_logical_window_ids: HashSet<String>,
}

impl DockingArbitrationDriver {
    const DOCK_LAYOUT_PATH: &'static str = ".fret/layout.json";
    const MAIN_LOGICAL_WINDOW_ID: &'static str = "main";

    fn new(pending_layout: Option<fret_core::DockLayout>) -> Self {
        let mut next_logical_window_ix = 1;
        if let Some(layout) = &pending_layout {
            for w in &layout.windows {
                let Some(suffix) = w.logical_window_id.strip_prefix("floating-") else {
                    continue;
                };
                let Ok(ix) = suffix.parse::<u32>() else {
                    continue;
                };
                next_logical_window_ix = next_logical_window_ix.max(ix.saturating_add(1));
            }
        }
        Self {
            main_window: None,
            pending_layout,
            restore: None,
            logical_windows: HashMap::new(),
            next_logical_window_ix,
        }
    }

    fn alloc_floating_logical_window_id(&mut self) -> String {
        let reserved = self.restore.as_ref().map(|r| &r.pending_logical_window_ids);

        loop {
            let logical = format!("floating-{}", self.next_logical_window_ix);
            self.next_logical_window_ix = self.next_logical_window_ix.saturating_add(1);

            if self.logical_windows.values().any(|v| v == &logical) {
                continue;
            }
            if reserved.is_some_and(|r| r.contains(&logical)) {
                continue;
            }
            return logical;
        }
    }

    fn build_ui(app: &mut App, window: AppWindowId) -> DockingArbitrationWindowState {
        let popover_open = app.models_mut().insert(false);
        let dialog_open = app.models_mut().insert(false);
        let last_viewport_input = app.models_mut().insert(Arc::<str>::from("<none>"));

        app.with_global_mut(ViewportDebugService::default, |svc, _app| {
            svc.last_event.insert(window, last_viewport_input);
        });

        let mut ui: UiTree<App> = UiTree::new();
        ui.set_window(window);

        DockingArbitrationWindowState {
            ui,
            root: None,
            popover_open,
            dialog_open,
            last_viewport_input,
        }
    }

    fn ensure_dock_graph(app: &mut App, window: AppWindowId) {
        use fret_core::{DockNode, PanelKey};

        app.with_global_mut(DockManager::default, |dock, _app| {
            let viewport_panel = PanelKey::new("demo.viewport");
            let controls_panel = PanelKey::new("demo.controls");

            dock.ensure_panel(&viewport_panel, || DockPanel {
                title: "Viewport".to_string(),
                color: Color::TRANSPARENT,
                viewport: Some(fret_components_docking::ViewportPanel {
                    target: fret_core::RenderTargetId::default(),
                    target_px_size: (960, 540),
                    fit: fret_core::ViewportFit::Contain,
                    context_menu_enabled: true,
                }),
            });
            dock.ensure_panel(&controls_panel, || DockPanel {
                title: "Controls".to_string(),
                color: Color::TRANSPARENT,
                viewport: None,
            });

            if dock.graph.window_root(window).is_some() {
                return;
            }

            let tabs = dock.graph.insert_node(DockNode::Tabs {
                tabs: vec![viewport_panel, controls_panel],
                active: 0,
            });
            dock.graph.set_window_root(window, tabs);
        });
    }

    fn apply_layout_if_ready(&mut self, app: &mut App) {
        let Some(main_window) = self.main_window else {
            return;
        };
        let Some(restore) = self.restore.as_mut() else {
            return;
        };
        if !restore.pending_logical_window_ids.is_empty() {
            return;
        }

        let mut windows: Vec<(AppWindowId, String)> = self
            .logical_windows
            .iter()
            .map(|(w, id)| (*w, id.clone()))
            .collect();
        windows.sort_by(|a, b| a.1.cmp(&b.1));

        app.with_global_mut(DockManager::default, |dock, app| {
            let changed = dock
                .graph
                .import_layout_for_windows(&restore.layout, &windows);
            if changed {
                fret_components_docking::runtime::request_dock_invalidation(
                    app,
                    dock.graph.windows(),
                );
                for w in dock.graph.windows() {
                    app.request_redraw(w);
                }
            }
        });

        self.restore = None;
        app.request_redraw(main_window);
    }

    fn try_restore_layout_on_init(&mut self, app: &mut App, main_window: AppWindowId) {
        let Some(layout) = self.pending_layout.take() else {
            return;
        };

        let multi_window = app
            .global::<PlatformCapabilities>()
            .map(|c| c.ui.multi_window)
            .unwrap_or(true);

        if !multi_window {
            app.with_global_mut(DockManager::default, |dock, app| {
                let changed = dock
                    .graph
                    .import_layout_for_windows_with_fallback_floatings(
                        &layout,
                        &[(main_window, Self::MAIN_LOGICAL_WINDOW_ID.to_string())],
                        main_window,
                    );
                if changed {
                    fret_components_docking::runtime::request_dock_invalidation(app, [main_window]);
                    app.request_redraw(main_window);
                }
            });
            return;
        }

        // Multi-window restore (best-effort): create OS windows for non-main logical windows, then
        // import the full layout once all windows exist. Until then, main window can still render
        // a default dock graph.
        let mut pending: HashSet<String> = HashSet::new();
        for w in &layout.windows {
            if w.logical_window_id == Self::MAIN_LOGICAL_WINDOW_ID {
                continue;
            }
            pending.insert(w.logical_window_id.clone());
            app.push_effect(Effect::Window(WindowRequest::Create(
                fret_app::CreateWindowRequest {
                    kind: CreateWindowKind::DockRestore {
                        logical_window_id: w.logical_window_id.clone(),
                    },
                    anchor: None,
                },
            )));
        }
        self.restore = Some(DockLayoutRestoreState {
            layout,
            pending_logical_window_ids: pending,
        });
        self.apply_layout_if_ready(app);
    }

    fn save_layout_on_exit(&mut self, app: &mut App) {
        let Some(main_window) = self.main_window else {
            return;
        };

        let mut windows: Vec<(AppWindowId, String)> = self
            .logical_windows
            .iter()
            .map(|(w, id)| (*w, id.clone()))
            .collect();
        windows.sort_by(|a, b| a.1.cmp(&b.1));

        let Some(metrics) = app.global::<fret_core::WindowMetricsService>() else {
            return;
        };

        let placements: HashMap<AppWindowId, fret_core::DockWindowPlacement> = windows
            .iter()
            .filter_map(|(window, _logical_window_id)| {
                let size = metrics.inner_size(*window)?;
                let width = (size.width.0.max(1.0).round() as u32).max(1);
                let height = (size.height.0.max(1.0).round() as u32).max(1);
                Some((
                    *window,
                    fret_core::DockWindowPlacement {
                        width,
                        height,
                        x: None,
                        y: None,
                        monitor_hint: None,
                    },
                ))
            })
            .collect();

        let layout = app.with_global_mut(DockManager::default, |dock, _app| {
            dock.graph
                .export_layout_with_placement(&windows, |window| placements.get(&window).cloned())
        });

        let file = fret_app::DockLayoutFileV1 { layout };
        if let Err(err) = file.save_json(Self::DOCK_LAYOUT_PATH) {
            tracing::warn!("failed to save dock layout: {err}");
        } else {
            app.request_redraw(main_window);
        }
    }

    fn render_dock(
        app: &mut App,
        services: &mut dyn UiServices,
        window: AppWindowId,
        state: &mut DockingArbitrationWindowState,
        bounds: Rect,
    ) {
        Self::ensure_dock_graph(app, window);

        let captured = format!("captured={:?}", state.ui.captured());
        let layer_lines: Vec<String> = state
            .ui
            .debug_layers_in_paint_order()
            .iter()
            .enumerate()
            .map(|(ix, layer)| {
                format!(
                    "#{ix} root={:?} visible={} barrier={} hit_testable={} outside={} move={} timer={}",
                    layer.root,
                    layer.visible,
                    layer.blocks_underlay_input,
                    layer.hit_testable,
                    layer.wants_pointer_down_outside_events,
                    layer.wants_pointer_move_events,
                    layer.wants_timer_events
                )
            })
            .collect();

        fret_components_ui::window_overlays::begin_frame(app, window);

        let dock_space = state.root.get_or_insert_with(|| {
            let node = fret_components_docking::create_dock_space_node(&mut state.ui, window);
            state.ui.set_root(node);
            node
        });

        let theme = Theme::global(&*app).clone();
        let padding = theme.metrics.padding_md;
        let background = theme.colors.surface_background;

        let popover_open = state.popover_open;
        let dialog_open = state.dialog_open;
        let last_viewport_input = state.last_viewport_input;

        let controls = declarative::render_root(
            &mut state.ui,
            app,
            services,
            window,
            bounds,
            "dock.panel.controls",
            |cx| {
                cx.observe_model(popover_open, Invalidation::Layout);
                cx.observe_model(dialog_open, Invalidation::Layout);
                cx.observe_model(last_viewport_input, Invalidation::Layout);

                let drag_state = cx
                    .app
                    .drag()
                    .map(|d| format!("drag(kind={:?}, dragging={})", d.kind, d.dragging))
                    .unwrap_or_else(|| "drag(<none>)".to_string());

                let last = cx
                    .app
                    .models()
                    .get(last_viewport_input)
                    .cloned()
                    .unwrap_or_else(|| Arc::<str>::from("<missing>"));

                let popover = shadcn::Popover::new(popover_open)
                    .auto_focus(true)
                    .into_element(
                        cx,
                        |cx| {
                            shadcn::Button::new("Open popover")
                                .variant(shadcn::ButtonVariant::Outline)
                                .toggle_model(popover_open)
                                .into_element(cx)
                        },
                        |cx| {
                            shadcn::PopoverContent::new(vec![
                                cx.text("Non-modal overlay (Popover)."),
                                shadcn::Button::new("Close")
                                    .variant(shadcn::ButtonVariant::Secondary)
                                    .toggle_model(popover_open)
                                    .into_element(cx),
                            ])
                            .into_element(cx)
                        },
                    );

                let dialog = shadcn::Dialog::new(dialog_open).into_element(
                    cx,
                    |cx| {
                        shadcn::Button::new("Open modal dialog")
                            .variant(shadcn::ButtonVariant::Outline)
                            .toggle_model(dialog_open)
                            .into_element(cx)
                    },
                    |cx| {
                        shadcn::DialogContent::new(vec![
                            shadcn::DialogHeader::new(vec![
                                shadcn::DialogTitle::new("Dialog").into_element(cx),
                                shadcn::DialogDescription::new(
                                    "Modal barrier should block docking + viewport input.",
                                )
                                .into_element(cx),
                            ])
                            .into_element(cx),
                            shadcn::DialogFooter::new(vec![
                                shadcn::Button::new("Close")
                                    .variant(shadcn::ButtonVariant::Secondary)
                                    .toggle_model(dialog_open)
                                    .into_element(cx),
                            ])
                            .into_element(cx),
                        ])
                        .into_element(cx)
                    },
                );

                let mut children = Vec::new();
                children.push(cx.container(
                    ContainerProps {
                        layout: {
                            let mut layout = LayoutStyle::default();
                            layout.size.width = Length::Fill;
                            layout.size.height = Length::Fill;
                            layout
                        },
                        padding: fret_core::Edges::all(padding),
                        background: Some(background),
                        ..Default::default()
                    },
                    |cx| {
                        let mut rows = Vec::new();
                        rows.push(cx.text("Docking arbitration demo (ADR 0072)"));
                        rows.push(cx.text(
                            "Open a popover, then drag a dock tab; start viewport drag inside the blue border; open a modal to block underlay.",
                        ));
                        rows.push(cx.text(drag_state));
                        rows.push(cx.text(captured.clone()));
                        rows.push(cx.text(format!("last_viewport_input={last}")));
                        rows.push(popover);
                        rows.push(dialog);
                        rows.push(cx.text("Layers (paint order):"));
                        for line in layer_lines.iter().cloned() {
                            rows.push(cx.text(line));
                        }
                        rows
                    },
                ));
                children
            },
        );

        state.ui.set_children(
            *dock_space,
            vec![controls /* viewport panel has no UI node */],
        );

        app.with_global_mut(DockPanelContentService::default, |svc, _app| {
            svc.set(window, fret_core::PanelKey::new("demo.controls"), controls);
        });

        fret_components_ui::window_overlays::render(&mut state.ui, app, services, window, bounds);
    }
}

impl WinitDriver for DockingArbitrationDriver {
    type WindowState = DockingArbitrationWindowState;

    fn init(&mut self, app: &mut App, main_window: AppWindowId) {
        self.main_window = Some(main_window);
        self.logical_windows
            .insert(main_window, Self::MAIN_LOGICAL_WINDOW_ID.to_string());
        self.try_restore_layout_on_init(app, main_window);
    }

    fn create_window_state(&mut self, app: &mut App, window: AppWindowId) -> Self::WindowState {
        Self::build_ui(app, window)
    }

    fn handle_model_changes(
        &mut self,
        app: &mut App,
        _window: AppWindowId,
        state: &mut Self::WindowState,
        changed: &[fret_app::ModelId],
    ) {
        state.ui.propagate_model_changes(app, changed);
    }

    fn handle_command(
        &mut self,
        app: &mut App,
        services: &mut dyn UiServices,
        _window: AppWindowId,
        state: &mut Self::WindowState,
        command: CommandId,
    ) {
        if state.ui.dispatch_command(app, services, &command) {
            return;
        }
    }

    fn handle_event(
        &mut self,
        app: &mut App,
        services: &mut dyn UiServices,
        window: AppWindowId,
        state: &mut Self::WindowState,
        event: &Event,
    ) {
        if matches!(event, Event::WindowCloseRequested) {
            app.push_effect(Effect::Window(WindowRequest::Close(window)));
            return;
        }
        state.ui.dispatch_event(app, services, event);
    }

    fn viewport_input(&mut self, app: &mut App, event: ViewportInputEvent) {
        let msg: Arc<str> = Arc::from(
            format!(
                "{:?} uv=({:.3},{:.3}) target_px=({}, {}) window={:?}",
                event.kind,
                event.uv.0,
                event.uv.1,
                event.target_px.0,
                event.target_px.1,
                event.window,
            )
            .into_boxed_str(),
        );
        app.with_global_mut(ViewportDebugService::default, |svc, app| {
            if let Some(model) = svc.last_event.get(&event.window).copied() {
                let _ = app.models_mut().update(model, |v| *v = msg.clone());
                app.request_redraw(event.window);
            }
        });
    }

    fn dock_op(&mut self, app: &mut App, op: fret_core::DockOp) {
        let _ = handle_dock_op(app, op);
    }

    fn render(
        &mut self,
        app: &mut App,
        window: AppWindowId,
        state: &mut Self::WindowState,
        bounds: Rect,
        scale_factor: f32,
        services: &mut dyn UiServices,
        scene: &mut Scene,
    ) {
        DockingArbitrationDriver::render_dock(app, services, window, state, bounds);

        state.ui.request_semantics_snapshot();
        state.ui.ingest_paint_cache_source(scene);
        scene.clear();
        state.ui.layout_all(app, services, bounds, scale_factor);
        state
            .ui
            .paint_all(app, services, bounds, scene, scale_factor);
    }

    fn window_create_spec(
        &mut self,
        _app: &mut App,
        request: &fret_app::CreateWindowRequest,
    ) -> Option<WindowCreateSpec> {
        match &request.kind {
            CreateWindowKind::DockFloating { panel, .. } => Some(WindowCreateSpec::new(
                format!("fret-demo docking_arbitration_demo — {}", panel.kind.0),
                winit::dpi::LogicalSize::new(720.0, 520.0),
            )),
            CreateWindowKind::DockRestore { logical_window_id } => {
                let mut size = winit::dpi::LogicalSize::new(980.0, 720.0);
                if let Some(restore) = &self.restore
                    && let Some(window) = restore
                        .layout
                        .windows
                        .iter()
                        .find(|w| w.logical_window_id == logical_window_id.as_str())
                    && let Some(p) = &window.placement
                {
                    size = winit::dpi::LogicalSize::new(p.width as f64, p.height as f64);
                }
                Some(WindowCreateSpec::new(
                    format!("fret-demo docking_arbitration_demo — {logical_window_id}"),
                    size,
                ))
            }
        }
    }

    fn window_created(
        &mut self,
        app: &mut App,
        request: &fret_app::CreateWindowRequest,
        new_window: AppWindowId,
    ) {
        match &request.kind {
            CreateWindowKind::DockFloating { .. } => {
                let _ = handle_dock_window_created(app, request, new_window);
                let logical = self.alloc_floating_logical_window_id();
                self.logical_windows.insert(new_window, logical);
            }
            CreateWindowKind::DockRestore { logical_window_id } => {
                self.logical_windows
                    .insert(new_window, logical_window_id.clone());
                if let Some(restore) = self.restore.as_mut() {
                    restore.pending_logical_window_ids.remove(logical_window_id);
                }
                self.apply_layout_if_ready(app);
            }
        }
    }

    fn before_close_window(&mut self, app: &mut App, window: AppWindowId) -> bool {
        if Some(window) == self.main_window {
            self.save_layout_on_exit(app);
        } else {
            self.logical_windows.remove(&window);
        }

        if let Some(main_window) = self.main_window {
            let _ = handle_dock_before_close_window(app, window, main_window);
        }
        true
    }

    fn accessibility_snapshot(
        &mut self,
        _app: &mut App,
        _window: AppWindowId,
        state: &mut Self::WindowState,
    ) -> Option<Arc<fret_core::SemanticsSnapshot>> {
        state.ui.semantics_snapshot_arc()
    }

    fn accessibility_focus(
        &mut self,
        _app: &mut App,
        _window: AppWindowId,
        state: &mut Self::WindowState,
        target: fret_core::NodeId,
    ) {
        state.ui.set_focus(Some(target));
    }

    fn accessibility_invoke(
        &mut self,
        app: &mut App,
        services: &mut dyn UiServices,
        _window: AppWindowId,
        state: &mut Self::WindowState,
        target: fret_core::NodeId,
    ) {
        fret_ui_app::accessibility_actions::invoke(&mut state.ui, app, services, target);
    }

    fn accessibility_set_value_text(
        &mut self,
        app: &mut App,
        services: &mut dyn UiServices,
        _window: AppWindowId,
        state: &mut Self::WindowState,
        target: fret_core::NodeId,
        value: &str,
    ) {
        fret_ui_app::accessibility_actions::set_value_text(
            &mut state.ui,
            app,
            services,
            target,
            value,
        );
    }

    fn accessibility_set_value_numeric(
        &mut self,
        app: &mut App,
        services: &mut dyn UiServices,
        _window: AppWindowId,
        state: &mut Self::WindowState,
        target: fret_core::NodeId,
        value: f64,
    ) {
        fret_ui_app::accessibility_actions::set_value_numeric(
            &mut state.ui,
            app,
            services,
            target,
            value,
        );
    }

    fn accessibility_set_text_selection(
        &mut self,
        app: &mut App,
        services: &mut dyn UiServices,
        _window: AppWindowId,
        state: &mut Self::WindowState,
        target: fret_core::NodeId,
        anchor: u32,
        focus: u32,
    ) {
        fret_ui_app::accessibility_actions::set_text_selection(
            &mut state.ui,
            app,
            services,
            target,
            anchor,
            focus,
        );
    }

    fn accessibility_replace_selected_text(
        &mut self,
        app: &mut App,
        services: &mut dyn UiServices,
        _window: AppWindowId,
        state: &mut Self::WindowState,
        target: fret_core::NodeId,
        value: &str,
    ) {
        fret_ui_app::accessibility_actions::replace_selected_text(
            &mut state.ui,
            app,
            services,
            target,
            value,
        );
    }
}

pub fn run() -> anyhow::Result<()> {
    let _ = tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::from_default_env()
                .add_directive("fret=info".parse().unwrap())
                .add_directive("fret_render=info".parse().unwrap())
                .add_directive("fret_runner_winit_wgpu=info".parse().unwrap()),
        )
        .try_init();

    let event_loop = EventLoop::new().context("create winit event loop")?;
    let mut app = App::new();
    let mut caps = PlatformCapabilities::default();
    if std::env::var("FRET_SINGLE_WINDOW")
        .ok()
        .is_some_and(|v| v == "1" || v.eq_ignore_ascii_case("true"))
    {
        caps.ui.multi_window = false;
        caps.ui.window_tear_off = true;
    }
    app.set_global(caps);
    app.with_global_mut(IconRegistry::default, |icons, _app| {
        fret_icons_lucide::register_icons(icons);
    });
    app.with_global_mut(DockViewportOverlayHooksService::default, |svc, _app| {
        svc.set(Arc::new(DemoViewportOverlayHooks));
    });

    let mut config = WinitRunnerConfig {
        main_window_title: "fret-demo docking_arbitration_demo".to_string(),
        main_window_size: winit::dpi::LogicalSize::new(980.0, 720.0),
        ..Default::default()
    };

    if let Some(settings) = fret_app::SettingsFileV1::load_json_if_exists(".fret/settings.json")
        .context("load .fret/settings.json")?
    {
        app.set_global(settings.docking_interaction_settings());
        config.text_font_families.ui_sans = settings.fonts.ui_sans;
        config.text_font_families.ui_serif = settings.fonts.ui_serif;
        config.text_font_families.ui_mono = settings.fonts.ui_mono;
    }

    let pending_layout =
        fret_app::DockLayoutFileV1::load_json_if_exists(DockingArbitrationDriver::DOCK_LAYOUT_PATH)
            .map(|v| v.map(|f| f.layout))
            .unwrap_or_else(|err| {
                tracing::warn!("failed to load dock layout: {err}");
                None
            });

    let driver = DockingArbitrationDriver::new(pending_layout);
    let mut runner = WinitRunner::new(config, app, driver);
    event_loop.run_app(&mut runner)?;
    Ok(())
}
