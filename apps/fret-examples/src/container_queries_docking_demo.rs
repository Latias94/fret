use anyhow::Context as _;
use fret::app::{AppRenderContext, text};
use fret_app::{App, CommandId, Effect, WindowRequest};
use fret_bootstrap::ui_diagnostics::UiDiagnosticsService;
use fret_core::{
    AppWindowId, Axis, DockLayout, DockLayoutNode, DockLayoutWindow, Event, Rect, UiServices,
    geometry::Px,
};
use fret_docking::{DockHostOptions, DockPanel, DockPanelElementRegistry, DockSurface};
use fret_launch::{
    FnDriver, WindowCreateSpec, WinitCommandContext, WinitEventContext, WinitHotReloadContext,
    WinitRenderContext, WinitRunnerConfig, WinitWindowContext,
};
use fret_runtime::PlatformCapabilities;
use fret_ui::element::{
    AnyElement, ContainerProps, InsetEdge, LayoutQueryRegionProps, LayoutStyle, Length,
    PositionStyle, SemanticsDecoration, SemanticsProps,
};
use fret_ui::{ElementContext, Invalidation, Theme, UiTree};
use fret_ui_shadcn::facade as shadcn;
use std::sync::Arc;

const INITIAL_SPLIT_FRACTION_LEFT: f32 = 0.75;
const SPLIT_ANCHOR_W: Px = Px(18.0);

fn container_query_docking_readout_text<'a, Cx>(
    cx: &mut Cx,
    text: impl Into<Arc<str>>,
) -> AnyElement
where
    Cx: AppRenderContext<'a>,
{
    text::control_readout(cx, text)
}

fn container_query_docking_placeholder_text<'a, Cx>(
    cx: &mut Cx,
    text: impl Into<Arc<str>>,
) -> AnyElement
where
    Cx: AppRenderContext<'a>,
{
    text::button_label(cx, text)
}

fn container_query_docking_absolute_layout(bounds: Rect, rect: Rect) -> LayoutStyle {
    let mut layout = LayoutStyle::default();
    layout.position = PositionStyle::Absolute;
    layout.inset.left = InsetEdge::Px(Px(rect.origin.x.0 - bounds.origin.x.0));
    layout.inset.top = InsetEdge::Px(Px(rect.origin.y.0 - bounds.origin.y.0));
    layout.size.width = Length::Px(rect.size.width);
    layout.size.height = Length::Px(rect.size.height);
    layout
}

fn container_query_docking_split_anchor_rect(bounds: Rect) -> Rect {
    // Position an input-transparent anchor over the initial split handle. Scripted drags can
    // target this anchor deterministically without needing docking internals to expose test ids
    // for split handles.
    let x = bounds.origin.x.0 + bounds.size.width.0 * INITIAL_SPLIT_FRACTION_LEFT;
    let x0 = x - (SPLIT_ANCHOR_W.0 * 0.5);
    Rect::new(
        fret_core::Point::new(Px(x0), bounds.origin.y),
        fret_core::Size::new(SPLIT_ANCHOR_W, bounds.size.height),
    )
}

fn container_query_docking_diagnostic_anchor<H: fret_ui::UiHost>(
    cx: &mut ElementContext<'_, H>,
    bounds: Rect,
    rect: Rect,
    test_id: &'static str,
) -> AnyElement {
    cx.keyed(test_id, |cx| {
        cx.semantics(
            SemanticsProps {
                layout: container_query_docking_absolute_layout(bounds, rect),
                role: fret_core::SemanticsRole::Group,
                test_id: Some(Arc::from(test_id)),
                ..Default::default()
            },
            |_cx| Vec::<AnyElement>::new(),
        )
    })
}

struct DemoDockPanelRegistry;

impl DemoDockPanelRegistry {
    fn render_left_panel(cx: &mut ElementContext<'_, App>, theme: &Theme) -> Vec<AnyElement> {
        let padding = theme.metric_token("metric.padding.md");
        let background = theme.color_token("background");
        let muted = theme.color_token("muted");

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
                padding: fret_core::Edges::all(padding).into(),
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
                                    padding: fret_core::Edges::all(Px(6.0)).into(),
                                    background: Some(muted),
                                    corner_radii: fret_core::Corners::all(Px(6.0)),
                                    ..Default::default()
                                },
                                move |cx| {
                                    vec![container_query_docking_readout_text(
                                        cx,
                                        Arc::clone(&mode_text),
                                    )]
                                },
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
                                padding: fret_core::Edges::all(Px(8.0)).into(),
                                background: Some(theme.color_token("secondary")),
                                corner_radii: fret_core::Corners::all(Px(6.0)),
                                ..Default::default()
                            },
                            |cx| vec![container_query_docking_placeholder_text(cx, "Input stub")],
                        );

                        let field = shadcn::Field::new([
                            shadcn::FieldLabel::new("Name").into_element(cx),
                            shadcn::FieldContent::new([field_input_stub]).into_element(cx),
                        ])
                        .orientation(shadcn::FieldOrientation::ContainerAdaptive)
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

impl DockPanelElementRegistry<App> for DemoDockPanelRegistry {
    fn render_panel(
        &self,
        cx: &mut ElementContext<'_, App>,
        _window: AppWindowId,
        panel: &fret_core::PanelKey,
    ) -> Option<AnyElement> {
        let kind = panel.kind.0.clone();
        let theme = Theme::global(&*cx.app).clone();
        match kind.as_str() {
            "examples.cq.left" => Self::render_left_panel(cx, &theme).into_iter().next(),
            "examples.cq.right" => Some(cx.container(
                ContainerProps {
                    layout: {
                        let mut layout = LayoutStyle::default();
                        layout.size.width = Length::Fill;
                        layout.size.height = Length::Fill;
                        layout
                    },
                    padding: fret_core::Edges::all(theme.metric_token("metric.padding.md")).into(),
                    background: Some(theme.color_token("background")),
                    ..Default::default()
                },
                |_cx| Vec::<AnyElement>::new(),
            )),
            _ => Some(container_query_docking_readout_text(
                cx,
                "Unregistered panel kind",
            )),
        }
    }
}

pub struct ContainerQueriesDockingDemoWindowState {
    ui: UiTree<App>,
    dock_space: Option<fret_core::NodeId>,
}

#[derive(Default)]
pub struct ContainerQueriesDockingDemoDriver {
    dock_surface: Option<DockSurface>,
}

impl ContainerQueriesDockingDemoDriver {
    fn build_ui(_app: &mut App, window: AppWindowId) -> ContainerQueriesDockingDemoWindowState {
        let mut ui: UiTree<App> = UiTree::new();
        ui.set_window(window);
        ui.set_view_cache_enabled(std::env::var_os("FRET_EXAMPLES_VIEW_CACHE").is_some());
        ui.set_debug_enabled(std::env::var_os("FRET_DIAG").is_some_and(|v| !v.is_empty()));
        ContainerQueriesDockingDemoWindowState {
            ui,
            dock_space: None,
        }
    }

    fn ensure_dock_graph(surface: DockSurface, app: &mut App, window: AppWindowId) {
        use fret_core::PanelKey;

        surface.ensure_panel(app, &PanelKey::new("examples.cq.left"), || DockPanel {
            title: "Container queries".to_string(),
            color: fret_core::Color::TRANSPARENT,
            viewport: None,
        });
        surface.ensure_panel(app, &PanelKey::new("examples.cq.right"), || DockPanel {
            title: "Spacer".to_string(),
            color: fret_core::Color::TRANSPARENT,
            viewport: None,
        });

        if surface.has_window_root(app, window) {
            return;
        }

        let layout = DockLayout::new(
            vec![DockLayoutWindow {
                logical_window_id: "main".to_string(),
                root: 3,
                placement: None,
                floatings: Vec::new(),
            }],
            vec![
                DockLayoutNode::Tabs {
                    id: 1,
                    tabs: vec![PanelKey::new("examples.cq.left")],
                    active: 0,
                },
                DockLayoutNode::Tabs {
                    id: 2,
                    tabs: vec![PanelKey::new("examples.cq.right")],
                    active: 0,
                },
                DockLayoutNode::Split {
                    id: 3,
                    axis: Axis::Horizontal,
                    children: vec![1, 2],
                    fractions: vec![
                        INITIAL_SPLIT_FRACTION_LEFT,
                        1.0 - INITIAL_SPLIT_FRACTION_LEFT,
                    ],
                },
            ],
        );
        let windows = [(window, "main".to_string())];
        let _ = surface.try_import_layout_for_windows(app, &layout, &windows);
    }

    fn render_dock(
        surface: DockSurface,
        app: &mut App,
        services: &mut dyn UiServices,
        window: AppWindowId,
        state: &mut ContainerQueriesDockingDemoWindowState,
        bounds: Rect,
    ) {
        Self::ensure_dock_graph(surface, app, window);

        let dock_space = fret_ui::declarative::render_root(
            &mut state.ui,
            app,
            services,
            window,
            bounds,
            "cq-dock-demo-dock-space",
            move |cx| {
                let split_anchor = container_query_docking_split_anchor_rect(bounds);
                let mut children = Vec::with_capacity(2);
                children.push(surface.host(
                    cx,
                    window,
                    DockHostOptions {
                        test_id: Some("cq-dock-demo-dock-space"),
                        ..Default::default()
                    },
                ));
                children.push(container_query_docking_diagnostic_anchor(
                    cx,
                    bounds,
                    split_anchor,
                    "cq-dock-demo-split-anchor",
                ));
                children
            },
        );
        state.dock_space = Some(dock_space);

        if state.ui.view_cache_enabled() {
            state
                .ui
                .set_node_view_cache_flags(dock_space, true, false, false);
        }
    }
}

fn init(driver: &mut ContainerQueriesDockingDemoDriver, app: &mut App, main_window: AppWindowId) {
    let surface = DockSurface::new(main_window);
    surface.install_panel_registry(app, Arc::new(DemoDockPanelRegistry));
    driver.dock_surface = Some(surface);
}

fn create_window_state(
    _driver: &mut ContainerQueriesDockingDemoDriver,
    app: &mut App,
    window: AppWindowId,
) -> ContainerQueriesDockingDemoWindowState {
    ContainerQueriesDockingDemoDriver::build_ui(app, window)
}

fn hot_reload_window(
    _driver: &mut ContainerQueriesDockingDemoDriver,
    context: WinitHotReloadContext<'_, ContainerQueriesDockingDemoWindowState>,
) {
    let WinitHotReloadContext {
        app,
        services: _,
        window,
        state,
    } = context;
    crate::hotpatch::reset_ui_tree(app, window, &mut state.ui);
    state.dock_space = None;
}

fn handle_model_changes(
    _driver: &mut ContainerQueriesDockingDemoDriver,
    context: WinitWindowContext<'_, ContainerQueriesDockingDemoWindowState>,
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
    _driver: &mut ContainerQueriesDockingDemoDriver,
    context: WinitWindowContext<'_, ContainerQueriesDockingDemoWindowState>,
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
    _driver: &mut ContainerQueriesDockingDemoDriver,
    context: WinitCommandContext<'_, ContainerQueriesDockingDemoWindowState>,
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

fn handle_event(
    _driver: &mut ContainerQueriesDockingDemoDriver,
    context: WinitEventContext<'_, ContainerQueriesDockingDemoWindowState>,
    event: &Event,
) {
    let WinitEventContext {
        app,
        services,
        window,
        state,
    } = context;

    if fret_bootstrap::maybe_consume_event(app, window, event) {
        return;
    }

    if matches!(event, Event::WindowCloseRequested) {
        app.push_effect(Effect::Window(WindowRequest::Close(window)));
        return;
    }
    state.ui.dispatch_event(app, services, event);
}

fn dock_op(driver: &mut ContainerQueriesDockingDemoDriver, app: &mut App, op: fret_core::DockOp) {
    if let Some(surface) = driver.dock_surface {
        let _ = surface.host_lifecycle().on_dock_op(app, op);
    }
}

fn render(
    driver: &mut ContainerQueriesDockingDemoDriver,
    context: WinitRenderContext<'_, ContainerQueriesDockingDemoWindowState>,
) {
    let WinitRenderContext {
        app,
        services,
        window,
        state,
        bounds,
        scale_factor,
        scene,
    } = context;

    if let Some(surface) = driver.dock_surface {
        ContainerQueriesDockingDemoDriver::render_dock(
            surface, app, services, window, state, bounds,
        );
    }

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

    let semantics_snapshot = state.ui.semantics_snapshot_arc();
    let drive = app.with_global_mut_untracked(UiDiagnosticsService::default, |svc, app| {
        svc.drive_script_for_window(
            app,
            services,
            window,
            bounds,
            scale_factor,
            Some(&mut state.ui),
            semantics_snapshot.as_deref(),
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
                            handle_command(
                                driver,
                                WinitCommandContext {
                                    app,
                                    services,
                                    window,
                                    state,
                                },
                                command,
                            );
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
            &mut state.ui,
            element_runtime,
            None,
            scene,
        );
        let _ = svc.maybe_dump_if_triggered();
        if svc.is_enabled() {
            app.push_effect(Effect::RequestAnimationFrame(window));
        }
    });
}

fn window_create_spec(
    _driver: &mut ContainerQueriesDockingDemoDriver,
    _app: &mut App,
    _request: &fret_app::CreateWindowRequest,
) -> Option<WindowCreateSpec> {
    None
}

fn window_created(
    driver: &mut ContainerQueriesDockingDemoDriver,
    app: &mut App,
    request: &fret_app::CreateWindowRequest,
    new_window: AppWindowId,
) {
    if let Some(surface) = driver.dock_surface {
        let _ = surface
            .host_lifecycle()
            .on_window_created(app, request, new_window);
    }
}

fn before_close_window(
    driver: &mut ContainerQueriesDockingDemoDriver,
    app: &mut App,
    window: AppWindowId,
) -> bool {
    if let Some(surface) = driver.dock_surface {
        let _ = surface.host_lifecycle().before_close_window(app, window);
    }
    true
}

fn semantics_snapshot(
    _driver: &mut ContainerQueriesDockingDemoDriver,
    _app: &mut App,
    _window: AppWindowId,
    state: &mut ContainerQueriesDockingDemoWindowState,
) -> Option<Arc<fret_core::SemanticsSnapshot>> {
    state.ui.semantics_snapshot_arc()
}

fn accessibility_focus(
    _driver: &mut ContainerQueriesDockingDemoDriver,
    app: &mut App,
    _window: AppWindowId,
    state: &mut ContainerQueriesDockingDemoWindowState,
    target: fret_core::NodeId,
) {
    fret_ui_app::accessibility_actions::focus(&mut state.ui, app, target);
}

fn accessibility_invoke(
    _driver: &mut ContainerQueriesDockingDemoDriver,
    app: &mut App,
    services: &mut dyn UiServices,
    _window: AppWindowId,
    state: &mut ContainerQueriesDockingDemoWindowState,
    target: fret_core::NodeId,
) {
    fret_ui_app::accessibility_actions::invoke(&mut state.ui, app, services, target);
}

fn accessibility_set_value_text(
    _driver: &mut ContainerQueriesDockingDemoDriver,
    app: &mut App,
    services: &mut dyn UiServices,
    _window: AppWindowId,
    state: &mut ContainerQueriesDockingDemoWindowState,
    target: fret_core::NodeId,
    value: &str,
) {
    fret_ui_app::accessibility_actions::set_value_text(&mut state.ui, app, services, target, value);
}

fn accessibility_set_value_numeric(
    _driver: &mut ContainerQueriesDockingDemoDriver,
    app: &mut App,
    services: &mut dyn UiServices,
    _window: AppWindowId,
    state: &mut ContainerQueriesDockingDemoWindowState,
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
    _driver: &mut ContainerQueriesDockingDemoDriver,
    app: &mut App,
    services: &mut dyn UiServices,
    _window: AppWindowId,
    state: &mut ContainerQueriesDockingDemoWindowState,
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
    _driver: &mut ContainerQueriesDockingDemoDriver,
    app: &mut App,
    services: &mut dyn UiServices,
    _window: AppWindowId,
    state: &mut ContainerQueriesDockingDemoWindowState,
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

fn configure_fn_driver_hooks(
    hooks: &mut fret_launch::FnDriverHooks<
        ContainerQueriesDockingDemoDriver,
        ContainerQueriesDockingDemoWindowState,
    >,
) {
    hooks.hot_reload_window = Some(hot_reload_window);
    hooks.dock_op = Some(dock_op);
    hooks.handle_model_changes = Some(handle_model_changes);
    hooks.handle_global_changes = Some(handle_global_changes);
    hooks.handle_command = Some(handle_command);
    hooks.window_create_spec = Some(window_create_spec);
    hooks.window_created = Some(window_created);
    hooks.before_close_window = Some(before_close_window);
    hooks.semantics_snapshot = Some(semantics_snapshot);
    hooks.accessibility_focus = Some(accessibility_focus);
    hooks.accessibility_invoke = Some(accessibility_invoke);
    hooks.accessibility_set_value_text = Some(accessibility_set_value_text);
    hooks.accessibility_set_value_numeric = Some(accessibility_set_value_numeric);
    hooks.accessibility_set_text_selection = Some(accessibility_set_text_selection);
    hooks.accessibility_replace_selected_text = Some(accessibility_replace_selected_text);
}

pub fn build_fn_driver()
-> FnDriver<ContainerQueriesDockingDemoDriver, ContainerQueriesDockingDemoWindowState> {
    FnDriver::new(
        ContainerQueriesDockingDemoDriver::default(),
        create_window_state,
        handle_event,
        render,
    )
    .with_init(init)
    .with_hooks(configure_fn_driver_hooks)
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

    let config = WinitRunnerConfig {
        main_window_title: "fret-demo container_queries_docking_demo".to_string(),
        // Ensure the left panel starts above md (>=768px) and can be dragged below it.
        main_window_size: fret_launch::WindowLogicalSize::new(1400.0, 760.0),
        ..Default::default()
    };

    fret::advanced::run_native_with_configured_fn_driver(config, app, build_fn_driver())
        .context("run container_queries_docking_demo app")
}
