use anyhow::Context as _;
use fret_app::{App, CommandId, Effect, WindowRequest};
use fret_bootstrap::ui_diagnostics::UiDiagnosticsService;
use fret_core::{AppWindowId, Event, Rect, UiServices, geometry::Px};
use fret_docking::{
    DockManager, DockPanel, DockPanelRegistry, DockPanelRegistryService, DockingRuntime,
    create_dock_space_node_with_test_id, render_and_bind_dock_panels, render_cached_panel_root,
};
use fret_launch::{
    WindowCreateSpec, WinitAppDriver, WinitCommandContext, WinitEventContext, WinitRenderContext,
    WinitRunnerConfig, WinitWindowContext,
};
use fret_runtime::PlatformCapabilities;
use fret_ui::element::{
    AnyElement, ContainerProps, LayoutQueryRegionProps, LayoutStyle, Length, SemanticsDecoration,
};
use fret_ui::retained_bridge::{LayoutCx, PaintCx, SemanticsCx, UiTreeRetainedExt as _, Widget};
use fret_ui::{ElementContext, Invalidation, Theme, UiTree};
use fret_ui_shadcn::{Field, FieldContent, FieldLabel, FieldOrientation};
use std::sync::Arc;

const INITIAL_SPLIT_FRACTION_LEFT: f32 = 0.75;
const SPLIT_ANCHOR_W: Px = Px(18.0);

struct SplitDragAnchor {
    test_id: &'static str,
}

impl SplitDragAnchor {
    fn new(test_id: &'static str) -> Self {
        Self { test_id }
    }
}

impl<H: fret_ui::UiHost> Widget<H> for SplitDragAnchor {
    fn hit_test(&self, _bounds: Rect, _position: fret_core::Point) -> bool {
        false
    }

    fn semantics(&mut self, cx: &mut SemanticsCx<'_, H>) {
        cx.set_role(fret_core::SemanticsRole::Group);
        cx.set_test_id(self.test_id);
    }
}

struct ContainerQueriesDockingHarnessRoot {
    dock_space: fret_core::NodeId,
    split_anchor: fret_core::NodeId,
}

impl<H: fret_ui::UiHost> Widget<H> for ContainerQueriesDockingHarnessRoot {
    fn layout(&mut self, cx: &mut LayoutCx<'_, H>) -> fret_core::Size {
        let bounds = cx.bounds;
        let _ = cx.layout_in(self.dock_space, bounds);

        // Position an input-transparent anchor over the initial split handle. Scripted drags
        // can target this anchor deterministically without needing docking internals to expose
        // test ids for split handles.
        let x = bounds.origin.x.0 + bounds.size.width.0 * INITIAL_SPLIT_FRACTION_LEFT;
        let x0 = x - (SPLIT_ANCHOR_W.0 * 0.5);
        let anchor_rect = Rect::new(
            fret_core::Point::new(Px(x0), bounds.origin.y),
            fret_core::Size::new(SPLIT_ANCHOR_W, bounds.size.height),
        );
        let _ = cx.layout_in(self.split_anchor, anchor_rect);

        cx.available
    }

    fn paint(&mut self, cx: &mut PaintCx<'_, H>) {
        if let Some(bounds) = cx.child_bounds(self.dock_space) {
            cx.paint(self.dock_space, bounds);
        } else {
            cx.paint(self.dock_space, cx.bounds);
        }
    }
}

struct DemoDockPanelRegistry;

impl DemoDockPanelRegistry {
    fn render_left_panel<H: fret_ui::UiHost>(
        cx: &mut ElementContext<'_, H>,
        theme: &Theme,
    ) -> Vec<AnyElement> {
        let padding = theme.metric_required("metric.padding.md");
        let background = theme.color_required("background");
        let muted = theme.color_required("muted");

        let mut fill = LayoutStyle::default();
        fill.size.width = Length::Fill;
        fill.size.height = Length::Fill;

        let region_props = LayoutQueryRegionProps {
            layout: fill,
            name: None,
        };

        vec![cx.container(
            ContainerProps {
                layout: fill,
                padding: fret_core::Edges::all(padding),
                background: Some(background),
                ..Default::default()
            },
            move |cx| {
                vec![fret_ui_kit::declarative::container_query_region_with_id(
                    cx,
                    "examples.container_queries_docking_demo.left_panel",
                    region_props,
                    move |cx, region_id| {
                        let md_breakpoint = fret_ui_kit::declarative::container_width_at_least(
                            cx,
                            region_id,
                            Invalidation::Layout,
                            true,
                            fret_ui_kit::declarative::tailwind::MD,
                            fret_ui_kit::declarative::ContainerQueryHysteresis::default(),
                        );

                        let mode_text: Arc<str> = if md_breakpoint {
                            Arc::from("Mode: md+ (container query)")
                        } else {
                            Arc::from("Mode: <md (container query)")
                        };

                        let mode_box = cx
                            .container(
                                ContainerProps {
                                    layout: {
                                        let mut layout = LayoutStyle::default();
                                        layout.size.width = Length::Px(Px(240.0));
                                        layout.size.height = Length::Px(Px(28.0));
                                        layout
                                    },
                                    padding: fret_core::Edges::all(Px(6.0)),
                                    background: Some(muted),
                                    corner_radii: fret_core::Corners::all(Px(6.0)),
                                    ..Default::default()
                                },
                                move |cx| vec![cx.text(Arc::clone(&mode_text))],
                            )
                            .attach_semantics(
                                SemanticsDecoration::default().test_id("cq-dock-demo-mode"),
                            );

                        let field_input_stub = cx.container(
                            ContainerProps {
                                layout: {
                                    let mut layout = LayoutStyle::default();
                                    layout.size.width = Length::Fill;
                                    layout.size.height = Length::Px(Px(34.0));
                                    layout
                                },
                                padding: fret_core::Edges::all(Px(8.0)),
                                background: Some(theme.color_required("secondary")),
                                corner_radii: fret_core::Corners::all(Px(6.0)),
                                ..Default::default()
                            },
                            |cx| vec![cx.text("Input stub")],
                        );

                        let field = Field::new([
                            FieldLabel::new("Name").into_element(cx),
                            FieldContent::new([field_input_stub]).into_element(cx),
                        ])
                        .orientation(FieldOrientation::Responsive)
                        .into_element(cx)
                        .attach_semantics(
                            SemanticsDecoration::default().test_id("cq-dock-demo-field"),
                        );

                        vec![mode_box, field]
                    },
                )]
            },
        )]
    }
}

impl DockPanelRegistry<App> for DemoDockPanelRegistry {
    fn render_panel(
        &self,
        ui: &mut UiTree<App>,
        app: &mut App,
        services: &mut dyn UiServices,
        window: AppWindowId,
        bounds: Rect,
        panel: &fret_core::PanelKey,
    ) -> Option<fret_core::NodeId> {
        let theme = Theme::global(&*app).clone();

        let root_name = format!("container_queries_docking_demo.panel.{}", panel.kind.0);
        Some(render_cached_panel_root(
            ui,
            app,
            services,
            window,
            bounds,
            &root_name,
            |cx| match panel.kind.0.as_str() {
                "examples.cq.left" => Self::render_left_panel(cx, &theme),
                "examples.cq.right" => vec![cx.container(
                    ContainerProps {
                        layout: {
                            let mut layout = LayoutStyle::default();
                            layout.size.width = Length::Fill;
                            layout.size.height = Length::Fill;
                            layout
                        },
                        padding: fret_core::Edges::all(theme.metric_required("metric.padding.md")),
                        background: Some(theme.color_required("background")),
                        ..Default::default()
                    },
                    |_cx| vec![],
                )],
                _ => vec![cx.text("Unregistered panel kind")],
            },
        ))
    }
}

struct ContainerQueriesDockingDemoWindowState {
    ui: UiTree<App>,
    root: Option<fret_core::NodeId>,
    dock_space: Option<fret_core::NodeId>,
}

#[derive(Default)]
struct ContainerQueriesDockingDemoDriver {
    docking_runtime: Option<DockingRuntime>,
}

impl ContainerQueriesDockingDemoDriver {
    fn build_ui(_app: &mut App, window: AppWindowId) -> ContainerQueriesDockingDemoWindowState {
        let mut ui: UiTree<App> = UiTree::new();
        ui.set_window(window);
        ui.set_view_cache_enabled(std::env::var_os("FRET_EXAMPLES_VIEW_CACHE").is_some());
        ui.set_debug_enabled(std::env::var_os("FRET_DIAG").is_some_and(|v| !v.is_empty()));
        ContainerQueriesDockingDemoWindowState {
            ui,
            root: None,
            dock_space: None,
        }
    }

    fn ensure_dock_graph(app: &mut App, window: AppWindowId) {
        use fret_core::{Axis, DockNode, PanelKey};

        app.with_global_mut(DockManager::default, |dock, _app| {
            dock.ensure_panel(&PanelKey::new("examples.cq.left"), || DockPanel {
                title: "Container queries".to_string(),
                color: fret_core::Color::TRANSPARENT,
                viewport: None,
            });
            dock.ensure_panel(&PanelKey::new("examples.cq.right"), || DockPanel {
                title: "Spacer".to_string(),
                color: fret_core::Color::TRANSPARENT,
                viewport: None,
            });

            if dock.graph.window_root(window).is_some() {
                return;
            }

            let left = dock.graph.insert_node(DockNode::Tabs {
                tabs: vec![PanelKey::new("examples.cq.left")],
                active: 0,
            });
            let right = dock.graph.insert_node(DockNode::Tabs {
                tabs: vec![PanelKey::new("examples.cq.right")],
                active: 0,
            });
            let split = dock.graph.insert_node(DockNode::Split {
                axis: Axis::Horizontal,
                children: vec![left, right],
                fractions: vec![
                    INITIAL_SPLIT_FRACTION_LEFT,
                    1.0 - INITIAL_SPLIT_FRACTION_LEFT,
                ],
            });
            dock.graph.set_window_root(window, split);
        });
    }

    fn render_dock(
        app: &mut App,
        services: &mut dyn UiServices,
        window: AppWindowId,
        state: &mut ContainerQueriesDockingDemoWindowState,
        bounds: Rect,
    ) {
        Self::ensure_dock_graph(app, window);

        let dock_space = state.dock_space.get_or_insert_with(|| {
            create_dock_space_node_with_test_id(&mut state.ui, window, "cq-dock-demo-dock-space")
        });

        if state.root.is_none() {
            let split_anchor = state
                .ui
                .create_node_retained(SplitDragAnchor::new("cq-dock-demo-split-anchor"));
            let root = state
                .ui
                .create_node_retained(ContainerQueriesDockingHarnessRoot {
                    dock_space: *dock_space,
                    split_anchor,
                });
            state.ui.set_children(root, vec![*dock_space, split_anchor]);
            state.ui.set_root(root);
            state.root = Some(root);
        }

        if state.ui.view_cache_enabled() {
            state
                .ui
                .set_node_view_cache_flags(*dock_space, true, false, false);
        }

        render_and_bind_dock_panels(&mut state.ui, app, services, window, bounds, *dock_space);
    }
}

impl WinitAppDriver for ContainerQueriesDockingDemoDriver {
    type WindowState = ContainerQueriesDockingDemoWindowState;

    fn init(&mut self, _app: &mut App, main_window: AppWindowId) {
        self.docking_runtime = Some(DockingRuntime::new(main_window));
    }

    fn create_window_state(&mut self, app: &mut App, window: AppWindowId) -> Self::WindowState {
        Self::build_ui(app, window)
    }

    fn hot_reload_window(
        &mut self,
        app: &mut App,
        _services: &mut dyn fret_core::UiServices,
        window: AppWindowId,
        state: &mut Self::WindowState,
    ) {
        crate::hotpatch::reset_ui_tree(app, window, &mut state.ui);
        state.root = None;
        state.dock_space = None;
    }

    fn handle_model_changes(
        &mut self,
        context: WinitWindowContext<'_, Self::WindowState>,
        changed: &[fret_app::ModelId],
    ) {
        context
            .app
            .with_global_mut_untracked(UiDiagnosticsService::default, |svc, _app| {
                svc.record_model_changes(context.window, changed);
            });
        context
            .state
            .ui
            .propagate_model_changes(context.app, changed);
    }

    fn handle_global_changes(
        &mut self,
        context: WinitWindowContext<'_, Self::WindowState>,
        changed: &[std::any::TypeId],
    ) {
        context
            .app
            .with_global_mut_untracked(UiDiagnosticsService::default, |svc, app| {
                svc.record_global_changes(app, context.window, changed);
            });
        context
            .state
            .ui
            .propagate_global_changes(context.app, changed);
    }

    fn handle_command(
        &mut self,
        context: WinitCommandContext<'_, Self::WindowState>,
        command: CommandId,
    ) {
        let WinitCommandContext {
            app,
            services,
            window,
            state,
        } = context;

        if state.ui.dispatch_command(app, services, &command) {
            return;
        }
        if command.as_str() == "container_queries_docking_demo.close" {
            app.push_effect(Effect::Window(WindowRequest::Close(window)));
        }
    }

    fn handle_event(&mut self, context: WinitEventContext<'_, Self::WindowState>, event: &Event) {
        let WinitEventContext {
            app,
            services,
            window,
            state,
        } = context;

        let consumed = app.with_global_mut_untracked(UiDiagnosticsService::default, |svc, app| {
            if !svc.is_enabled() {
                return false;
            }
            if svc.maybe_intercept_event_for_inspect_shortcuts(app, window, event) {
                return true;
            }
            svc.maybe_intercept_event_for_picking(app, window, event)
        });
        if consumed {
            return;
        }

        if matches!(event, Event::WindowCloseRequested) {
            app.push_effect(Effect::Window(WindowRequest::Close(window)));
            return;
        }
        state.ui.dispatch_event(app, services, event);
    }

    fn dock_op(&mut self, app: &mut App, op: fret_core::DockOp) {
        let _ = self
            .docking_runtime
            .as_ref()
            .map(|rt| rt.on_dock_op(app, op))
            .unwrap_or(false);
    }

    fn render(&mut self, context: WinitRenderContext<'_, Self::WindowState>) {
        let WinitRenderContext {
            app,
            services,
            window,
            state,
            bounds,
            scale_factor,
            scene,
        } = context;

        ContainerQueriesDockingDemoDriver::render_dock(app, services, window, state, bounds);

        state.ui.request_semantics_snapshot();
        state.ui.ingest_paint_cache_source(scene);

        let inspection_active = app
            .with_global_mut_untracked(UiDiagnosticsService::default, |svc, _app| {
                svc.wants_inspection_active(window)
            });
        state.ui.set_inspection_active(inspection_active);

        scene.clear();
        {
            let mut frame =
                fret_ui::UiFrameCx::new(&mut state.ui, app, services, window, bounds, scale_factor);
            frame.layout_all();
        }

        let semantics_snapshot = state.ui.semantics_snapshot();
        let drive = app.with_global_mut_untracked(UiDiagnosticsService::default, |svc, app| {
            let element_runtime = app.global::<fret_ui::elements::ElementRuntime>();
            svc.drive_script_for_window(
                app,
                window,
                bounds,
                scale_factor,
                Some(&state.ui),
                semantics_snapshot,
                element_runtime,
            )
        });

        for effect in drive.effects {
            app.push_effect(effect);
        }

        if drive.request_redraw {
            app.request_redraw(window);
            app.push_effect(Effect::RequestAnimationFrame(window));
        }

        let mut injected_any = false;
        for event in drive.events {
            injected_any = true;
            state.ui.dispatch_event(app, services, &event);
        }

        if injected_any {
            let mut deferred_effects: Vec<Effect> = Vec::new();
            loop {
                let effects = app.flush_effects();
                if effects.is_empty() {
                    break;
                }

                let mut applied_any_command = false;
                for effect in effects {
                    match effect {
                        Effect::Command { window: w, command } => {
                            if w.is_none() || w == Some(window) {
                                let _ = state.ui.dispatch_command(app, services, &command);
                                applied_any_command = true;
                            } else {
                                deferred_effects.push(Effect::Command { window: w, command });
                            }
                        }
                        other => deferred_effects.push(other),
                    }
                }

                if !applied_any_command {
                    break;
                }
            }
            for effect in deferred_effects {
                app.push_effect(effect);
            }

            state.ui.request_semantics_snapshot();
            let mut frame =
                fret_ui::UiFrameCx::new(&mut state.ui, app, services, window, bounds, scale_factor);
            frame.layout_all();
        }

        let mut frame =
            fret_ui::UiFrameCx::new(&mut state.ui, app, services, window, bounds, scale_factor);
        frame.paint_all(scene);

        app.with_global_mut_untracked(UiDiagnosticsService::default, |svc, app| {
            let element_runtime = app.global::<fret_ui::elements::ElementRuntime>();
            svc.record_snapshot(
                app,
                window,
                bounds,
                scale_factor,
                &state.ui,
                element_runtime,
                scene,
            );
            let _ = svc.maybe_dump_if_triggered();
            if svc.is_enabled() {
                app.push_effect(Effect::RequestAnimationFrame(window));
            }
        });
    }

    fn window_create_spec(
        &mut self,
        _app: &mut App,
        _request: &fret_app::CreateWindowRequest,
    ) -> Option<WindowCreateSpec> {
        None
    }

    fn window_created(
        &mut self,
        app: &mut App,
        request: &fret_app::CreateWindowRequest,
        new_window: AppWindowId,
    ) {
        let _ = self
            .docking_runtime
            .as_ref()
            .map(|rt| rt.on_window_created(app, request, new_window))
            .unwrap_or(false);
    }

    fn before_close_window(&mut self, app: &mut App, window: AppWindowId) -> bool {
        let _ = self
            .docking_runtime
            .as_ref()
            .map(|rt| rt.before_close_window(app, window))
            .unwrap_or(false);
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
                .add_directive("fret_launch=info".parse().unwrap()),
        )
        .try_init();

    let mut app = App::new();
    app.set_global(PlatformCapabilities::default());

    app.with_global_mut(DockPanelRegistryService::<App>::default, |svc, _app| {
        svc.set(Arc::new(DemoDockPanelRegistry));
    });

    let config = WinitRunnerConfig {
        main_window_title: "fret-demo container_queries_docking_demo".to_string(),
        // Ensure the left panel starts above md (>=768px) and can be dragged below it.
        main_window_size: winit::dpi::LogicalSize::new(1400.0, 760.0),
        ..Default::default()
    };

    let driver = ContainerQueriesDockingDemoDriver::default();
    fret_kit::run_native_demo(config, app, driver).context("run container_queries_docking_demo app")
}
