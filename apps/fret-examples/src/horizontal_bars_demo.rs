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
use fret_ui::UiTree;

use delinea::data::{Column, DataTable};
use delinea::ids::{AxisId, FieldId, StackId, VisualMapId};
use delinea::{
    AxisKind, AxisPointerTrigger, AxisPointerType, AxisScale, SeriesKind, VisualMapMode,
};
use delinea::{
    ChartSpec, DatasetSpec, FieldSpec, GridSpec, SeriesEncode, SeriesSpec, VisualMapSpec,
};
use fret_chart::retained::ChartCanvas;

struct HorizontalBarsDemoWindowState {
    ui: UiTree<App>,
    root: Option<fret_core::NodeId>,
}

#[derive(Default)]
struct HorizontalBarsDemoDriver;

impl HorizontalBarsDemoDriver {
    fn build_ui(_app: &mut App, window: AppWindowId) -> HorizontalBarsDemoWindowState {
        let mut ui: UiTree<App> = UiTree::new();
        ui.set_window(window);

        HorizontalBarsDemoWindowState { ui, root: None }
    }

    fn build_canvas() -> ChartCanvas {
        let dataset_id = delinea::ids::DatasetId::new(1);
        let grid_id = delinea::ids::GridId::new(1);
        let x_axis = AxisId::new(1);
        let y_axis = AxisId::new(2);

        let stack_id = StackId::new(1);
        let series_a_id = delinea::ids::SeriesId::new(1);
        let series_b_id = delinea::ids::SeriesId::new(2);
        let series_c_id = delinea::ids::SeriesId::new(3);

        let x_a_field = FieldId::new(1);
        let x_b_field = FieldId::new(2);
        let x_c_field = FieldId::new(3);
        let y_cat_field = FieldId::new(4);

        let categories: Vec<String> = (0..12).map(|i| format!("Category {i}")).collect();

        let spec = ChartSpec {
            id: delinea::ids::ChartId::new(1),
            viewport: None,
            datasets: vec![DatasetSpec {
                id: dataset_id,
                fields: vec![
                    FieldSpec {
                        id: x_a_field,
                        column: 0,
                    },
                    FieldSpec {
                        id: x_b_field,
                        column: 1,
                    },
                    FieldSpec {
                        id: x_c_field,
                        column: 2,
                    },
                    FieldSpec {
                        id: y_cat_field,
                        column: 3,
                    },
                ],
                ..Default::default()
            }],
            grids: vec![GridSpec { id: grid_id }],
            axes: vec![
                delinea::AxisSpec {
                    id: x_axis,
                    name: Some("Value".to_string()),
                    kind: AxisKind::X,
                    grid: grid_id,
                    position: None,
                    scale: AxisScale::Value(Default::default()),
                    range: None,
                },
                delinea::AxisSpec {
                    id: y_axis,
                    name: Some("Category".to_string()),
                    kind: AxisKind::Y,
                    grid: grid_id,
                    position: None,
                    scale: AxisScale::Category(delinea::CategoryAxisScale { categories }),
                    range: None,
                },
            ],
            data_zoom_x: vec![],
            data_zoom_y: vec![],
            tooltip: None,
            axis_pointer: Some(delinea::AxisPointerSpec {
                enabled: true,
                trigger: AxisPointerTrigger::Axis,
                pointer_type: AxisPointerType::Shadow,
                label: Default::default(),
                snap: false,
                trigger_distance_px: 12.0,
                throttle_px: 0.75,
            }),
            visual_maps: vec![VisualMapSpec {
                id: VisualMapId::new(1),
                mode: VisualMapMode::Continuous,
                dataset: None,
                series: vec![series_c_id],
                field: x_c_field,
                domain: (-80.0, 80.0),
                initial_range: Some((-20.0, 20.0)),
                initial_piece_mask: None,
                point_radius_mul_range: None,
                stroke_width_range: None,
                opacity_mul_range: Some((0.2, 1.0)),
                buckets: 8,
                out_of_range_opacity: 0.25,
            }],
            series: vec![
                SeriesSpec {
                    id: series_a_id,
                    name: Some("Stack A".to_string()),
                    kind: SeriesKind::Bar,
                    dataset: dataset_id,
                    encode: SeriesEncode {
                        x: x_a_field,
                        y: y_cat_field,
                        y2: None,
                    },
                    x_axis,
                    y_axis,
                    stack: Some(stack_id),
                    stack_strategy: Default::default(),
                    bar_layout: Default::default(),
                    area_baseline: None,
                    lod: None,
                },
                SeriesSpec {
                    id: series_b_id,
                    name: Some("Stack B".to_string()),
                    kind: SeriesKind::Bar,
                    dataset: dataset_id,
                    encode: SeriesEncode {
                        x: x_b_field,
                        y: y_cat_field,
                        y2: None,
                    },
                    x_axis,
                    y_axis,
                    stack: Some(stack_id),
                    stack_strategy: Default::default(),
                    bar_layout: Default::default(),
                    area_baseline: None,
                    lod: None,
                },
                SeriesSpec {
                    id: series_c_id,
                    name: Some("Unstacked".to_string()),
                    kind: SeriesKind::Bar,
                    dataset: dataset_id,
                    encode: SeriesEncode {
                        x: x_c_field,
                        y: y_cat_field,
                        y2: None,
                    },
                    x_axis,
                    y_axis,
                    stack: None,
                    stack_strategy: Default::default(),
                    bar_layout: Default::default(),
                    area_baseline: None,
                    lod: None,
                },
            ],
        };

        let mut canvas = ChartCanvas::new(spec).expect("chart spec should be valid");

        let n = 12usize;
        let mut x_a: Vec<f64> = Vec::with_capacity(n);
        let mut x_b: Vec<f64> = Vec::with_capacity(n);
        let mut x_c: Vec<f64> = Vec::with_capacity(n);
        let mut y_cat: Vec<f64> = Vec::with_capacity(n);

        for i in 0..n {
            let t = i as f64 / (n - 1).max(1) as f64;
            // Value columns.
            x_a.push((t * 7.0).sin() * 40.0 + 60.0);
            x_b.push((t * 5.0).cos() * 30.0 + 40.0);
            x_c.push((t * 3.0).sin() * 55.0 - 20.0);
            // Category ordinal values.
            y_cat.push(i as f64);
        }

        let mut table = DataTable::default();
        table.push_column(Column::F64(x_a));
        table.push_column(Column::F64(x_b));
        table.push_column(Column::F64(x_c));
        table.push_column(Column::F64(y_cat));
        canvas.engine_mut().datasets_mut().insert(dataset_id, table);

        canvas
    }
}

impl WinitAppDriver for HorizontalBarsDemoDriver {
    type WindowState = HorizontalBarsDemoWindowState;

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
            }
            _ => {
                state.ui.dispatch_event(app, services, event);
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

        let root = state.root.get_or_insert_with(|| {
            let canvas = Self::build_canvas();
            let node = ChartCanvas::create_node(&mut state.ui, canvas);
            state.ui.set_root(node);
            node
        });

        state.ui.set_root(*root);
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
        main_window_title: "fret-demo horizontal_bars_demo (delinea + fret-chart)".to_string(),
        ..Default::default()
    }
}

pub fn build_driver() -> impl WinitAppDriver {
    HorizontalBarsDemoDriver::default()
}

#[cfg(not(target_arch = "wasm32"))]
pub fn run() -> anyhow::Result<()> {
    let app = build_app();
    let config = build_runner_config();
    let driver = build_driver();

    run_app(config, app, driver)
        .context("run horizontal_bars_demo app")
        .map_err(anyhow::Error::from)
}

#[cfg(target_arch = "wasm32")]
pub fn run() -> anyhow::Result<()> {
    Ok(())
}
