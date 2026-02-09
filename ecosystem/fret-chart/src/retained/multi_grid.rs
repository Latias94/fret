use std::cell::RefCell;
use std::collections::BTreeSet;
use std::rc::Rc;

use delinea::data::DataTable;
use delinea::engine::ChartEngine;
use delinea::engine::model::ModelError;
use delinea::ids::{DatasetId, GridId};
use delinea::spec::ChartSpec;
use fret_core::{NodeId, Px, Rect, Size};
use fret_ui::layout_pass::LayoutPassKind;
use fret_ui::retained_bridge::{LayoutCx, UiTreeRetainedExt as _, Widget};
use fret_ui::{UiHost, UiTree};

use super::ChartCanvas;

#[derive(Debug, Clone, Copy)]
pub struct UniformGrid {
    pub columns: usize,
    pub gap: Px,
}

impl UniformGrid {
    pub fn new(columns: usize) -> Self {
        Self {
            columns: columns.max(1),
            gap: Px(0.0),
        }
    }

    pub fn with_gap(mut self, gap: Px) -> Self {
        self.gap = gap;
        self
    }

    pub fn create_node<H: UiHost>(ui: &mut UiTree<H>, grid: UniformGrid) -> NodeId {
        ui.create_node_retained(grid)
    }
}

impl<H: UiHost> Widget<H> for UniformGrid {
    fn layout(&mut self, cx: &mut LayoutCx<'_, H>) -> Size {
        let child_count = cx.children.len();
        if child_count == 0 {
            return cx.available;
        }

        let columns = self.columns.max(1).min(child_count);
        let rows = child_count.div_ceil(columns);

        let gap = self.gap.0.max(0.0);
        let total_gap_x = gap * (columns.saturating_sub(1) as f32);
        let total_gap_y = gap * (rows.saturating_sub(1) as f32);

        let cell_w = ((cx.available.width.0 - total_gap_x) / columns.max(1) as f32).max(0.0);
        let cell_h = ((cx.available.height.0 - total_gap_y) / rows.max(1) as f32).max(0.0);

        let origin = cx.bounds.origin;
        let is_final = cx.pass_kind == LayoutPassKind::Final;

        for (i, child) in cx.children.iter().copied().enumerate() {
            let col = i % columns;
            let row = i / columns;

            let x = origin.x.0 + (cell_w + gap) * (col as f32);
            let y = origin.y.0 + (cell_h + gap) * (row as f32);

            let rect = Rect::new(
                fret_core::Point::new(Px(x), Px(y)),
                Size::new(Px(cell_w), Px(cell_h)),
            );

            if is_final {
                let _ = cx.layout_viewport_root(child, rect);
            } else {
                let _ = cx.layout_in(child, rect);
            }
        }

        cx.available
    }
}

/// A minimal retained container that lays out all children to fill the same bounds.
///
/// Child order defines paint/input stacking (last child is on top).
#[derive(Debug, Default, Clone, Copy)]
pub struct FillStack;

impl FillStack {
    pub fn create_node<H: UiHost>(ui: &mut UiTree<H>) -> NodeId {
        ui.create_node_retained(Self)
    }
}

impl<H: UiHost> Widget<H> for FillStack {
    fn layout(&mut self, cx: &mut LayoutCx<'_, H>) -> Size {
        let is_final = cx.pass_kind == LayoutPassKind::Final;
        let rect = cx.bounds;
        for child in cx.children.iter().copied() {
            if is_final {
                let _ = cx.layout_viewport_root(child, rect);
            } else {
                let _ = cx.layout_in(child, rect);
            }
        }
        cx.available
    }
}

#[derive(Debug, Clone)]
pub struct MultiGridChartCanvasNodes {
    pub root: NodeId,
    pub canvases: Vec<(GridId, NodeId)>,
}

fn collect_grids(spec: &ChartSpec) -> Vec<GridId> {
    if !spec.grids.is_empty() {
        return spec.grids.iter().map(|g| g.id).collect();
    }

    let mut ids: BTreeSet<GridId> = spec.axes.iter().map(|a| a.grid).collect();
    if ids.is_empty() {
        ids.insert(GridId::new(1));
    }
    ids.into_iter().collect()
}

pub fn create_multi_grid_chart_canvas_nodes<H: UiHost>(
    ui: &mut UiTree<H>,
    spec: ChartSpec,
    datasets: &[(DatasetId, DataTable)],
    layout: UniformGrid,
) -> Result<MultiGridChartCanvasNodes, ModelError> {
    let grids = collect_grids(&spec);
    if grids.len() <= 1 {
        let mut canvas = ChartCanvas::new(spec)?;
        for (dataset_id, table) in datasets {
            canvas
                .engine_mut()
                .datasets_mut()
                .insert(*dataset_id, table.clone());
        }
        let node = ChartCanvas::create_node(ui, canvas);
        let root = UniformGrid::create_node(ui, layout);
        ui.add_child(root, node);
        return Ok(MultiGridChartCanvasNodes {
            root,
            canvases: vec![(grids.get(0).copied().unwrap_or(GridId::new(1)), node)],
        });
    }

    let mut spec = spec;
    spec.axis_pointer.get_or_insert_with(Default::default);
    let engine = Rc::new(RefCell::new(ChartEngine::new(spec)?));
    {
        let mut engine = engine.borrow_mut();
        for (dataset_id, table) in datasets {
            engine.datasets_mut().insert(*dataset_id, table.clone());
        }
    }

    let mut canvases: Vec<(GridId, NodeId)> = Vec::with_capacity(grids.len());
    for grid in grids {
        let canvas = ChartCanvas::new_grid_view(engine.clone(), grid);
        let node = ChartCanvas::create_node(ui, canvas);
        canvases.push((grid, node));
    }

    let grid_root = UniformGrid::create_node(ui, layout);
    for (_, node) in &canvases {
        ui.add_child(grid_root, *node);
    }

    let overlay = ChartCanvas::create_node(ui, ChartCanvas::new_overlay(engine.clone()));

    let root = FillStack::create_node(ui);
    ui.add_child(root, grid_root);
    ui.add_child(root, overlay);

    Ok(MultiGridChartCanvasNodes { root, canvases })
}
