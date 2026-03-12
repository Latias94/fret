use std::sync::Arc;

use crate::ui::canvas::widget::paint_render_data::RenderData;
use crate::ui::canvas::widget::*;

use super::selection::RenderSelections;

#[allow(clippy::too_many_arguments)]
pub(super) fn collect_render_data<M: NodeGraphCanvasMiddleware, H: UiHost>(
    canvas: &NodeGraphCanvasWith<M>,
    host: &H,
    snapshot: &ViewSnapshot,
    geom: Arc<CanvasGeometry>,
    index: Arc<CanvasSpatialDerived>,
    render_cull_rect: Option<Rect>,
    zoom: f32,
    hovered_edge: Option<EdgeId>,
    include_groups: bool,
    include_nodes: bool,
    include_edges: bool,
    presenter: &dyn NodeGraphPresenter,
    selections: RenderSelections,
) -> RenderData {
    let cull = render_cull_rect;
    canvas
        .graph
        .read_ref(host, |graph| {
            let mut out = RenderData::default();

            let geom = geom.as_ref();
            let index = index.as_ref();
            let label_overhead = canvas.node_render_label_overhead();

            if include_groups {
                canvas.collect_group_render_data(
                    graph,
                    snapshot,
                    &selections.selected_groups,
                    cull,
                    &mut out,
                );
            }

            if include_nodes {
                out.metrics.node_total = geom.order.len();
                let (node_candidates, visible_nodes) =
                    canvas.visible_node_ids_for_render(geom, index, cull);
                out.metrics.node_candidates = node_candidates;

                for node in visible_nodes {
                    canvas.append_node_render_data(
                        graph,
                        geom,
                        presenter,
                        &mut out,
                        node,
                        selections.selected_nodes.contains(&node),
                        zoom,
                        label_overhead,
                    );
                }
            }

            if include_edges {
                canvas.collect_edge_render_data(
                    graph,
                    snapshot,
                    geom,
                    index,
                    presenter,
                    &selections.selected_edges,
                    hovered_edge,
                    cull,
                    zoom,
                    &mut out,
                );
            }

            out
        })
        .unwrap_or_default()
}
