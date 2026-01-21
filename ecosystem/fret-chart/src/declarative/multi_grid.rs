use std::collections::BTreeMap;

use delinea::data::DataTable;
use delinea::ids::{ChartId, DatasetId, GridId};
use delinea::{ChartEngine, ChartSpec};
use fret_runtime::Model;
use fret_ui::element::{AnyElement, ColumnProps};
use fret_ui::{ElementContext, Invalidation, UiHost};

use crate::declarative::{ChartCanvasPanelProps, chart_canvas_panel};
use crate::multi_grid::split_chart_spec_by_grid;

#[derive(Clone)]
pub struct ChartCanvasMultiGridPanelProps {
    pub panel: ChartCanvasPanelProps,
    pub datasets: Vec<(DatasetId, DataTable)>,
    pub column: ColumnProps,
}

impl ChartCanvasMultiGridPanelProps {
    pub fn new(panel: ChartCanvasPanelProps, datasets: Vec<(DatasetId, DataTable)>) -> Self {
        Self {
            panel,
            datasets,
            column: ColumnProps::default(),
        }
    }
}

#[derive(Default)]
struct MultiGridEngineState {
    by_grid: BTreeMap<GridId, (ChartId, Model<ChartEngine>)>,
}

fn build_engine_for_chart_id<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
    chart_id: ChartId,
    mut spec: ChartSpec,
    datasets: &[(DatasetId, DataTable)],
) -> Model<ChartEngine> {
    spec.id = chart_id;
    spec.axis_pointer.get_or_insert_with(Default::default);

    let mut engine = ChartEngine::new(spec).expect("chart spec should be valid");
    for (dataset_id, table) in datasets {
        engine.datasets_mut().insert(*dataset_id, table.clone());
    }

    cx.app.models_mut().insert(engine)
}

fn ensure_grid_engine<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
    grid: GridId,
    chart_id: ChartId,
    spec: ChartSpec,
    datasets: &[(DatasetId, DataTable)],
) -> Model<ChartEngine> {
    let existing: Option<(ChartId, Model<ChartEngine>)> = cx
        .with_state(MultiGridEngineState::default, |st| {
            st.by_grid.get(&grid).cloned()
        });
    if let Some((existing_id, engine)) = existing
        && existing_id == chart_id
    {
        return engine;
    }

    let engine = build_engine_for_chart_id(cx, chart_id, spec, datasets);
    cx.with_state(MultiGridEngineState::default, |st| {
        st.by_grid.insert(grid, (chart_id, engine.clone()));
    });
    engine
}

#[track_caller]
pub fn chart_canvas_multi_grid_panel<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
    mut props: ChartCanvasMultiGridPanelProps,
) -> AnyElement {
    let split = split_chart_spec_by_grid(&props.panel.spec)
        .expect("chart spec should be splittable (single-grid fallback)");
    if split.len() <= 1 {
        return chart_canvas_panel(cx, props.panel);
    }

    let mut engines: Vec<(GridId, Model<ChartEngine>)> = Vec::with_capacity(split.len());
    for entry in &split {
        let engine = ensure_grid_engine(
            cx,
            entry.grid,
            entry.spec.id,
            entry.spec.clone(),
            &props.datasets,
        );
        engines.push((entry.grid, engine));
    }

    for (_, engine) in engines.iter() {
        cx.observe_model(engine, Invalidation::Paint);
    }

    // P0: simple vertical stack. Layout policy is intentionally minimal; ECharts-like per-grid
    // positioning is part of the long-term per-grid viewport plan.
    props.column.layout.size.width = fret_ui::element::Length::Fill;
    props.column.layout.size.height = fret_ui::element::Length::Fill;

    cx.column(props.column, |cx| {
        split
            .into_iter()
            .map(|entry| {
                let engine = engines
                    .iter()
                    .find(|(grid, _)| *grid == entry.grid)
                    .map(|(_, engine)| engine.clone());

                let mut panel = props.panel.clone();
                panel.spec = entry.spec;
                panel.engine = engine;
                chart_canvas_panel(cx, panel)
            })
            .collect::<Vec<_>>()
    })
}
