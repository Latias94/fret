use anyhow::Context as _;
use fret_app::CreateWindowKind;
use fret_app::{App, CommandId, Effect, Model, WindowRequest};
use fret_bootstrap::ui_diagnostics::UiDiagnosticsService;
use fret_core::{
    AppWindowId, Color, Corners, DrawOrder, Edges, Event, Modifiers, MouseButton, MouseButtons,
    Point, Rect, RenderTargetId, Scene, SceneOp, Size, UiServices, ViewportInputEvent,
    geometry::Px,
};
use fret_docking::{
    DockManager, DockPanel, DockPanelRegistry, DockPanelRegistryService, DockViewportOverlayHooks,
    DockViewportOverlayHooksService, DockingRuntime, create_dock_space_node_with_test_id,
    render_and_bind_dock_panels, render_cached_panel_root,
};
use fret_launch::{
    WindowCreateSpec, WinitAppDriver, WinitCommandContext, WinitEventContext, WinitRenderContext,
    WinitRunnerConfig, WinitWindowContext,
};
use fret_runtime::PlatformCapabilities;
use fret_ui::declarative;
use fret_ui::element::{ContainerProps, LayoutStyle, Length};
use fret_ui::retained_bridge::{LayoutCx, PaintCx, SemanticsCx, UiTreeRetainedExt as _, Widget};
use fret_ui::{Invalidation, Theme, UiTree};
use fret_ui_kit::OverlayController;
use fret_ui_shadcn as shadcn;
use slotmap::KeyData;
use std::collections::{HashMap, HashSet};
use std::sync::{Arc, Mutex};

type ViewportKey = (AppWindowId, RenderTargetId);

const DOCKING_ARBITRATION_TAB_BAR_H: Px = Px(28.0);
const DOCKING_ARBITRATION_DRAG_ANCHOR_SIZE: Px = Px(12.0);

struct DockingArbitrationDragAnchor {
    test_id: &'static str,
}

impl DockingArbitrationDragAnchor {
    fn new(test_id: &'static str) -> Self {
        Self { test_id }
    }
}

impl<H: fret_ui::UiHost> Widget<H> for DockingArbitrationDragAnchor {
    fn hit_test(&self, _bounds: Rect, _position: Point) -> bool {
        false
    }

    fn semantics(&mut self, cx: &mut SemanticsCx<'_, H>) {
        cx.set_role(fret_core::SemanticsRole::Group);
        cx.set_test_id(self.test_id);
    }
}

struct DockingArbitrationHarnessRoot {
    dock_space: fret_core::NodeId,
    left_anchor: fret_core::NodeId,
    right_anchor: fret_core::NodeId,
}

impl<H: fret_ui::UiHost> Widget<H> for DockingArbitrationHarnessRoot {
    fn layout(&mut self, cx: &mut LayoutCx<'_, H>) -> Size {
        let bounds = cx.bounds;
        let _ = cx.layout_in(self.dock_space, bounds);

        let x_l = bounds.origin.x.0 + bounds.size.width.0 * 0.25;
        let x_r = bounds.origin.x.0 + bounds.size.width.0 * 0.75;
        let y = bounds.origin.y.0 + (DOCKING_ARBITRATION_TAB_BAR_H.0 * 0.5);

        let half = DOCKING_ARBITRATION_DRAG_ANCHOR_SIZE.0 * 0.5;
        let rect = |x: f32| {
            Rect::new(
                Point::new(Px((x - half).max(bounds.origin.x.0)), Px(y - half)),
                Size::new(
                    DOCKING_ARBITRATION_DRAG_ANCHOR_SIZE,
                    DOCKING_ARBITRATION_DRAG_ANCHOR_SIZE,
                ),
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

#[derive(Default)]
struct DemoViewportToolState {
    tools: HashMap<ViewportKey, fret_editor::ViewportToolManager>,
}
#[derive(Clone)]
struct DockingArbitrationPanelModels {
    popover_open: Model<bool>,
    dialog_open: Model<bool>,
    last_viewport_input: Model<Arc<str>>,
    synth_pointer_debug: Model<Arc<str>>,
}

#[derive(Default)]
struct DockingArbitrationPanelModelsService {
    by_window: HashMap<AppWindowId, DockingArbitrationPanelModels>,
}

impl DockingArbitrationPanelModelsService {
    fn set(&mut self, window: AppWindowId, models: DockingArbitrationPanelModels) {
        self.by_window.insert(window, models);
    }

    fn get(&self, window: AppWindowId) -> Option<&DockingArbitrationPanelModels> {
        self.by_window.get(&window)
    }
}

struct DockingArbitrationDockPanelRegistry;

impl DockPanelRegistry<App> for DockingArbitrationDockPanelRegistry {
    fn render_panel(
        &self,
        ui: &mut UiTree<App>,
        app: &mut App,
        services: &mut dyn UiServices,
        window: AppWindowId,
        bounds: Rect,
        panel: &fret_core::PanelKey,
    ) -> Option<fret_core::NodeId> {
        match panel.kind.0.as_str() {
            "demo.viewport.left" => {
                let root_name = "dock.panel.viewport_left";
                return Some(render_cached_panel_root(
                    ui,
                    app,
                    services,
                    window,
                    bounds,
                    root_name,
                    |cx| {
                        let mut layout = fret_ui::element::LayoutStyle::default();
                        layout.size.width = fret_ui::element::Length::Fill;
                        layout.size.height = fret_ui::element::Length::Fill;
                        vec![cx.semantics(
                            fret_ui::element::SemanticsProps {
                                layout,
                                role: fret_core::SemanticsRole::Viewport,
                                test_id: Some(Arc::<str>::from("dock-arb-viewport-left")),
                                ..Default::default()
                            },
                            |_cx| vec![],
                        )]
                    },
                ));
            }
            "demo.viewport.right" => {
                let root_name = "dock.panel.viewport_right";
                return Some(render_cached_panel_root(
                    ui,
                    app,
                    services,
                    window,
                    bounds,
                    root_name,
                    |cx| {
                        let mut layout = fret_ui::element::LayoutStyle::default();
                        layout.size.width = fret_ui::element::Length::Fill;
                        layout.size.height = fret_ui::element::Length::Fill;
                        vec![cx.semantics(
                            fret_ui::element::SemanticsProps {
                                layout,
                                role: fret_core::SemanticsRole::Viewport,
                                test_id: Some(Arc::<str>::from("dock-arb-viewport-right")),
                                ..Default::default()
                            },
                            |_cx| vec![],
                        )]
                    },
                ));
            }
            "demo.controls" => {}
            _ => return None,
        }

        let models = app
            .global::<DockingArbitrationPanelModelsService>()
            .and_then(|svc| svc.get(window))
            .cloned()?;

        let captured = format!("captured={:?}", ui.captured());
        let layer_lines: Vec<String> = ui
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

        let root_name = "dock.panel.controls";
        Some(
            declarative::RenderRootContext::new(ui, app, services, window, bounds).render_root(
                root_name,
                |cx| {
                cx.observe_model(&models.popover_open, Invalidation::Layout);
                cx.observe_model(&models.dialog_open, Invalidation::Layout);
                cx.observe_model(&models.last_viewport_input, Invalidation::Layout);
                cx.observe_model(&models.synth_pointer_debug, Invalidation::Layout);

                let theme = Theme::global(&*cx.app);
                let padding = theme.metric_required("metric.padding.md");
                let background = theme.color_required("background");

                let drag_state = cx
                    .app
                    .drag(fret_core::PointerId(0))
                    .map(|d| format!("drag(kind={:?}, dragging={})", d.kind, d.dragging))
                    .unwrap_or_else(|| "drag(<none>)".to_string());

                let last = cx
                    .app
                    .models()
                    .get_cloned(&models.last_viewport_input)
                    .unwrap_or_else(|| Arc::<str>::from("<missing>"));
                let synth_debug = cx
                    .app
                    .models()
                    .get_cloned(&models.synth_pointer_debug)
                    .unwrap_or_else(|| Arc::<str>::from("<missing>"));

                let popover_open = models.popover_open.clone();
                let dialog_open = models.dialog_open.clone();
                let sonner = shadcn::Sonner::global(&mut *cx.app);
                let popover_is_open = cx
                    .app
                    .models()
                    .get_cloned(&popover_open)
                    .unwrap_or(false);
                let dialog_is_open = cx
                    .app
                    .models()
                    .get_cloned(&dialog_open)
                    .unwrap_or(false);

                let popover = shadcn::Popover::new(popover_open.clone())
                    .auto_focus(true)
                    .into_element(
                        cx,
                        |cx| {
                            shadcn::Button::new("Open popover")
                                .variant(shadcn::ButtonVariant::Outline)
                                .test_id("dock-arb-popover-trigger")
                                .toggle_model(popover_open.clone())
                                .into_element(cx)
                        },
                        |cx| {
                            shadcn::PopoverContent::new(vec![
                                cx.text("Non-modal overlay (Popover)."),
                                shadcn::Button::new("Close")
                                    .variant(shadcn::ButtonVariant::Secondary)
                                    .test_id("dock-arb-popover-close")
                                    .toggle_model(popover_open.clone())
                                    .into_element(cx),
                            ])
                            .into_element(cx)
                        },
                    );

                let dialog = shadcn::Dialog::new(dialog_open.clone()).into_element(
                    cx,
                    |cx| {
                        shadcn::Button::new("Open modal dialog")
                            .variant(shadcn::ButtonVariant::Outline)
                            .test_id("dock-arb-dialog-trigger")
                            .toggle_model(dialog_open.clone())
                            .into_element(cx)
                    },
                    |cx| {
                        let sonner_for_dialog = sonner.clone();
                        let mut layout = fret_ui::element::LayoutStyle::default();
                        layout.size.width = fret_ui::element::Length::Fill;
                        layout.size.height = fret_ui::element::Length::Fill;
                        cx.semantics(
                            fret_ui::element::SemanticsProps {
                                layout,
                                role: fret_core::SemanticsRole::Dialog,
                                test_id: Some(Arc::<str>::from("dock-arb-dialog-content")),
                                ..Default::default()
                            },
                            |cx| {
                                vec![shadcn::DialogContent::new(vec![
                                    shadcn::DialogHeader::new(vec![
                                        shadcn::DialogTitle::new("Dialog").into_element(cx),
                                        shadcn::DialogDescription::new(
                                            "Modal barrier should block docking + viewport input.",
                                        )
                                        .into_element(cx),
                                    ])
                                    .into_element(cx),
                                    shadcn::Button::new("Trigger toast (Sonner)")
                                        .variant(shadcn::ButtonVariant::Secondary)
                                        .test_id("dock-arb-sonner-trigger")
                                        .on_activate(Arc::new(move |host, action_cx, _reason| {
                                            sonner_for_dialog.toast(
                                                host,
                                                action_cx.window,
                                                shadcn::ToastRequest::new(
                                                    "Toast while modal is open",
                                                )
                                                .duration(None)
                                                .test_id("dock-arb-sonner-toast"),
                                            );
                                        }))
                                        .into_element(cx),
                                    shadcn::DialogFooter::new(vec![
                                        shadcn::Button::new("Close")
                                            .variant(shadcn::ButtonVariant::Secondary)
                                            .test_id("dock-arb-dialog-close")
                                            .toggle_model(dialog_open.clone())
                                            .into_element(cx),
                                    ])
                                    .into_element(cx),
                                ])
                                .into_element(cx)]
                            },
                        )
                    },
                );

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
                |cx| {
                    let mut rows = Vec::new();
                    rows.push(cx.text("Docking arbitration demo (ADR 0072)"));
                    rows.push(cx.text(
                        "Open a popover, then drag a dock tab; start viewport drag inside the blue border; open a modal to block underlay.",
                    ));
                    rows.push(cx.text(drag_state));
                    rows.push(cx.text(captured.clone()));
                    rows.push(cx.text(format!("last_viewport_input={last}")));
                    rows.push(cx.text(
                        "Synth pointer: F1 toggle; I/J/K/L move; Space down/up; B right down/up; U/O wheel up/down (consumes these keys while enabled).",
                    ));
                    rows.push(cx.text(synth_debug.to_string()));
                    rows.push(cx.semantics(
                        fret_ui::element::SemanticsProps {
                            role: fret_core::SemanticsRole::Text,
                            test_id: Some(Arc::<str>::from(if popover_is_open {
                                "dock-arb-popover-open"
                            } else {
                                "dock-arb-popover-closed"
                            })),
                            label: Some(Arc::<str>::from(if popover_is_open {
                                "popover:open"
                            } else {
                                "popover:closed"
                            })),
                            ..Default::default()
                        },
                        |cx| vec![cx.text(if popover_is_open { "Popover: open" } else { "Popover: closed" })],
                    ));
                    rows.push(cx.semantics(
                        fret_ui::element::SemanticsProps {
                            role: fret_core::SemanticsRole::Text,
                            test_id: Some(Arc::<str>::from(if dialog_is_open {
                                "dock-arb-dialog-open"
                            } else {
                                "dock-arb-dialog-closed"
                            })),
                            label: Some(Arc::<str>::from(if dialog_is_open {
                                "dialog:open"
                            } else {
                                "dialog:closed"
                            })),
                            ..Default::default()
                        },
                        |cx| vec![cx.text(if dialog_is_open { "Dialog: open" } else { "Dialog: closed" })],
                    ));
                    rows.push(popover);
                    rows.push(dialog);
                    rows.push(
                        shadcn::Button::new("Underlay (modal barrier target)")
                            .variant(shadcn::ButtonVariant::Secondary)
                            .test_id("dock-arb-underlay-probe")
                            .into_element(cx),
                    );
                    rows.push(shadcn::Toaster::new().into_element(cx));
                    rows.push(cx.text("Layers (paint order):"));
                    for line in layer_lines.iter().cloned() {
                        rows.push(cx.text(line));
                    }
                    rows
                },
            )]
                },
            ),
        )
    }
}

#[derive(Default)]
struct ViewportDebugService {
    last_event: HashMap<AppWindowId, Model<Arc<str>>>,
}

struct DemoViewportOverlayHooks {
    tools: Arc<Mutex<DemoViewportToolState>>,
}

impl DockViewportOverlayHooks for DemoViewportOverlayHooks {
    fn paint_with_layout(
        &self,
        theme: fret_ui::ThemeSnapshot,
        window: AppWindowId,
        _panel: &fret_core::PanelKey,
        viewport: fret_docking::ViewportPanel,
        layout: fret_docking::DockViewportLayout,
        scene: &mut Scene,
    ) {
        let border_color = Color {
            a: 0.80,
            ..theme.color_required("primary")
        };
        let draw_rect = layout.draw_rect;
        scene.push(SceneOp::Quad {
            order: DrawOrder(6),
            rect: draw_rect,
            background: fret_core::Paint::TRANSPARENT,

            border: Edges::all(Px(2.0)),
            border_paint: fret_core::Paint::Solid(border_color),
            corner_radii: Corners::all(Px(0.0)),
        });

        let overlay = self.tools.lock().ok().and_then(|state| {
            state
                .tools
                .get(&(window, viewport.target))
                .map(|m| m.overlay())
        });
        if let Some(overlay) = overlay {
            fret_editor::paint_viewport_overlay(theme, layout.draw_rect, overlay, scene);
        }
    }
}

struct DockingArbitrationWindowState {
    ui: UiTree<App>,
    root: Option<fret_core::NodeId>,
    dock_space: Option<fret_core::NodeId>,
}

#[derive(Default)]
struct DockingArbitrationDriver {
    main_window: Option<AppWindowId>,
    docking_runtime: Option<DockingRuntime>,
    pending_layout: Option<fret_core::DockLayout>,
    restore: Option<DockLayoutRestoreState>,
    logical_windows: HashMap<AppWindowId, String>,
    next_logical_window_ix: u32,
    viewport_tools: Arc<Mutex<DemoViewportToolState>>,
    synth_pointers: HashMap<AppWindowId, SynthPointerState>,
    next_synth_touch_id: u64,
}

#[derive(Debug, Clone)]
struct SynthPointerState {
    enabled: bool,
    pointer_id: fret_core::PointerId,
    position: Point,
    pressed: bool,
    mouse_right_pressed: bool,
}

impl Default for SynthPointerState {
    fn default() -> Self {
        Self {
            enabled: false,
            // Use the same namespace as runner touch IDs (see `fret-runner-winit`):
            // `PointerId(0)` is reserved for mouse, so we pick a stable touch-like ID.
            pointer_id: fret_core::PointerId((1u64 << 56) | 42),
            position: Point::new(Px(160.0), Px(120.0)),
            pressed: false,
            mouse_right_pressed: false,
        }
    }
}

struct DockLayoutRestoreState {
    layout: fret_core::DockLayout,
    pending_logical_window_ids: HashSet<String>,
}

impl DockingArbitrationDriver {
    const DOCK_LAYOUT_PATH: &'static str = ".fret/layout.json";
    const MAIN_LOGICAL_WINDOW_ID: &'static str = "main";

    fn new(
        pending_layout: Option<fret_core::DockLayout>,
        viewport_tools: Arc<Mutex<DemoViewportToolState>>,
    ) -> Self {
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
            docking_runtime: None,
            pending_layout,
            restore: None,
            logical_windows: HashMap::new(),
            next_logical_window_ix,
            viewport_tools,
            synth_pointers: HashMap::new(),
            next_synth_touch_id: 42,
        }
    }

    fn synth_pointer_mut(&mut self, app: &App, window: AppWindowId) -> &mut SynthPointerState {
        let st = match self.synth_pointers.entry(window) {
            std::collections::hash_map::Entry::Occupied(entry) => entry.into_mut(),
            std::collections::hash_map::Entry::Vacant(entry) => {
                let mut st = SynthPointerState::default();
                st.pointer_id = fret_core::PointerId((1u64 << 56) | self.next_synth_touch_id);
                self.next_synth_touch_id = self.next_synth_touch_id.saturating_add(1);
                entry.insert(st)
            }
        };
        if !st.enabled {
            if let Some(metrics) = app.global::<fret_core::WindowMetricsService>()
                && let Some(size) = metrics.inner_size(window)
            {
                st.position = Point::new(Px(size.width.0 * 0.5), Px(size.height.0 * 0.5));
            }
        }
        st
    }

    fn update_synth_debug(app: &mut App, window: AppWindowId, synth: &SynthPointerState) {
        let Some(models) = app
            .global::<DockingArbitrationPanelModelsService>()
            .and_then(|svc| svc.get(window))
            .cloned()
        else {
            return;
        };

        let drag = app
            .drag(synth.pointer_id)
            .map(|d| format!("drag(kind={:?}, dragging={})", d.kind, d.dragging))
            .unwrap_or_else(|| "drag(<none>)".to_string());

        let msg: Arc<str> = Arc::from(
            format!(
                "synth_pointer: enabled={} id={:?} pos=({:.1},{:.1}) down={} mouse_right_down={} {}",
                synth.enabled,
                synth.pointer_id,
                synth.position.x.0,
                synth.position.y.0,
                synth.pressed,
                synth.mouse_right_pressed,
                drag
            )
            .into_boxed_str(),
        );
        let _ = app
            .models_mut()
            .update(&models.synth_pointer_debug, |v| *v = msg);
        app.request_redraw(window);
    }

    fn dispatch_synth_pointer_move(
        ui: &mut UiTree<App>,
        app: &mut App,
        services: &mut dyn UiServices,
        window: AppWindowId,
        synth: &SynthPointerState,
        modifiers: Modifiers,
    ) {
        if !synth.enabled {
            return;
        }
        let buttons = MouseButtons {
            left: synth.pressed,
            ..Default::default()
        };
        ui.dispatch_event(
            app,
            services,
            &Event::Pointer(fret_core::PointerEvent::Move {
                pointer_id: synth.pointer_id,
                position: synth.position,
                buttons,
                modifiers,
                pointer_type: fret_core::PointerType::Touch,
            }),
        );
        app.request_redraw(window);
    }

    fn dispatch_synth_pointer_button(
        ui: &mut UiTree<App>,
        app: &mut App,
        services: &mut dyn UiServices,
        window: AppWindowId,
        synth: &SynthPointerState,
        pressed: bool,
        modifiers: Modifiers,
    ) {
        if !synth.enabled {
            return;
        }
        let evt = if pressed {
            fret_core::PointerEvent::Down {
                pointer_id: synth.pointer_id,
                position: synth.position,
                button: MouseButton::Left,
                modifiers,
                click_count: 1,
                pointer_type: fret_core::PointerType::Touch,
            }
        } else {
            fret_core::PointerEvent::Up {
                pointer_id: synth.pointer_id,
                position: synth.position,
                button: MouseButton::Left,
                modifiers,
                is_click: true,
                click_count: 1,
                pointer_type: fret_core::PointerType::Touch,
            }
        };
        ui.dispatch_event(app, services, &Event::Pointer(evt));
        app.request_redraw(window);
    }

    fn dispatch_synth_mouse_button(
        ui: &mut UiTree<App>,
        app: &mut App,
        services: &mut dyn UiServices,
        window: AppWindowId,
        synth: &SynthPointerState,
        button: MouseButton,
        pressed: bool,
        modifiers: Modifiers,
    ) {
        if !synth.enabled {
            return;
        }
        let evt = if pressed {
            fret_core::PointerEvent::Down {
                pointer_id: fret_core::PointerId(0),
                position: synth.position,
                button,
                modifiers,
                click_count: 1,
                pointer_type: fret_core::PointerType::Mouse,
            }
        } else {
            fret_core::PointerEvent::Up {
                pointer_id: fret_core::PointerId(0),
                position: synth.position,
                button,
                modifiers,
                is_click: true,
                click_count: 1,
                pointer_type: fret_core::PointerType::Mouse,
            }
        };
        ui.dispatch_event(app, services, &Event::Pointer(evt));
        app.request_redraw(window);
    }

    fn dispatch_synth_mouse_wheel(
        ui: &mut UiTree<App>,
        app: &mut App,
        services: &mut dyn UiServices,
        window: AppWindowId,
        synth: &SynthPointerState,
        delta: Point,
        modifiers: Modifiers,
    ) {
        if !synth.enabled {
            return;
        }
        ui.dispatch_event(
            app,
            services,
            &Event::Pointer(fret_core::PointerEvent::Wheel {
                pointer_id: fret_core::PointerId(0),
                position: synth.position,
                delta,
                modifiers,
                pointer_type: fret_core::PointerType::Mouse,
            }),
        );
        app.request_redraw(window);
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
        let synth_pointer_debug = app.models_mut().insert(Arc::<str>::from(
            "synth_pointer: enabled=false id=<unset> pos=(n/a) down=false mouse_right_down=false drag(<none>)",
        ));

        app.with_global_mut(ViewportDebugService::default, |svc, _app| {
            svc.last_event.insert(window, last_viewport_input.clone());
        });

        let mut ui: UiTree<App> = UiTree::new();
        ui.set_window(window);
        ui.set_view_cache_enabled(std::env::var_os("FRET_EXAMPLES_VIEW_CACHE").is_some());

        app.with_global_mut(
            DockingArbitrationPanelModelsService::default,
            |svc, _app| {
                svc.set(
                    window,
                    DockingArbitrationPanelModels {
                        popover_open: popover_open.clone(),
                        dialog_open: dialog_open.clone(),
                        last_viewport_input: last_viewport_input.clone(),
                        synth_pointer_debug: synth_pointer_debug.clone(),
                    },
                );
            },
        );

        DockingArbitrationWindowState {
            ui,
            root: None,
            dock_space: None,
        }
    }

    fn ensure_dock_graph(app: &mut App, window: AppWindowId) {
        use fret_core::{DockNode, PanelKey};

        app.with_global_mut(DockManager::default, |dock, _app| {
            let viewport_left = PanelKey::new("demo.viewport.left");
            let viewport_right = PanelKey::new("demo.viewport.right");
            let controls_panel = PanelKey::new("demo.controls");

            dock.ensure_panel(&viewport_left, || DockPanel {
                title: "Viewport Left".to_string(),
                color: Color::TRANSPARENT,
                viewport: Some(fret_docking::ViewportPanel {
                    target: RenderTargetId::from(KeyData::from_ffi(1)),
                    target_px_size: (960, 540),
                    fit: fret_core::ViewportFit::Stretch,
                    context_menu_enabled: true,
                }),
            });
            dock.ensure_panel(&viewport_right, || DockPanel {
                title: "Viewport Right".to_string(),
                color: Color::TRANSPARENT,
                viewport: Some(fret_docking::ViewportPanel {
                    target: RenderTargetId::from(KeyData::from_ffi(2)),
                    target_px_size: (960, 540),
                    fit: fret_core::ViewportFit::Stretch,
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

            let tabs_left = dock.graph.insert_node(DockNode::Tabs {
                tabs: vec![viewport_left],
                active: 0,
            });
            let tabs_right = dock.graph.insert_node(DockNode::Tabs {
                tabs: vec![viewport_right],
                active: 0,
            });
            let viewport_split = dock.graph.insert_node(DockNode::Split {
                axis: fret_core::Axis::Horizontal,
                children: vec![tabs_left, tabs_right],
                fractions: vec![0.5, 0.5],
            });
            let tabs_controls = dock.graph.insert_node(DockNode::Tabs {
                tabs: vec![controls_panel],
                active: 0,
            });
            let root = dock.graph.insert_node(DockNode::Split {
                axis: fret_core::Axis::Vertical,
                children: vec![viewport_split, tabs_controls],
                fractions: vec![0.7, 0.3],
            });
            dock.graph.set_window_root(window, root);
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
                fret_docking::runtime::request_dock_invalidation(app, dock.graph.windows());
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
                    fret_docking::runtime::request_dock_invalidation(app, [main_window]);
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
                    role: fret_runtime::WindowRole::Auxiliary,
                    style: fret_runtime::WindowStyleRequest::default(),
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

        OverlayController::begin_frame(app, window);

        let dock_space = state.dock_space.get_or_insert_with(|| {
            create_dock_space_node_with_test_id(&mut state.ui, window, "dock-arb-dock-space")
        });
        let _ = state.root.get_or_insert_with(|| {
            let left_anchor = state
                .ui
                .create_node_retained(DockingArbitrationDragAnchor::new(
                    "dock-arb-tab-drag-anchor-left",
                ));
            let right_anchor = state
                .ui
                .create_node_retained(DockingArbitrationDragAnchor::new(
                    "dock-arb-tab-drag-anchor-right",
                ));
            let root = state
                .ui
                .create_node_retained(DockingArbitrationHarnessRoot {
                    dock_space: *dock_space,
                    left_anchor,
                    right_anchor,
                });
            state.ui.set_root(root);
            // Ensure the retained harness nodes participate in hit-testing and event routing.
            // Without explicit parent/child wiring, `layout_in` can position nodes for paint, but
            // pointer hit-testing will not descend into them (it only follows the UI tree).
            state
                .ui
                .set_children(root, vec![*dock_space, left_anchor, right_anchor]);
            root
        });

        render_and_bind_dock_panels(&mut state.ui, app, services, window, bounds, *dock_space);

        OverlayController::render(&mut state.ui, app, services, window, bounds);
    }
}

impl WinitAppDriver for DockingArbitrationDriver {
    type WindowState = DockingArbitrationWindowState;

    fn init(&mut self, app: &mut App, main_window: AppWindowId) {
        self.main_window = Some(main_window);
        self.docking_runtime = Some(DockingRuntime::new(main_window));
        self.logical_windows
            .insert(main_window, Self::MAIN_LOGICAL_WINDOW_ID.to_string());
        self.try_restore_layout_on_init(app, main_window);
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
            state,
            ..
        } = context;
        if state.ui.dispatch_command(app, services, &command) {
            return;
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

        // Demo-only: synthesize a second pointer stream to validate ADR 0072 multi-pointer
        // arbitration even on hardware without a touch screen / pen.
        //
        // F1 toggles the synth pointer. While enabled:
        // - I/J/K/L move the pointer (logical pixels),
        // - Space presses/releases the pointer (Left button semantics).
        // - B presses/releases the mouse right button at the synth pointer position,
        // - U/O emit mouse wheel up/down at the synth pointer position.
        //
        // These keys are consumed while enabled to keep the demo deterministic.
        let mut dispatch_to_ui = true;
        let mut synth_move_after = false;
        let mut synth_button_after: Option<bool> = None;
        let mut synth_mouse_right_after: Option<bool> = None;
        let mut synth_wheel_after: Option<Point> = None;
        let mut synth_mods = Modifiers::default();

        match event {
            Event::KeyDown {
                key: fret_core::KeyCode::F1,
                repeat: false,
                modifiers,
            } => {
                let synth = self.synth_pointer_mut(app, window);
                synth.enabled = !synth.enabled;
                if !synth.enabled && synth.pressed {
                    // Release deterministically before disabling.
                    synth.pressed = false;
                    synth_button_after = Some(false);
                }
                if !synth.enabled && synth.mouse_right_pressed {
                    synth.mouse_right_pressed = false;
                    synth_mouse_right_after = Some(false);
                }
                synth_mods = *modifiers;
                dispatch_to_ui = false;
                DockingArbitrationDriver::update_synth_debug(app, window, synth);
            }
            Event::KeyDown {
                key:
                    fret_core::KeyCode::KeyI
                    | fret_core::KeyCode::KeyJ
                    | fret_core::KeyCode::KeyK
                    | fret_core::KeyCode::KeyL,
                repeat: false,
                modifiers,
            } => {
                let synth = self.synth_pointer_mut(app, window);
                if synth.enabled {
                    let step = 24.0;
                    match event {
                        Event::KeyDown {
                            key: fret_core::KeyCode::KeyI,
                            ..
                        } => synth.position.y.0 -= step,
                        Event::KeyDown {
                            key: fret_core::KeyCode::KeyK,
                            ..
                        } => synth.position.y.0 += step,
                        Event::KeyDown {
                            key: fret_core::KeyCode::KeyJ,
                            ..
                        } => synth.position.x.0 -= step,
                        Event::KeyDown {
                            key: fret_core::KeyCode::KeyL,
                            ..
                        } => synth.position.x.0 += step,
                        _ => {}
                    }

                    if let Some(metrics) = app.global::<fret_core::WindowMetricsService>()
                        && let Some(size) = metrics.inner_size(window)
                    {
                        synth.position.x.0 = synth.position.x.0.clamp(0.0, size.width.0.max(0.0));
                        synth.position.y.0 = synth.position.y.0.clamp(0.0, size.height.0.max(0.0));
                    }

                    synth_mods = *modifiers;
                    synth_move_after = true;
                    dispatch_to_ui = false;
                    DockingArbitrationDriver::update_synth_debug(app, window, synth);
                }
            }
            Event::KeyDown {
                key: fret_core::KeyCode::Space,
                repeat: false,
                modifiers,
            } => {
                let synth = self.synth_pointer_mut(app, window);
                if synth.enabled && !synth.pressed {
                    synth.pressed = true;
                    synth_button_after = Some(true);
                    synth_mods = *modifiers;
                    dispatch_to_ui = false;
                    DockingArbitrationDriver::update_synth_debug(app, window, synth);
                }
            }
            Event::KeyUp {
                key: fret_core::KeyCode::Space,
                modifiers,
            } => {
                let synth = self.synth_pointer_mut(app, window);
                if synth.enabled && synth.pressed {
                    synth.pressed = false;
                    synth_button_after = Some(false);
                    synth_mods = *modifiers;
                    dispatch_to_ui = false;
                    DockingArbitrationDriver::update_synth_debug(app, window, synth);
                }
            }
            Event::KeyDown {
                key: fret_core::KeyCode::KeyB,
                repeat: false,
                modifiers,
            } => {
                let synth = self.synth_pointer_mut(app, window);
                if synth.enabled && !synth.mouse_right_pressed {
                    synth.mouse_right_pressed = true;
                    synth_mouse_right_after = Some(true);
                    synth_mods = *modifiers;
                    dispatch_to_ui = false;
                    DockingArbitrationDriver::update_synth_debug(app, window, synth);
                }
            }
            Event::KeyUp {
                key: fret_core::KeyCode::KeyB,
                modifiers,
            } => {
                let synth = self.synth_pointer_mut(app, window);
                if synth.enabled && synth.mouse_right_pressed {
                    synth.mouse_right_pressed = false;
                    synth_mouse_right_after = Some(false);
                    synth_mods = *modifiers;
                    dispatch_to_ui = false;
                    DockingArbitrationDriver::update_synth_debug(app, window, synth);
                }
            }
            Event::KeyDown {
                key: fret_core::KeyCode::KeyU | fret_core::KeyCode::KeyO,
                repeat: false,
                modifiers,
            } => {
                let synth = self.synth_pointer_mut(app, window);
                if synth.enabled {
                    // Positive wheel Y means scrolling up (winit semantics).
                    let delta = match event {
                        Event::KeyDown {
                            key: fret_core::KeyCode::KeyU,
                            ..
                        } => Point::new(Px(0.0), Px(24.0)),
                        Event::KeyDown {
                            key: fret_core::KeyCode::KeyO,
                            ..
                        } => Point::new(Px(0.0), Px(-24.0)),
                        _ => Point::new(Px(0.0), Px(0.0)),
                    };
                    synth_wheel_after = Some(delta);
                    synth_mods = *modifiers;
                    dispatch_to_ui = false;
                }
            }
            _ => {}
        }

        if let Event::KeyDown {
            key: fret_core::KeyCode::KeyQ | fret_core::KeyCode::KeyW | fret_core::KeyCode::KeyE,
            repeat: false,
            ..
        } = event
        {
            let mode = match event {
                Event::KeyDown {
                    key: fret_core::KeyCode::KeyQ,
                    ..
                } => fret_editor::ViewportToolMode::Select,
                Event::KeyDown {
                    key: fret_core::KeyCode::KeyW,
                    ..
                } => fret_editor::ViewportToolMode::Move,
                Event::KeyDown {
                    key: fret_core::KeyCode::KeyE,
                    ..
                } => fret_editor::ViewportToolMode::Rotate,
                _ => return,
            };

            if let Ok(mut tools) = self.viewport_tools.lock() {
                let mut did_change = false;
                for ((w, _target), mgr) in tools.tools.iter_mut() {
                    if *w != window {
                        continue;
                    }
                    if mgr.active != mode {
                        mgr.active = mode;
                        mgr.interaction = None;
                        did_change = true;
                    }
                }
                if did_change {
                    app.request_redraw(window);
                }
            }
        }

        if dispatch_to_ui {
            state.ui.dispatch_event(app, services, event);
        }

        // Inject synth pointer events after we update the synth state, so dock/overlay policies
        // observe the correct final state for this frame.
        if let Some(st) = self.synth_pointers.get(&window).cloned() {
            if synth_move_after {
                DockingArbitrationDriver::dispatch_synth_pointer_move(
                    &mut state.ui,
                    app,
                    services,
                    window,
                    &st,
                    synth_mods,
                );
            }
            if let Some(pressed) = synth_button_after {
                DockingArbitrationDriver::dispatch_synth_pointer_button(
                    &mut state.ui,
                    app,
                    services,
                    window,
                    &st,
                    pressed,
                    synth_mods,
                );
            }
            if let Some(pressed) = synth_mouse_right_after {
                DockingArbitrationDriver::dispatch_synth_mouse_button(
                    &mut state.ui,
                    app,
                    services,
                    window,
                    &st,
                    MouseButton::Right,
                    pressed,
                    synth_mods,
                );
            }
            if let Some(delta) = synth_wheel_after {
                DockingArbitrationDriver::dispatch_synth_mouse_wheel(
                    &mut state.ui,
                    app,
                    services,
                    window,
                    &st,
                    delta,
                    synth_mods,
                );
            }
        }
    }

    fn viewport_input(&mut self, app: &mut App, event: ViewportInputEvent) {
        app.with_global_mut_untracked(UiDiagnosticsService::default, |svc, _app| {
            svc.record_viewport_input(event.clone());
        });

        let cursor_target_px = event
            .cursor_target_px_f32()
            .map(|(x, y)| format!("{x:.1},{y:.1}"))
            .unwrap_or_else(|| "n/a".to_string());
        let target_px_per_screen_px = event.target_px_per_screen_px().unwrap_or(0.0);
        let msg: Arc<str> = Arc::from(
            format!(
                "{:?} cursor_px=({:.1},{:.1}) uv=({:.3},{:.3}) target_px=({}, {}) cursor_target_px=({}) target_px_per_screen_px={:.3} target={:?} window={:?}",
                event.kind,
                event.cursor_px.x.0,
                event.cursor_px.y.0,
                event.uv.0,
                event.uv.1,
                event.target_px.0,
                event.target_px.1,
                cursor_target_px,
                target_px_per_screen_px,
                event.target,
                event.window,
            )
            .into_boxed_str(),
        );
        app.with_global_mut(ViewportDebugService::default, |svc, app| {
            if let Some(model) = svc.last_event.get(&event.window).cloned() {
                let _ = app.models_mut().update(&model, |v| *v = msg.clone());
                app.request_redraw(event.window);
            }
        });

        if let Ok(mut state) = self.viewport_tools.lock() {
            let key = (event.window, event.target);
            let mgr = state.tools.entry(key).or_insert_with(Default::default);
            if mgr.handle_viewport_input(&event) {
                app.request_redraw(event.window);
            }
        }
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
        DockingArbitrationDriver::render_dock(app, services, window, state, bounds);

        state.ui.request_semantics_snapshot();
        state.ui.ingest_paint_cache_source(scene);

        let inspection_active = app
            .with_global_mut_untracked(UiDiagnosticsService::default, |svc, _app| {
                svc.wants_inspection_active(window)
            });
        state.ui.set_inspection_active(inspection_active);

        let diag_enabled = app
            .with_global_mut_untracked(UiDiagnosticsService::default, |svc, _app| svc.is_enabled());
        if diag_enabled {
            app.with_global_mut_untracked(
                fret_runtime::WindowInteractionDiagnosticsStore::default,
                |store, app| store.begin_frame(window, app.frame_id()),
            );
        }

        scene.clear();
        let mut frame =
            fret_ui::UiFrameCx::new(&mut state.ui, app, services, window, bounds, scale_factor);
        frame.layout_all();

        let semantics_snapshot = state.ui.semantics_snapshot();
        let drive = app.with_global_mut_untracked(UiDiagnosticsService::default, |svc, app| {
            let element_runtime = app.global::<fret_ui::elements::ElementRuntime>();
            svc.drive_script_for_window(
                app,
                window,
                bounds,
                scale_factor,
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

        if let Some(synth) = self.synth_pointers.get(&window)
            && synth.enabled
        {
            let red = Color {
                r: 1.0,
                g: 0.0,
                b: 0.0,
                a: 1.0,
            };
            let half = 6.0;
            let rect = Rect::new(
                Point::new(Px(synth.position.x.0 - half), Px(synth.position.y.0 - half)),
                Size::new(Px(half * 2.0), Px(half * 2.0)),
            );
            scene.push(SceneOp::Quad {
                order: DrawOrder(10_000),
                rect,
                background: fret_core::Paint::Solid(if synth.pressed {
                    Color { a: 0.25, ..red }
                } else {
                    Color::TRANSPARENT
                }),
                border: Edges::all(Px(2.0)),
                border_paint: fret_core::Paint::Solid(red),

                corner_radii: Corners::all(Px(2.0)),
            });
        }

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
                let _ = self
                    .docking_runtime
                    .as_ref()
                    .map(|rt| rt.on_window_created(app, request, new_window))
                    .unwrap_or(false);
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
    let viewport_tools = Arc::new(Mutex::new(DemoViewportToolState::default()));
    let mut caps = PlatformCapabilities::default();
    if std::env::var("FRET_SINGLE_WINDOW")
        .ok()
        .is_some_and(|v| v == "1" || v.eq_ignore_ascii_case("true"))
    {
        caps.ui.multi_window = false;
        caps.ui.window_tear_off = true;
    }
    app.set_global(caps);
    app.with_global_mut(DockPanelRegistryService::<App>::default, |svc, _app| {
        svc.set(Arc::new(DockingArbitrationDockPanelRegistry));
    });
    app.with_global_mut(DockViewportOverlayHooksService::default, |svc, _app| {
        svc.set(Arc::new(DemoViewportOverlayHooks {
            tools: viewport_tools.clone(),
        }));
    });

    let config = WinitRunnerConfig {
        main_window_title: "fret-demo docking_arbitration_demo".to_string(),
        main_window_size: winit::dpi::LogicalSize::new(980.0, 720.0),
        ..Default::default()
    };

    let pending_layout =
        fret_app::DockLayoutFileV1::load_json_if_exists(DockingArbitrationDriver::DOCK_LAYOUT_PATH)
            .map(|v| v.map(|f| f.layout))
            .unwrap_or_else(|err| {
                tracing::warn!("failed to load dock layout: {err}");
                None
            });

    let driver = DockingArbitrationDriver::new(pending_layout, viewport_tools);
    fret_kit::run_native_demo(config, app, driver).context("run docking_arbitration_demo app")
}
