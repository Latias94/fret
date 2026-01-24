use anyhow::Context as _;
use fret_app::{App, CommandId, Effect, WindowRequest};
use fret_bootstrap::ui_diagnostics::UiDiagnosticsService;
use fret_core::{
    AppWindowId, Color, Corners, DrawOrder, Edges, Event, Rect, Scene, SceneOp, UiServices,
    geometry::Px,
};
use fret_docking::{
    DockManager, DockPanel, DockPanelRegistry, DockPanelRegistryService, DockViewportOverlayHooks,
    DockViewportOverlayHooksService, create_dock_space_node_with_test_id,
    handle_dock_before_close_window, handle_dock_op, handle_dock_window_created,
    render_and_bind_dock_panels, render_cached_panel_root,
};
use fret_launch::{
    WindowCreateSpec, WinitAppDriver, WinitCommandContext, WinitEventContext, WinitRenderContext,
    WinitRunnerConfig, WinitWindowContext,
};
use fret_runtime::PlatformCapabilities;
use fret_ui::element::{ContainerProps, LayoutStyle, Length};
use fret_ui::retained_bridge::{LayoutCx, PaintCx, SemanticsCx, UiTreeRetainedExt as _, Widget};
use fret_ui::{Theme, UiTree};
use std::sync::Arc;

const DOCKING_DEMO_TAB_BAR_H: Px = Px(28.0);
const DOCKING_DEMO_DRAG_ANCHOR_SIZE: Px = Px(12.0);

struct DockingDemoDragAnchor {
    test_id: &'static str,
}

impl DockingDemoDragAnchor {
    fn new(test_id: &'static str) -> Self {
        Self { test_id }
    }
}

impl<H: fret_ui::UiHost> Widget<H> for DockingDemoDragAnchor {
    fn hit_test(&self, _bounds: Rect, _position: fret_core::Point) -> bool {
        false
    }

    fn semantics(&mut self, cx: &mut SemanticsCx<'_, H>) {
        cx.set_role(fret_core::SemanticsRole::Group);
        cx.set_test_id(self.test_id);
    }
}

struct DockingDemoHarnessRoot {
    dock_space: fret_core::NodeId,
    left_anchor: fret_core::NodeId,
    right_anchor: fret_core::NodeId,
}

impl<H: fret_ui::UiHost> Widget<H> for DockingDemoHarnessRoot {
    fn layout(&mut self, cx: &mut LayoutCx<'_, H>) -> fret_core::Size {
        let bounds = cx.bounds;

        let _ = cx.layout_in(self.dock_space, bounds);

        let x_l = bounds.origin.x.0 + bounds.size.width.0 * 0.25;
        let x_r = bounds.origin.x.0 + bounds.size.width.0 * 0.75;
        let y = bounds.origin.y.0 + (DOCKING_DEMO_TAB_BAR_H.0 * 0.5);

        let half = DOCKING_DEMO_DRAG_ANCHOR_SIZE.0 * 0.5;
        let rect = |x: f32| {
            Rect::new(
                fret_core::Point::new(Px((x - half).max(bounds.origin.x.0)), Px(y - half)),
                fret_core::Size::new(DOCKING_DEMO_DRAG_ANCHOR_SIZE, DOCKING_DEMO_DRAG_ANCHOR_SIZE),
            )
        };

        let _ = cx.layout_in(self.left_anchor, rect(x_l));
        let _ = cx.layout_in(self.right_anchor, rect(x_r));

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
        let padding = theme.metric_required("metric.padding.md");
        let background = theme.color_required("background");

        let label: &str = match panel.kind.0.as_str() {
            "core.hierarchy" => "Hierarchy panel (declarative root)",
            "core.inspector" => "Inspector panel (declarative root)",
            _ => "Panel (unregistered kind)",
        };

        let root_name = format!("dock_demo.panel.{}", panel.kind.0);
        Some(render_cached_panel_root(
            ui,
            app,
            services,
            window,
            bounds,
            &root_name,
            |cx| {
                vec![cx.container(
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
                    |cx| vec![cx.text(label)],
                )]
            },
        ))
    }
}

struct DemoViewportOverlayHooks;

impl DockViewportOverlayHooks for DemoViewportOverlayHooks {
    fn paint_with_layout(
        &self,
        theme: fret_ui::ThemeSnapshot,
        _window: AppWindowId,
        _panel: &fret_core::PanelKey,
        _viewport: fret_docking::ViewportPanel,
        layout: fret_docking::DockViewportLayout,
        scene: &mut Scene,
    ) {
        let border_color = Color {
            a: 0.65,
            ..theme.color_required("primary")
        };
        let draw_rect = layout.draw_rect;
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

struct DockingDemoWindowState {
    ui: UiTree<App>,
    root: Option<fret_core::NodeId>,
    dock_space: Option<fret_core::NodeId>,
}

#[derive(Default)]
struct DockingDemoDriver {
    main_window: Option<AppWindowId>,
}

impl DockingDemoDriver {
    fn build_ui(_app: &mut App, window: AppWindowId) -> DockingDemoWindowState {
        let mut ui: UiTree<App> = UiTree::new();
        ui.set_window(window);
        ui.set_view_cache_enabled(std::env::var_os("FRET_EXAMPLES_VIEW_CACHE").is_some());
        ui.set_debug_enabled(std::env::var_os("FRET_DIAG").is_some_and(|v| !v.is_empty()));
        DockingDemoWindowState {
            ui,
            root: None,
            dock_space: None,
        }
    }

    fn ensure_dock_graph(app: &mut App, window: AppWindowId) {
        use fret_core::{Axis, DockNode, PanelKey};

        app.with_global_mut(DockManager::default, |dock, _app| {
            dock.ensure_panel(&PanelKey::new("core.hierarchy"), || DockPanel {
                title: "Hierarchy".to_string(),
                color: fret_core::Color::TRANSPARENT,
                viewport: None,
            });
            dock.ensure_panel(&PanelKey::new("core.inspector"), || DockPanel {
                title: "Inspector".to_string(),
                color: fret_core::Color::TRANSPARENT,
                viewport: None,
            });

            if dock.graph.window_root(window).is_some() {
                return;
            }

            let left = dock.graph.insert_node(DockNode::Tabs {
                tabs: vec![PanelKey::new("core.hierarchy")],
                active: 0,
            });
            let right = dock.graph.insert_node(DockNode::Tabs {
                tabs: vec![PanelKey::new("core.inspector")],
                active: 0,
            });
            let split = dock.graph.insert_node(DockNode::Split {
                axis: Axis::Horizontal,
                children: vec![left, right],
                fractions: vec![0.5, 0.5],
            });
            dock.graph.set_window_root(window, split);
        });
    }

    fn render_dock(
        app: &mut App,
        services: &mut dyn UiServices,
        window: AppWindowId,
        state: &mut DockingDemoWindowState,
        bounds: Rect,
    ) {
        Self::ensure_dock_graph(app, window);

        let dock_space = state.dock_space.get_or_insert_with(|| {
            create_dock_space_node_with_test_id(&mut state.ui, window, "dock-demo-dock-space")
        });

        if state.root.is_none() {
            let left_anchor = state
                .ui
                .create_node_retained(DockingDemoDragAnchor::new("dock-demo-tab-drag-anchor-left"));
            let right_anchor = state.ui.create_node_retained(DockingDemoDragAnchor::new(
                "dock-demo-tab-drag-anchor-right",
            ));
            let root = state.ui.create_node_retained(DockingDemoHarnessRoot {
                dock_space: *dock_space,
                left_anchor,
                right_anchor,
            });
            state
                .ui
                .set_children(root, vec![*dock_space, left_anchor, right_anchor]);
            state.ui.set_root(root);
            state.root = Some(root);
        }

        // When view caching is active, explicitly mark the dock space as a cache root so paint
        // caching + prepaint hooks are exercised in the same mode as UI Gallery shell caching.
        if state.ui.view_cache_enabled() {
            state
                .ui
                .set_node_view_cache_flags(*dock_space, true, false, false);
        }

        render_and_bind_dock_panels(&mut state.ui, app, services, window, bounds, *dock_space);
    }
}

impl WinitAppDriver for DockingDemoDriver {
    type WindowState = DockingDemoWindowState;

    fn init(&mut self, _app: &mut App, main_window: AppWindowId) {
        self.main_window = Some(main_window);
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
        if command.as_str() == "dock_demo.close" {
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
        let _ = handle_dock_op(app, op);
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

        DockingDemoDriver::render_dock(app, services, window, state, bounds);

        state.ui.request_semantics_snapshot();
        state.ui.ingest_paint_cache_source(scene);

        let inspection_active = app
            .with_global_mut_untracked(UiDiagnosticsService::default, |svc, _app| {
                svc.wants_inspection_active(window)
            });
        state.ui.set_inspection_active(inspection_active);

        scene.clear();
        let mut frame =
            fret_ui::UiFrameCx::new(&mut state.ui, app, services, window, bounds, scale_factor);
        frame.layout_all();

        let semantics_snapshot = state.ui.semantics_snapshot();
        let drive = app.with_global_mut_untracked(UiDiagnosticsService::default, |svc, app| {
            let element_runtime = app.global::<fret_ui::elements::ElementRuntime>();
            svc.drive_script_for_window(app, window, semantics_snapshot, element_runtime)
        });

        if drive.request_redraw {
            app.request_redraw(window);
            // Script-driven `wait_frames` needs a reliable way to advance frames even when the
            // scene is otherwise idle. Requesting an animation frame ensures the runner
            // schedules another render tick.
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
        let _ = handle_dock_window_created(app, request, new_window);
    }

    fn before_close_window(&mut self, app: &mut App, window: AppWindowId) -> bool {
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
                .add_directive("fret_launch=info".parse().unwrap()),
        )
        .try_init();

    let mut app = App::new();
    app.set_global(PlatformCapabilities::default());
    app.with_global_mut(DockPanelRegistryService::<App>::default, |svc, _app| {
        svc.set(Arc::new(DemoDockPanelRegistry));
    });
    app.with_global_mut(DockViewportOverlayHooksService::default, |svc, _app| {
        svc.set(Arc::new(DemoViewportOverlayHooks));
    });

    let config = WinitRunnerConfig {
        main_window_title: "fret-demo docking_demo".to_string(),
        main_window_size: winit::dpi::LogicalSize::new(980.0, 720.0),
        ..Default::default()
    };

    let driver = DockingDemoDriver::default();
    fret_kit::run_native_demo(config, app, driver).context("run docking_demo app")
}
