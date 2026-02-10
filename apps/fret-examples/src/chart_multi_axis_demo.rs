#[cfg(not(target_arch = "wasm32"))]
use anyhow::Context as _;
use fret_app::{App, Effect, WindowRequest};
use fret_core::{AppWindowId, Event};
#[cfg(not(target_arch = "wasm32"))]
use fret_launch::run_app;
use fret_launch::{
    WindowCreateSpec, WinitAppDriver, WinitEventContext, WinitRenderContext, WinitRunnerConfig,
};
use fret_runtime::PlatformCapabilities;
use fret_ui::{FixedSplit, UiTree};
use std::collections::BTreeMap;

use delinea::data::{Column, DataTable};
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
    linked: Option<LinkedChartGroup>,
    shared_brush: Model<Option<BrushSelectionLink2D>>,
    shared_axis_pointer: Model<Option<AxisPointerLinkAnchor>>,
    shared_domain_windows: Model<BTreeMap<LinkAxisKey, Option<DataWindow>>>,
    top_output: Model<ChartCanvasOutput>,
    bottom_output: Model<ChartCanvasOutput>,
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
            linked: None,
            shared_brush,
            shared_axis_pointer,
            shared_domain_windows,
            top_output,
            bottom_output,
        }
    }

    fn build_canvas(
        chart_id: delinea::ids::ChartId,
        shared_brush: Model<Option<BrushSelectionLink2D>>,
        shared_axis_pointer: Model<Option<AxisPointerLinkAnchor>>,
        shared_domain_windows: Model<BTreeMap<LinkAxisKey, Option<DataWindow>>>,
        output: Model<ChartCanvasOutput>,
    ) -> (ChartCanvas, ChartLinkRouter) {
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
        let mut canvas = ChartCanvas::new(spec)
            .expect("chart spec should be valid")
            .linked_brush(shared_brush)
            .linked_axis_pointer(shared_axis_pointer)
            .linked_domain_windows(shared_domain_windows)
            .output_model(output);

        canvas.engine_mut().apply_action(Action::SetLinkGroup {
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
        canvas.engine_mut().datasets_mut().insert(dataset_id, table);

        canvas.engine_mut().apply_action(Action::SetDataWindowX {
            axis: x_bottom,
            window: Some(DataWindow {
                min: -200.0,
                max: 200.0,
            }),
        });
        canvas.engine_mut().apply_action(Action::SetDataWindowX {
            axis: x_top,
            window: Some(DataWindow {
                min: -200.0,
                max: 200.0,
            }),
        });

        canvas.engine_mut().apply_action(Action::SetDataWindowY {
            axis: y_left,
            window: Some(DataWindow {
                min: -15.0,
                max: 15.0,
            }),
        });
        canvas.engine_mut().apply_action(Action::SetDataWindowY {
            axis: y_right,
            window: Some(DataWindow {
                min: 0.0,
                max: 1600.0,
            }),
        });

        (canvas, router)
    }
}

impl WinitAppDriver for ChartMultiAxisDemoDriver {
    type WindowState = ChartMultiAxisDemoWindowState;

    fn create_window_state(&mut self, app: &mut App, window: AppWindowId) -> Self::WindowState {
        Self::build_ui(app, window)
    }

    fn handle_event(&mut self, context: WinitEventContext<'_, Self::WindowState>, event: &Event) {
        let WinitEventContext {
            app,
            services,
            window,
            state,
            ..
        } = context;

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

        if state.root.is_none() {
            let shared_brush = state.shared_brush.clone();
            let shared_axis_pointer = state.shared_axis_pointer.clone();
            let shared_domain_windows = state.shared_domain_windows.clone();

            let (top_canvas, top_router) = Self::build_canvas(
                delinea::ids::ChartId::new(1),
                shared_brush.clone(),
                shared_axis_pointer.clone(),
                shared_domain_windows.clone(),
                state.top_output.clone(),
            );
            let (bottom_canvas, bottom_router) = Self::build_canvas(
                delinea::ids::ChartId::new(2),
                shared_brush,
                shared_axis_pointer,
                shared_domain_windows,
                state.bottom_output.clone(),
            );

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
        }

        state.ui.request_semantics_snapshot();
        state.ui.ingest_paint_cache_source(scene);
        scene.clear();

        let mut frame =
            fret_ui::UiFrameCx::new(&mut state.ui, app, services, window, bounds, scale_factor);
        frame.layout_all();
        frame.paint_all(scene);
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
