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
        let mut batches = PreparedEdgePaintBatches::default();
        let edge_insert_marker_request = self
            .interaction
            .edge_insert_drag
            .as_ref()
            .map(|drag| (drag.edge, drag.pos));
        let insert_node_drag_preview = self.interaction.insert_node_drag_preview.as_ref();
        let elevate_edges_on_select = snapshot.interaction.elevate_edges_on_select;
        let bezier_steps = usize::from(snapshot.interaction.bezier_hit_test_steps.max(1));

        batches.edge_insert_marker = edge_insert_marker_request.and_then(|(edge_id, pos)| {
            self.resolve_edge_insert_marker(render, custom_paths, bezier_steps, zoom, edge_id, pos)
        });

        batches.insert_node_drag_marker = insert_node_drag_preview.as_ref().map(|preview| {
            if let Some(edge_id) = preview.edge
                && let Some((pos, color)) = self.resolve_edge_insert_marker(
                    render,
                    custom_paths,
                    bezier_steps,
                    zoom,
                    edge_id,
                    preview.pos,
                )
            {
                (pos, color)
            } else {
                (preview.pos, self.style.paint.wire_color_preview)
            }
        });

        for edge in &render.edges {
            let paint = self.build_edge_paint(edge);
            if edge.hovered {
                batches.edges_hovered.push(paint);
            } else if edge.selected && elevate_edges_on_select {
                batches.edges_selected.push(paint);
            } else {
                batches.edges_normal.push(paint);
            }
        }

        batches
    }

    fn build_edge_paint(
        &self,
        edge: &crate::ui::canvas::widget::paint_render_data::EdgeRender,
    ) -> EdgePaint {
        let mut width = self.style.geometry.wire_width * edge.hint.width_mul.max(0.0);
        if edge.selected {
            width *= self.style.paint.wire_width_selected_mul;
        }
        if edge.hovered {
            width *= self.style.paint.wire_width_hover_mul;
        }

        EdgePaint {
            id: edge.id,
            from: edge.from,
            to: edge.to,
            color: edge.color,
            paint: edge.paint,
            width,
            route: edge.hint.route,
            dash: edge.hint.dash,
            start_marker: edge.hint.start_marker.clone(),
            end_marker: edge.hint.end_marker.clone(),
            selected: edge.selected,
            hovered: edge.hovered,
        }
    }

    fn resolve_edge_insert_marker(
        &self,
        render: &RenderData,
        custom_paths: &HashMap<EdgeId, crate::ui::edge_types::EdgeCustomPath>,
        bezier_steps: usize,
        zoom: f32,
        edge_id: EdgeId,
        pos: Point,
    ) -> Option<(Point, Color)> {
        render
            .edges
            .iter()
            .find(|edge| edge.id == edge_id)
            .map(|edge| {
                let point = custom_paths
                    .get(&edge_id)
                    .map(|custom| closest_point_on_path(&custom.commands, bezier_steps, pos))
                    .unwrap_or_else(|| {
                        closest_point_on_edge_route(
                            edge.hint.route,
                            edge.from,
                            edge.to,
                            zoom,
                            bezier_steps,
                            pos,
                        )
                    });
                (point, edge.color)
            })
    }
}
