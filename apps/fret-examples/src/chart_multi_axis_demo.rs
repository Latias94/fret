#[cfg(not(target_arch = "wasm32"))]
use anyhow::Context as _;
use fret_app::{App, Effect, WindowRequest};
use fret_bootstrap::ui_diagnostics::UiDiagnosticsService;
use fret_core::{AppWindowId, Event};
#[cfg(not(target_arch = "wasm32"))]
use fret_launch::run_app;
use fret_launch::{
    WindowCreateSpec, WinitAppDriver, WinitEventContext, WinitRenderContext, WinitRunnerConfig,
    WinitWindowContext,
};
use fret_runtime::PlatformCapabilities;
use fret_ui::{FixedSplit, Invalidation, UiTree};
use std::collections::BTreeMap;

use delinea::data::{Column, DataTable};
use delinea::engine::ChartEngine;
use delinea::engine::window::DataWindow;
use delinea::ids::{AxisId, FieldId, LinkGroupId, VisualMapId};
use delinea::{
    Action, ChartSpec, DataZoomXSpec, DataZoomYSpec, DatasetSpec, FieldSpec, FilterMode, GridSpec,
    SeriesEncode, SeriesSpec, VisualMapSpec,
};
use delinea::{AxisKind, AxisPosition, AxisScale, SeriesKind};
use fret_chart::retained::{ChartCanvas, ChartCanvasOutput};
use fret_chart::{
    AxisPointerLinkAnchor, BrushSelectionLink2D, ChartLinkPolicy, ChartLinkRouter, LinkAxisKey,
    LinkedChartGroup, LinkedChartMember,
};
use fret_runtime::Model;

struct ChartMultiAxisDemoWindowState {
    ui: UiTree<App>,
    root: Option<fret_core::NodeId>,
    top_node: Option<fret_core::NodeId>,
    bottom_node: Option<fret_core::NodeId>,
    top_engine: Option<std::rc::Rc<std::cell::RefCell<ChartEngine>>>,
    bottom_engine: Option<std::rc::Rc<std::cell::RefCell<ChartEngine>>>,
    linked: Option<LinkedChartGroup>,
    shared_brush: Model<Option<BrushSelectionLink2D>>,
    shared_axis_pointer: Model<Option<AxisPointerLinkAnchor>>,
    shared_domain_windows: Model<BTreeMap<LinkAxisKey, Option<DataWindow>>>,
    top_output: Model<ChartCanvasOutput>,
    bottom_output: Model<ChartCanvasOutput>,
    last_diag_shared_domain_windows: BTreeMap<LinkAxisKey, Option<DataWindow>>,
    last_diag_top_domain_windows: BTreeMap<LinkAxisKey, Option<DataWindow>>,
    last_diag_bottom_domain_windows: BTreeMap<LinkAxisKey, Option<DataWindow>>,
    diag_auto_zoom_frame_id: Option<u64>,
    diag_auto_zoom_done: bool,
}

#[derive(Default)]
struct ChartMultiAxisDemoDriver;

impl ChartMultiAxisDemoDriver {
    fn build_ui(app: &mut App, window: AppWindowId) -> ChartMultiAxisDemoWindowState {
        let mut ui: UiTree<App> = UiTree::new();
        ui.set_window(window);

        eprintln!(
            "[chart_multi_axis_demo] X minSpan/maxSpan are enforced for interaction-derived zoom writes only.\n\
             If the current span is already outside limits (e.g. programmatic set), interactions do not force it back.\n\
             Config:\n\
             - X minSpan=50, maxSpan=2000; initial window [-200, 200].\n\
             - Y(left) minSpan=2, maxSpan=40; initial window [-15, 15].\n\
             - Y(right) minSpan=50, maxSpan=2000; initial window [0, 1600].\n\
             \n\
             Brush linking (P0):\n\
             - Both charts are in the same LinkGroup.\n\
             - Use Alt + RMB drag to brush-select; the selection is mirrored to the other chart.\n\
             - This demo uses fret-chart LinkedChartGroup to route delinea link events by LinkAxisKey."
        );

        let shared_brush = app.models_mut().insert(None::<BrushSelectionLink2D>);
        let shared_axis_pointer = app.models_mut().insert(None::<AxisPointerLinkAnchor>);
        let shared_domain_windows = app
            .models_mut()
            .insert(BTreeMap::<LinkAxisKey, Option<DataWindow>>::default());
        let top_output = app.models_mut().insert(ChartCanvasOutput::default());
        let bottom_output = app.models_mut().insert(ChartCanvasOutput::default());

        ChartMultiAxisDemoWindowState {
            ui,
            root: None,
            top_node: None,
            bottom_node: None,
            top_engine: None,
            bottom_engine: None,
            linked: None,
            shared_brush,
            shared_axis_pointer,
            shared_domain_windows,
            top_output,
            bottom_output,
            last_diag_shared_domain_windows: BTreeMap::new(),
            last_diag_top_domain_windows: BTreeMap::new(),
            last_diag_bottom_domain_windows: BTreeMap::new(),
            diag_auto_zoom_frame_id: None,
            diag_auto_zoom_done: false,
        }
    }

    fn build_canvas(
        chart_id: delinea::ids::ChartId,
        shared_brush: Model<Option<BrushSelectionLink2D>>,
        shared_axis_pointer: Model<Option<AxisPointerLinkAnchor>>,
        shared_domain_windows: Model<BTreeMap<LinkAxisKey, Option<DataWindow>>>,
        output: Model<ChartCanvasOutput>,
    ) -> (
        ChartCanvas,
        ChartLinkRouter,
        std::rc::Rc<std::cell::RefCell<ChartEngine>>,
    ) {
        let dataset_id = delinea::ids::DatasetId::new(1);
        let grid_id = delinea::ids::GridId::new(1);

        let x_bottom = AxisId::new(1);
        let x_top = AxisId::new(2);
        let y_left = AxisId::new(3);
        let y_right = AxisId::new(4);

        let x1_field = FieldId::new(1);
        let x2_field = FieldId::new(2);
        let y_left_a_field = FieldId::new(3);
        let y_left_b_field = FieldId::new(4);
        let y_right_a_field = FieldId::new(5);
        let y_right_b_field = FieldId::new(6);

        let spec = ChartSpec {
            id: chart_id,
            viewport: None,
            datasets: vec![DatasetSpec {
                id: dataset_id,
                fields: vec![
                    FieldSpec {
                        id: x1_field,
                        column: 0,
                    },
                    FieldSpec {
                        id: x2_field,
                        column: 1,
                    },
                    FieldSpec {
                        id: y_left_a_field,
                        column: 2,
                    },
                    FieldSpec {
                        id: y_left_b_field,
                        column: 3,
                    },
                    FieldSpec {
                        id: y_right_a_field,
                        column: 4,
                    },
                    FieldSpec {
                        id: y_right_b_field,
                        column: 5,
                    },
                ],
                ..Default::default()
            }],
            grids: vec![GridSpec { id: grid_id }],
            axes: vec![
                delinea::AxisSpec {
                    id: x_bottom,
                    name: Some("X (bottom) [minSpan=50 maxSpan=2000]".to_string()),
                    kind: AxisKind::X,
                    grid: grid_id,
                    position: Some(AxisPosition::Bottom),
                    scale: AxisScale::Value(Default::default()),
                    range: None,
                },
                delinea::AxisSpec {
                    id: x_top,
                    name: Some("X (top) [minSpan=50 maxSpan=2000]".to_string()),
                    kind: AxisKind::X,
                    grid: grid_id,
                    position: Some(AxisPosition::Top),
                    scale: AxisScale::Value(Default::default()),
                    range: None,
                },
                delinea::AxisSpec {
                    id: y_left,
                    name: Some("Y (left) [minSpan=2 maxSpan=40]".to_string()),
                    kind: AxisKind::Y,
                    grid: grid_id,
                    position: Some(AxisPosition::Left),
                    scale: AxisScale::Value(Default::default()),
                    range: None,
                },
                delinea::AxisSpec {
                    id: y_right,
                    name: Some("Y (right) [minSpan=50 maxSpan=2000]".to_string()),
                    kind: AxisKind::Y,
                    grid: grid_id,
                    position: Some(AxisPosition::Right),
                    scale: AxisScale::Value(Default::default()),
                    range: None,
                },
            ],
            data_zoom_x: vec![
                DataZoomXSpec {
                    id: delinea::ids::DataZoomId::new(1),
                    axis: x_bottom,
                    filter_mode: FilterMode::Filter,
                    min_value_span: Some(50.0),
                    max_value_span: Some(2000.0),
                },
                DataZoomXSpec {
                    id: delinea::ids::DataZoomId::new(2),
                    axis: x_top,
                    filter_mode: FilterMode::Filter,
                    min_value_span: Some(50.0),
                    max_value_span: Some(2000.0),
                },
            ],
            data_zoom_y: vec![
                DataZoomYSpec {
                    id: delinea::ids::DataZoomId::new(10),
                    axis: y_left,
                    filter_mode: FilterMode::None,
                    min_value_span: Some(2.0),
                    max_value_span: Some(40.0),
                },
                DataZoomYSpec {
                    id: delinea::ids::DataZoomId::new(11),
                    axis: y_right,
                    filter_mode: FilterMode::None,
                    min_value_span: Some(50.0),
                    max_value_span: Some(2000.0),
                },
            ],
            tooltip: None,
            axis_pointer: Some(delinea::AxisPointerSpec::default()),
            visual_maps: vec![
                VisualMapSpec {
                    id: VisualMapId::new(1),
                    mode: delinea::VisualMapMode::Continuous,
                    dataset: None,
                    series: vec![delinea::ids::SeriesId::new(3)],
                    field: y_left_b_field,
                    domain: (-4.0, 4.0),
                    initial_range: Some((-1.0, 1.0)),
                    initial_piece_mask: None,
                    point_radius_mul_range: Some((0.6, 1.8)),
                    stroke_width_range: None,
                    opacity_mul_range: Some((0.15, 1.0)),
                    buckets: 8,
                    out_of_range_opacity: 0.25,
                },
                VisualMapSpec {
                    id: VisualMapId::new(2),
                    mode: delinea::VisualMapMode::Piecewise,
                    dataset: None,
                    series: vec![delinea::ids::SeriesId::new(4)],
                    field: y_right_b_field,
                    domain: (0.0, 1600.0),
                    initial_range: None,
                    initial_piece_mask: None,
                    point_radius_mul_range: Some((0.8, 1.4)),
                    stroke_width_range: None,
                    opacity_mul_range: None,
                    buckets: 8,
                    out_of_range_opacity: 0.25,
                },
            ],
            series: vec![
                SeriesSpec {
                    id: delinea::ids::SeriesId::new(1),
                    name: Some("Bottom X / Left Y (line)".to_string()),
                    kind: SeriesKind::Line,
                    dataset: dataset_id,
                    encode: SeriesEncode {
                        x: x1_field,
                        y: y_left_a_field,
                        y2: None,
                    },
                    x_axis: x_bottom,
                    y_axis: y_left,
                    stack: None,
                    stack_strategy: Default::default(),
                    bar_layout: Default::default(),
                    area_baseline: None,
                    lod: None,
                },
                SeriesSpec {
                    id: delinea::ids::SeriesId::new(2),
                    name: Some("Bottom X / Right Y (line)".to_string()),
                    kind: SeriesKind::Line,
                    dataset: dataset_id,
                    encode: SeriesEncode {
                        x: x1_field,
                        y: y_right_a_field,
                        y2: None,
                    },
                    x_axis: x_bottom,
                    y_axis: y_right,
                    stack: None,
                    stack_strategy: Default::default(),
                    bar_layout: Default::default(),
                    area_baseline: None,
                    lod: None,
                },
                SeriesSpec {
                    id: delinea::ids::SeriesId::new(3),
                    name: Some("Top X / Left Y (scatter)".to_string()),
                    kind: SeriesKind::Scatter,
                    dataset: dataset_id,
                    encode: SeriesEncode {
                        x: x2_field,
                        y: y_left_b_field,
                        y2: None,
                    },
                    x_axis: x_top,
                    y_axis: y_left,
                    stack: None,
                    stack_strategy: Default::default(),
                    bar_layout: Default::default(),
                    area_baseline: None,
                    lod: None,
                },
                SeriesSpec {
                    id: delinea::ids::SeriesId::new(4),
                    name: Some("Top X / Right Y (scatter)".to_string()),
                    kind: SeriesKind::Scatter,
                    dataset: dataset_id,
                    encode: SeriesEncode {
                        x: x2_field,
                        y: y_right_b_field,
                        y2: None,
                    },
                    x_axis: x_top,
                    y_axis: y_right,
                    stack: None,
                    stack_strategy: Default::default(),
                    bar_layout: Default::default(),
                    area_baseline: None,
                    lod: None,
                },
            ],
        };

        let router = ChartLinkRouter::from_spec(&spec);
        let test_id = if chart_id.0 == 1 {
            "chart-multi-axis-top"
        } else {
            "chart-multi-axis-bottom"
        };
        let engine = std::rc::Rc::new(std::cell::RefCell::new(
            ChartEngine::new(spec).expect("chart spec should be valid"),
        ));

        {
            let mut e = engine.borrow_mut();
            e.apply_action(Action::SetLinkGroup {
                group: Some(LinkGroupId::new(1)),
            });

            let n = 4096usize;
            let mut x1: Vec<f64> = Vec::with_capacity(n);
            let mut x2: Vec<f64> = Vec::with_capacity(n);
            let mut y_left_a: Vec<f64> = Vec::with_capacity(n);
            let mut y_left_b: Vec<f64> = Vec::with_capacity(n);
            let mut y_right_a: Vec<f64> = Vec::with_capacity(n);
            let mut y_right_b: Vec<f64> = Vec::with_capacity(n);

            for i in 0..n {
                let t = i as f64 / (n - 1).max(1) as f64;
                let theta = t * std::f64::consts::TAU;

                let x = (t * 2000.0) - 1000.0;
                x1.push(x);
                x2.push(x);

                y_left_a.push((theta * 2.0).sin() * 20.0);
                y_left_b.push((theta * 16.0).sin() * 2.0 + (theta * 3.0).cos() * 1.5);

                y_right_a.push((theta * 1.25).cos() * 400.0 + 1000.0);
                y_right_b.push((theta * 8.0).cos() * 75.0 + 200.0);
            }

            let mut table = DataTable::default();
            table.push_column(Column::F64(x1));
            table.push_column(Column::F64(x2));
            table.push_column(Column::F64(y_left_a));
            table.push_column(Column::F64(y_left_b));
            table.push_column(Column::F64(y_right_a));
            table.push_column(Column::F64(y_right_b));
            e.datasets_mut().insert(dataset_id, table);

            e.apply_action(Action::SetDataWindowX {
                axis: x_bottom,
                window: Some(DataWindow {
                    min: -200.0,
                    max: 200.0,
                }),
            });
            e.apply_action(Action::SetDataWindowX {
                axis: x_top,
                window: Some(DataWindow {
                    min: -200.0,
                    max: 200.0,
                }),
            });

            e.apply_action(Action::SetDataWindowY {
                axis: y_left,
                window: Some(DataWindow {
                    min: -15.0,
                    max: 15.0,
                }),
            });
            e.apply_action(Action::SetDataWindowY {
                axis: y_right,
                window: Some(DataWindow {
                    min: 0.0,
                    max: 1600.0,
                }),
            });
        }

        let canvas = ChartCanvas::new_shared(engine.clone())
            .linked_brush(shared_brush)
            .linked_axis_pointer(shared_axis_pointer)
            .linked_domain_windows(shared_domain_windows)
            .output_model(output)
            .test_id(test_id);

        (canvas, router, engine)
    }

    fn tick_linking(app: &mut App, window: AppWindowId, state: &mut ChartMultiAxisDemoWindowState) {
        let Some(linked) = state.linked.as_mut() else {
            return;
        };

        if linked.tick(app) {
            if let Some(root) = state.root {
                state.ui.invalidate(root, Invalidation::Paint);
            }
            if let Some(node) = state.top_node {
                state.ui.invalidate(node, Invalidation::Paint);
            }
            if let Some(node) = state.bottom_node {
                state.ui.invalidate(node, Invalidation::Paint);
            }
            app.request_redraw(window);
            app.push_effect(Effect::RequestAnimationFrame(window));
        }
    }

    fn maybe_log_link_events_for_diag(
        app: &mut App,
        window: AppWindowId,
        state: &mut ChartMultiAxisDemoWindowState,
    ) {
        let diag_enabled = app
            .with_global_mut_untracked(UiDiagnosticsService::default, |svc, _app| svc.is_enabled());
        if !diag_enabled {
            return;
        }

        let top = state
            .top_output
            .read(app, |_app, o| o.snapshot.link_events.clone())
            .ok()
            .unwrap_or_default();
        let bottom = state
            .bottom_output
            .read(app, |_app, o| o.snapshot.link_events.clone())
            .ok()
            .unwrap_or_default();
        let shared_domain_windows = state
            .shared_domain_windows
            .read(app, |_app, w| w.clone())
            .ok()
            .unwrap_or_default();
        let top_domain_windows = state
            .top_output
            .read(app, |_app, o| o.snapshot.domain_windows_by_key.clone())
            .ok()
            .unwrap_or_default();
        let bottom_domain_windows = state
            .bottom_output
            .read(app, |_app, o| o.snapshot.domain_windows_by_key.clone())
            .ok()
            .unwrap_or_default();

        let any_changed = state.last_diag_shared_domain_windows != shared_domain_windows
            || state.last_diag_top_domain_windows != top_domain_windows
            || state.last_diag_bottom_domain_windows != bottom_domain_windows;
        if top.is_empty() && bottom.is_empty() && !any_changed {
            return;
        }

        state.last_diag_shared_domain_windows = shared_domain_windows.clone();
        state.last_diag_top_domain_windows = top_domain_windows.clone();
        state.last_diag_bottom_domain_windows = bottom_domain_windows.clone();

        let summarize = |events: &[delinea::LinkEvent]| -> Option<String> {
            events.iter().rev().find_map(|e| match e {
                delinea::LinkEvent::DomainWindowChanged { axis, window } => Some(format!(
                    "DomainWindowChanged(axis={axis:?} window={window:?})"
                )),
                delinea::LinkEvent::AxisPointerChanged { anchor } => {
                    Some(format!("AxisPointerChanged(anchor={anchor:?})"))
                }
                delinea::LinkEvent::BrushSelectionChanged { selection } => {
                    Some(format!("BrushSelectionChanged(selection={selection:?})"))
                }
            })
        };

        println!(
            "[chart_multi_axis_demo][diag] window={window:?} link_events: top={} bottom={} top_last={} bottom_last={} shared_domain_windows={:?} top_domain_windows={:?} bottom_domain_windows={:?}",
            top.len(),
            bottom.len(),
            summarize(&top).unwrap_or_else(|| "<none>".to_string()),
            summarize(&bottom).unwrap_or_else(|| "<none>".to_string()),
            shared_domain_windows,
            top_domain_windows,
            bottom_domain_windows,
        );
    }

    fn maybe_apply_diag_auto_zoom_top(
        app: &mut App,
        window: AppWindowId,
        state: &mut ChartMultiAxisDemoWindowState,
    ) {
        if state.diag_auto_zoom_done {
            return;
        }

        let diag_enabled = app
            .with_global_mut_untracked(UiDiagnosticsService::default, |svc, _app| svc.is_enabled());
        if !diag_enabled {
            return;
        }

        // Wait until the diag harness has captured its "before" screenshot so the pixels-changed
        // gate compares a real baseline against the post-zoom frame.
        const DIAG_AUTO_ZOOM_AFTER_NEXT_STEP: u32 = 6;
        let next_step = app
            .with_global_mut_untracked(UiDiagnosticsService::default, |svc, _app| {
                svc.active_script_next_step_index(window)
            });
        if next_step.is_some_and(|step| step < DIAG_AUTO_ZOOM_AFTER_NEXT_STEP) {
            app.push_effect(Effect::RequestAnimationFrame(window));
            return;
        }

        let frame_id = app.frame_id().0;
        let apply_at = state
            .diag_auto_zoom_frame_id
            .get_or_insert(frame_id.saturating_add(30));
        if frame_id < *apply_at {
            // Ensure the frame clock advances in diag mode even when the script isn't injecting
            // input events yet.
            app.push_effect(Effect::RequestAnimationFrame(window));
            return;
        }

        let Some(engine) = state.top_engine.as_ref() else {
            return;
        };

        // Apply a deterministic X-domain window change on the primary X axis. This acts as a
        // stable producer signal for `LinkedChartGroup` and avoids relying on pointer input
        // dispatch in the diag harness.
        let before_zoom = engine
            .borrow()
            .state()
            .data_zoom_x
            .get(&AxisId::new(1))
            .and_then(|s| s.window);
        println!(
            "[chart_multi_axis_demo][diag] applying deterministic top auto-zoom window at frame_id={frame_id} next_step={next_step:?} before_zoom={before_zoom:?}"
        );
        engine.borrow_mut().apply_action(Action::SetDataWindowX {
            axis: AxisId::new(1),
            window: Some(DataWindow {
                min: -75.0,
                max: 75.0,
            }),
        });
        let after_zoom = engine
            .borrow()
            .state()
            .data_zoom_x
            .get(&AxisId::new(1))
            .and_then(|s| s.window);
        println!("[chart_multi_axis_demo][diag] top auto-zoom applied: after_zoom={after_zoom:?}");
        state.diag_auto_zoom_done = true;

        if let Some(node) = state.top_node {
            state.ui.invalidate(node, Invalidation::Paint);
        }
        if let Some(node) = state.bottom_node {
            state.ui.invalidate(node, Invalidation::Paint);
        }
        app.request_redraw(window);
        app.push_effect(Effect::RequestAnimationFrame(window));
    }
}

impl WinitAppDriver for ChartMultiAxisDemoDriver {
    type WindowState = ChartMultiAxisDemoWindowState;

    fn create_window_state(&mut self, app: &mut App, window: AppWindowId) -> Self::WindowState {
        let state = Self::build_ui(app, window);
        app.request_redraw(window);
        state
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

        Self::tick_linking(context.app, context.window, context.state);
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

    fn handle_event(&mut self, context: WinitEventContext<'_, Self::WindowState>, event: &Event) {
        let WinitEventContext {
            app,
            services,
            window,
            state,
            ..
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

        match event {
            Event::WindowCloseRequested
            | Event::KeyDown {
                key: fret_core::KeyCode::Escape,
                ..
            } => {
                app.push_effect(Effect::Window(WindowRequest::Close(window)));
                return;
            }
            _ => {
                state.ui.dispatch_event(app, services, event);
            }
        }

        if let Some(linked) = state.linked.as_mut() {
            if linked.tick(app) {
                app.request_redraw(window);
            }
        }
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

        let diag_enabled = app
            .with_global_mut_untracked(UiDiagnosticsService::default, |svc, _app| svc.is_enabled());
        if diag_enabled {
            // Diag harnesses rely on deterministic visual diffs. Disable view/paint caching so
            // externally-driven engine state changes always re-render and publish fresh outputs.
            state.ui.set_view_cache_enabled(false);
            state.ui.set_paint_cache_enabled(false);
        }

        if state.root.is_none() {
            let shared_brush = state.shared_brush.clone();
            let shared_axis_pointer = state.shared_axis_pointer.clone();
            let shared_domain_windows = state.shared_domain_windows.clone();

            let (top_canvas, top_router, top_engine) = Self::build_canvas(
                delinea::ids::ChartId::new(1),
                shared_brush.clone(),
                shared_axis_pointer.clone(),
                shared_domain_windows.clone(),
                state.top_output.clone(),
            );
            let (bottom_canvas, bottom_router, bottom_engine) = Self::build_canvas(
                delinea::ids::ChartId::new(2),
                shared_brush,
                shared_axis_pointer,
                shared_domain_windows,
                state.bottom_output.clone(),
            );

            state.top_engine = Some(top_engine);
            state.bottom_engine = Some(bottom_engine);

            if state.linked.is_none() {
                let policy = ChartLinkPolicy {
                    brush: true,
                    axis_pointer: true,
                    domain_windows: true,
                };
                let mut linked = LinkedChartGroup::new(
                    policy,
                    state.shared_brush.clone(),
                    state.shared_axis_pointer.clone(),
                    state.shared_domain_windows.clone(),
                );
                linked
                    .push(LinkedChartMember {
                        router: top_router,
                        output: state.top_output.clone(),
                    })
                    .push(LinkedChartMember {
                        router: bottom_router,
                        output: state.bottom_output.clone(),
                    });
                state.linked = Some(linked);
            }

            let top_node = ChartCanvas::create_node(&mut state.ui, top_canvas);
            let bottom_node = ChartCanvas::create_node(&mut state.ui, bottom_canvas);
            let root = FixedSplit::create_node_with_children(
                &mut state.ui,
                FixedSplit::vertical(0.5),
                top_node,
                bottom_node,
            );

            state.ui.set_root(root);
            state.ui.set_focus(Some(top_node));
            state.root = Some(root);
            state.top_node = Some(top_node);
            state.bottom_node = Some(bottom_node);
        }

        // Tick linking before layout/paint so any outputs from the previous frame are
        // propagated into shared models before the canvases try to apply them.
        Self::tick_linking(app, window, state);
        Self::maybe_apply_diag_auto_zoom_top(app, window, state);

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

        Self::maybe_log_link_events_for_diag(app, window, state);

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
            if svc.poll_exit_trigger() {
                app.push_effect(Effect::QuitApp);
            } else if svc.is_enabled() {
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
        _app: &mut App,
        _request: &fret_app::CreateWindowRequest,
        _new_window: AppWindowId,
    ) {
    }
}

pub fn build_app() -> App {
    let mut app = App::new();
    app.set_global(PlatformCapabilities::default());
    app
}

pub fn build_runner_config() -> WinitRunnerConfig {
    WinitRunnerConfig {
        main_window_title:
            "fret-demo chart_multi_axis_demo (delinea + fret-chart, minSpan/maxSpan + brushLink demo)"
                .to_string(),
        ..Default::default()
    }
}

pub fn build_driver() -> impl WinitAppDriver {
    ChartMultiAxisDemoDriver::default()
}

#[cfg(not(target_arch = "wasm32"))]
pub fn run() -> anyhow::Result<()> {
    let app = build_app();
    let config = build_runner_config();
    let driver = build_driver();

    run_app(config, app, driver)
        .context("run chart_multi_axis_demo app")
        .map_err(anyhow::Error::from)
}

#[cfg(target_arch = "wasm32")]
pub fn run() -> anyhow::Result<()> {
    Ok(())
}
