use anyhow::Context as _;
use fret::app::{AppRenderContext, text};
use fret_app::{App, CommandId, Effect, WindowRequest};
use fret_bootstrap::ui_diagnostics::UiDiagnosticsService;
use fret_core::{
    AppWindowId, Axis, Color, Corners, DockLayout, DockLayoutNode, DockLayoutWindow, DrawOrder,
    Edges, Event, Rect, Scene, SceneOp, UiServices, geometry::Px,
};
use fret_docking::{
    DockHostOptions, DockPanel, DockPanelElementRegistry, DockSurface, DockViewportLayout,
    DockViewportOverlayHooks, ViewportPanel, advanced::DockManager,
};
use fret_launch::{
    DevStateExport, DevStateHook, DevStateHooks, FnDriver, WindowCreateSpec, WinitCommandContext,
    WinitEventContext, WinitHotReloadContext, WinitRenderContext, WinitRunnerConfig,
    WinitWindowContext,
};
use fret_runtime::PlatformCapabilities;
use fret_ui::element::{
    AnyElement, ContainerProps, InsetEdge, LayoutStyle, Length, PositionStyle, SemanticsProps,
};
use fret_ui::{ElementContext, Theme, UiTree};
use fret_ui_kit::ui;
use fret_ui_kit::{LayoutRefinement, Space};
use fret_ui_shadcn::facade as shadcn;
use std::sync::Arc;

const DOCKING_DEMO_TAB_BAR_H: Px = Px(28.0);
const DOCKING_DEMO_DRAG_ANCHOR_SIZE: Px = Px(12.0);

const CMD_DOCK_DEMO_SPLIT_TOGGLE: &str = "dock_demo.split.toggle";
const DEV_STATE_DOCKING_LAYOUT_KEY: &str = "docking.layout";

fn docking_demo_list_row_text<'a, Cx>(cx: &mut Cx, text: impl Into<Arc<str>>) -> AnyElement
where
    Cx: AppRenderContext<'a>,
{
    text::list_row_label(cx, text)
}

fn docking_demo_readout_text<'a, Cx>(cx: &mut Cx, text: impl Into<Arc<str>>) -> AnyElement
where
    Cx: AppRenderContext<'a>,
{
    text::control_readout(cx, text)
}

fn docking_demo_absolute_layout(bounds: Rect, rect: Rect) -> LayoutStyle {
    let mut layout = LayoutStyle::default();
    layout.position = PositionStyle::Absolute;
    layout.inset.left = InsetEdge::Px(Px(rect.origin.x.0 - bounds.origin.x.0));
    layout.inset.top = InsetEdge::Px(Px(rect.origin.y.0 - bounds.origin.y.0));
    layout.size.width = Length::Px(rect.size.width);
    layout.size.height = Length::Px(rect.size.height);
    layout
}

fn docking_demo_diagnostic_anchor<H: fret_ui::UiHost>(
    cx: &mut fret_ui::ElementContext<'_, H>,
    bounds: Rect,
    rect: Rect,
    test_id: &'static str,
) -> AnyElement {
    cx.keyed(test_id, |cx| {
        cx.semantics(
            SemanticsProps {
                layout: docking_demo_absolute_layout(bounds, rect),
                role: fret_core::SemanticsRole::Group,
                test_id: Some(Arc::from(test_id)),
                ..Default::default()
            },
            |_cx| Vec::<AnyElement>::new(),
        )
    })
}

fn docking_demo_tab_anchor_rects(bounds: Rect) -> (Rect, Rect) {
    // Keep the scripted drag anchors inside the *tab* rect even when tabs use natural widths.
    let mid_x = bounds.origin.x.0 + bounds.size.width.0 * 0.5;
    let pad_x = 48.0_f32.min((bounds.size.width.0 * 0.25).max(0.0));
    let x_l = bounds.origin.x.0 + pad_x;
    let x_r = mid_x + pad_x;
    let y = bounds.origin.y.0 + (DOCKING_DEMO_TAB_BAR_H.0 * 0.5);

    let half = DOCKING_DEMO_DRAG_ANCHOR_SIZE.0 * 0.5;
    let rect = |x: f32| {
        Rect::new(
            fret_core::Point::new(Px((x - half).max(bounds.origin.x.0)), Px(y - half)),
            fret_core::Size::new(DOCKING_DEMO_DRAG_ANCHOR_SIZE, DOCKING_DEMO_DRAG_ANCHOR_SIZE),
        )
    };

    (rect(x_l), rect(x_r))
}

#[derive(Debug, Default)]
struct DockingDemoDevStateIncoming {
    layout: Option<fret_core::DockLayout>,
}

#[derive(Debug, Default)]
struct DockingDemoDevStateModels {
    main_window: Option<AppWindowId>,
}

struct DemoDockPanelRegistry;

impl DemoDockPanelRegistry {
    fn render_known_panel(
        &self,
        cx: &mut ElementContext<'_, App>,
        panel: &fret_core::PanelKey,
    ) -> Option<AnyElement> {
        if !matches!(panel.kind.0.as_str(), "core.hierarchy" | "core.inspector") {
            return None;
        }

        let kind = panel.kind.0.clone();
        let theme = Theme::global(&*cx.app).clone();
        let padding = theme.metric_token("metric.padding.md");
        let background = theme.color_token("background");

        let label: &str = match kind.as_str() {
            "core.hierarchy" => "Hierarchy panel (declarative root)",
            "core.inspector" => "Inspector panel (declarative root)",
            _ => "Panel (unregistered kind)",
        };

        Some(cx.container(
            ContainerProps {
                layout: {
                    let mut layout = LayoutStyle::default();
                    layout.size.width = Length::Fill;
                    layout.size.height = Length::Fill;
                    layout
                },
                padding: fret_core::Edges::all(padding).into(),
                background: Some(background),
                ..Default::default()
            },
            |cx| {
                let body = match kind.as_str() {
                    "core.hierarchy" => shadcn::Card::new(vec![
                        shadcn::CardHeader::new(vec![
                            shadcn::CardTitle::new("Hierarchy").into_element(cx),
                            shadcn::CardDescription::new(
                                "Placeholder content for docking + tab drag smoke tests.",
                            )
                            .into_element(cx),
                        ])
                        .into_element(cx),
                        shadcn::CardContent::new([ui::v_flex(|cx| {
                            [
                                shadcn::Button::new("Toggle collapse (layout.expand motion)")
                                    .variant(shadcn::ButtonVariant::Outline)
                                    .size(shadcn::ButtonSize::Sm)
                                    .on_click(CMD_DOCK_DEMO_SPLIT_TOGGLE)
                                    .test_id("dock-demo-split-toggle")
                                    .into_element(cx),
                                docking_demo_list_row_text(cx, "Scene"),
                                docking_demo_list_row_text(cx, "Camera"),
                                docking_demo_list_row_text(cx, "Directional Light"),
                                docking_demo_list_row_text(cx, "Player"),
                            ]
                        })
                        .gap(Space::N1)
                        .layout(LayoutRefinement::default().w_full())
                        .into_element(cx)])
                        .into_element(cx),
                    ])
                    .size(shadcn::CardSize::Sm)
                    .into_element(cx),
                    "core.inspector" => shadcn::Card::new(vec![
                        shadcn::CardHeader::new(vec![
                            shadcn::CardTitle::new("Inspector").into_element(cx),
                            shadcn::CardDescription::new(label).into_element(cx),
                        ])
                        .into_element(cx),
                        shadcn::CardContent::new([ui::v_flex(|cx| {
                            [
                                docking_demo_readout_text(cx, "Name: Player"),
                                docking_demo_readout_text(cx, "Position: (12.0, 3.0, -8.0)"),
                                docking_demo_readout_text(cx, "Rotation: (0.0, 90.0, 0.0)"),
                            ]
                        })
                        .gap(Space::N1)
                        .layout(LayoutRefinement::default().w_full())
                        .into_element(cx)])
                        .into_element(cx),
                    ])
                    .size(shadcn::CardSize::Sm)
                    .into_element(cx),
                    _ => shadcn::Card::new(vec![
                        shadcn::CardHeader::new(vec![
                            shadcn::CardTitle::new(label).into_element(cx),
                        ])
                        .into_element(cx),
                    ])
                    .size(shadcn::CardSize::Sm)
                    .into_element(cx),
                };

                vec![body]
            },
        ))
    }
}

impl DockPanelElementRegistry<App> for DemoDockPanelRegistry {
    fn render_panel(
        &self,
        cx: &mut ElementContext<'_, App>,
        _window: AppWindowId,
        panel: &fret_core::PanelKey,
    ) -> Option<AnyElement> {
        self.render_known_panel(cx, panel)
    }
}

struct DemoViewportOverlayHooks;

impl DockViewportOverlayHooks for DemoViewportOverlayHooks {
    fn paint_with_layout(
        &self,
        theme: fret_ui::ThemeSnapshot,
        _window: AppWindowId,
        _panel: &fret_core::PanelKey,
        _viewport: ViewportPanel,
        layout: DockViewportLayout,
        scene: &mut Scene,
    ) {
        let border_color = Color {
            a: 0.65,
            ..theme.color_token("primary")
        };
        let draw_rect = layout.draw_rect;
        scene.push(SceneOp::Quad {
            order: DrawOrder(6),
            rect: draw_rect,
            background: fret_core::Paint::TRANSPARENT.into(),

            border: Edges::all(Px(2.0)),
            border_paint: fret_core::Paint::Solid(border_color).into(),
            corner_radii: Corners::all(Px(0.0)),
        });
    }
}

pub struct DockingDemoWindowState {
    ui: UiTree<App>,
    dock_space: Option<fret_core::NodeId>,
}

#[derive(Default)]
pub struct DockingDemoDriver {
    dock_surface: Option<DockSurface>,
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
            dock_space: None,
        }
    }

    fn ensure_dock_graph(surface: DockSurface, app: &mut App, window: AppWindowId) {
        use fret_core::PanelKey;

        let incoming_layout = app
            .with_global_mut_untracked(DockingDemoDevStateIncoming::default, |st, _app| {
                st.layout.take()
            });

        surface.ensure_panel(app, &PanelKey::new("core.hierarchy"), || DockPanel {
            title: "Hierarchy".to_string(),
            color: fret_core::Color::TRANSPARENT,
            viewport: None,
        });
        surface.ensure_panel(app, &PanelKey::new("core.inspector"), || DockPanel {
            title: "Inspector".to_string(),
            color: fret_core::Color::TRANSPARENT,
            viewport: None,
        });

        if surface.has_window_root(app, window) {
            return;
        }

        if let Some(layout) = incoming_layout.as_ref() {
            let windows = [(window, "main".to_string())];
            if matches!(
                surface.try_import_layout_for_windows(app, layout, &windows),
                Ok(true)
            ) {
                return;
            }
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
                    tabs: vec![PanelKey::new("core.hierarchy")],
                    active: 0,
                },
                DockLayoutNode::Tabs {
                    id: 2,
                    tabs: vec![PanelKey::new("core.inspector")],
                    active: 0,
                },
                DockLayoutNode::Split {
                    id: 3,
                    axis: Axis::Horizontal,
                    children: vec![1, 2],
                    fractions: vec![0.5, 0.5],
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
        state: &mut DockingDemoWindowState,
        bounds: Rect,
    ) {
        Self::ensure_dock_graph(surface, app, window);

        let dock_space = fret_ui::declarative::render_root(
            &mut state.ui,
            app,
            services,
            window,
            bounds,
            "dock-demo-dock-space",
            move |cx| {
                let (left_anchor, right_anchor) = docking_demo_tab_anchor_rects(bounds);
                let mut children = Vec::with_capacity(3);
                children.push(surface.host(
                    cx,
                    window,
                    DockHostOptions {
                        test_id: Some("dock-demo-dock-space"),
                        ..Default::default()
                    },
                ));
                children.push(docking_demo_diagnostic_anchor(
                    cx,
                    bounds,
                    left_anchor,
                    "dock-demo-tab-drag-anchor-left",
                ));
                children.push(docking_demo_diagnostic_anchor(
                    cx,
                    bounds,
                    right_anchor,
                    "dock-demo-tab-drag-anchor-right",
                ));
                children
            },
        );
        state.dock_space = Some(dock_space);

        // When view caching is active, explicitly mark the dock space as a cache root so paint
        // caching + prepaint hooks are exercised in the same mode as UI Gallery shell caching.
        if state.ui.view_cache_enabled() {
            state
                .ui
                .set_node_view_cache_flags(dock_space, true, false, false);
        }
    }
}

fn init(driver: &mut DockingDemoDriver, app: &mut App, main_window: AppWindowId) {
    let surface = DockSurface::new(main_window);
    surface.install_panel_registry(app, Arc::new(DemoDockPanelRegistry));
    surface.install_viewport_overlay_hooks(app, Arc::new(DemoViewportOverlayHooks));
    driver.dock_surface = Some(surface);
    driver.main_window = Some(main_window);
    app.with_global_mut_untracked(DockingDemoDevStateModels::default, |st, _app| {
        st.main_window = Some(main_window);
    });
}

fn create_window_state(
    _driver: &mut DockingDemoDriver,
    app: &mut App,
    window: AppWindowId,
) -> DockingDemoWindowState {
    DockingDemoDriver::build_ui(app, window)
}

fn hot_reload_window(
    _driver: &mut DockingDemoDriver,
    context: WinitHotReloadContext<'_, DockingDemoWindowState>,
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
    _driver: &mut DockingDemoDriver,
    context: WinitWindowContext<'_, DockingDemoWindowState>,
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
    _driver: &mut DockingDemoDriver,
    context: WinitWindowContext<'_, DockingDemoWindowState>,
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
    driver: &mut DockingDemoDriver,
    context: WinitCommandContext<'_, DockingDemoWindowState>,
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
    if command.as_str() == CMD_DOCK_DEMO_SPLIT_TOGGLE {
        let Some(surface) = driver.dock_surface else {
            return;
        };

        let Some((split, first_fraction)) = app.global::<DockManager>().and_then(|dock| {
            let split = dock.workspace.graph.window_root(window)?;
            match dock.workspace.graph.node(split)? {
                fret_core::DockNode::Split { fractions, .. } if fractions.len() == 2 => {
                    Some((split, *fractions.first().unwrap_or(&0.5)))
                }
                _ => None,
            }
        }) else {
            return;
        };

        let target = if first_fraction < 0.2 { 0.5 } else { 0.12 };
        let changed = surface.host_lifecycle().on_dock_op(
            app,
            fret_core::DockOp::SetSplitFractions {
                split,
                fractions: vec![target, (1.0 - target).max(0.0)],
            },
        );
        let _ = changed;
        return;
    }
    if command.as_str() == "dock_demo.close" {
        app.push_effect(Effect::Window(WindowRequest::Close(window)));
    }
}

fn handle_event(
    _driver: &mut DockingDemoDriver,
    context: WinitEventContext<'_, DockingDemoWindowState>,
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

fn dock_op(driver: &mut DockingDemoDriver, app: &mut App, op: fret_core::DockOp) {
    if let Some(surface) = driver.dock_surface {
        let _ = surface.host_lifecycle().on_dock_op(app, op);
    }
}

fn render(driver: &mut DockingDemoDriver, context: WinitRenderContext<'_, DockingDemoWindowState>) {
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
        DockingDemoDriver::render_dock(surface, app, services, window, state, bounds);
    }

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
    _driver: &mut DockingDemoDriver,
    _app: &mut App,
    _request: &fret_app::CreateWindowRequest,
) -> Option<WindowCreateSpec> {
    None
}

fn window_created(
    driver: &mut DockingDemoDriver,
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

fn before_close_window(driver: &mut DockingDemoDriver, app: &mut App, window: AppWindowId) -> bool {
    if let Some(surface) = driver.dock_surface {
        let _ = surface.host_lifecycle().before_close_window(app, window);
    }
    true
}

fn semantics_snapshot(
    _driver: &mut DockingDemoDriver,
    _app: &mut App,
    _window: AppWindowId,
    state: &mut DockingDemoWindowState,
) -> Option<Arc<fret_core::SemanticsSnapshot>> {
    state.ui.semantics_snapshot_arc()
}

fn accessibility_focus(
    _driver: &mut DockingDemoDriver,
    app: &mut App,
    _window: AppWindowId,
    state: &mut DockingDemoWindowState,
    target: fret_core::NodeId,
) {
    fret_ui_app::accessibility_actions::focus(&mut state.ui, app, target);
}

fn accessibility_invoke(
    _driver: &mut DockingDemoDriver,
    app: &mut App,
    services: &mut dyn UiServices,
    _window: AppWindowId,
    state: &mut DockingDemoWindowState,
    target: fret_core::NodeId,
) {
    fret_ui_app::accessibility_actions::invoke(&mut state.ui, app, services, target);
}

fn accessibility_set_value_text(
    _driver: &mut DockingDemoDriver,
    app: &mut App,
    services: &mut dyn UiServices,
    _window: AppWindowId,
    state: &mut DockingDemoWindowState,
    target: fret_core::NodeId,
    value: &str,
) {
    fret_ui_app::accessibility_actions::set_value_text(&mut state.ui, app, services, target, value);
}

fn accessibility_set_value_numeric(
    _driver: &mut DockingDemoDriver,
    app: &mut App,
    services: &mut dyn UiServices,
    _window: AppWindowId,
    state: &mut DockingDemoWindowState,
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
    _driver: &mut DockingDemoDriver,
    app: &mut App,
    services: &mut dyn UiServices,
    _window: AppWindowId,
    state: &mut DockingDemoWindowState,
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
    _driver: &mut DockingDemoDriver,
    app: &mut App,
    services: &mut dyn UiServices,
    _window: AppWindowId,
    state: &mut DockingDemoWindowState,
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
    hooks: &mut fret_launch::FnDriverHooks<DockingDemoDriver, DockingDemoWindowState>,
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

pub fn build_fn_driver() -> FnDriver<DockingDemoDriver, DockingDemoWindowState> {
    FnDriver::new(
        DockingDemoDriver::default(),
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
    app.with_global_mut_untracked(DevStateHooks::default, |hooks, _app| {
        hooks.register(
            DevStateHook::new(DEV_STATE_DOCKING_LAYOUT_KEY, |app| {
                let Some(models) = app.global::<DockingDemoDevStateModels>() else {
                    return DevStateExport::Noop;
                };
                let Some(window) = models.main_window else {
                    return DevStateExport::Noop;
                };
                let surface = DockSurface::new(window);
                if !surface.has_window_root(app, window) {
                    return DevStateExport::Noop;
                }

                let Some(layout) = surface.export_layout(app, &[(window, "main".to_string())])
                else {
                    return DevStateExport::Noop;
                };
                match serde_json::to_value(layout) {
                    Ok(value) => DevStateExport::Set(value),
                    Err(_) => DevStateExport::Noop,
                }
            })
            .with_import(|app, value| {
                let layout = serde_json::from_value(value).map_err(|e| e.to_string())?;
                app.with_global_mut_untracked(DockingDemoDevStateIncoming::default, |st, _app| {
                    st.layout = Some(layout);
                });
                Ok(())
            }),
        );
    });

    let config = WinitRunnerConfig {
        main_window_title: "fret-demo docking_demo".to_string(),
        main_window_size: fret_launch::WindowLogicalSize::new(980.0, 720.0),
        ..Default::default()
    };

    fret::advanced::run_native_with_configured_fn_driver(config, app, build_fn_driver())
        .context("run docking_demo app")
}
