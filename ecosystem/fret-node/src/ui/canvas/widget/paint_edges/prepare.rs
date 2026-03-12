#[path = "prepare/batches.rs"]
mod batches;
#[path = "prepare/build.rs"]
mod build;
#[path = "prepare/marker.rs"]
mod marker;

use std::collections::HashMap;

use crate::ui::canvas::widget::paint_render_data::RenderData;
use crate::ui::canvas::widget::*;
use fret_core::scene::DashPatternV1;
use fret_core::scene::PaintBindingV1;

#[derive(Debug, Clone)]
pub(super) struct EdgePaint {
    pub id: EdgeId,
    pub from: Point,
    pub to: Point,
    pub color: Color,
    pub paint: PaintBindingV1,
    pub width: f32,
    pub route: EdgeRouteKind,
    pub dash: Option<DashPatternV1>,
    pub start_marker: Option<crate::ui::presenter::EdgeMarker>,
    pub end_marker: Option<crate::ui::presenter::EdgeMarker>,
    pub selected: bool,
    pub hovered: bool,
}

#[derive(Debug, Default)]
pub(super) struct PreparedEdgePaintBatches {
    pub edges_normal: Vec<EdgePaint>,
    pub edges_selected: Vec<EdgePaint>,
    pub edges_hovered: Vec<EdgePaint>,
    pub edge_insert_marker: Option<(Point, Color)>,
    pub insert_node_drag_marker: Option<(Point, Color)>,
}

impl<M: NodeGraphCanvasMiddleware> NodeGraphCanvasWith<M> {
    pub(super) fn prepare_edge_paint_batches(
        &self,
        snapshot: &ViewSnapshot,
        render: &RenderData,
        custom_paths: &HashMap<EdgeId, crate::ui::edge_types::EdgeCustomPath>,
        zoom: f32,
    ) -> PreparedEdgePaintBatches {
        batches::prepare_edge_paint_batches(self, snapshot, render, custom_paths, zoom)
    }
}
