use crate::ui::canvas::widget::paint_render_data::RenderData;
use crate::ui::canvas::widget::*;

pub(super) fn collect_selected_nodes_render_data<M: NodeGraphCanvasMiddleware, H: UiHost>(
    canvas: &NodeGraphCanvasWith<M>,
    host: &H,
    geom: &CanvasGeometry,
    render_cull_rect: Option<Rect>,
    zoom: f32,
    presenter: &dyn NodeGraphPresenter,
    selected_nodes: Vec<GraphNodeId>,
) -> RenderData {
    let cull = render_cull_rect;
    canvas
        .graph
        .read_ref(host, |graph| {
            let mut out = RenderData::default();
            let label_overhead = canvas.node_render_label_overhead();

            for node in selected_nodes.iter().copied() {
                let Some(node_geom) = geom.nodes.get(&node) else {
                    continue;
                };
                if cull.is_some_and(|c| !rects_intersect(node_geom.rect, c)) {
                    continue;
                }

                canvas.append_node_render_data(
                    graph,
                    geom,
                    presenter,
                    &mut out,
                    node,
                    true,
                    zoom,
                    label_overhead,
                );
            }

            out.nodes.sort_unstable_by(|a, b| {
                let rank_a = geom.node_rank.get(&a.0).copied().unwrap_or(u32::MAX);
                let rank_b = geom.node_rank.get(&b.0).copied().unwrap_or(u32::MAX);
                rank_a.cmp(&rank_b).then_with(|| a.0.cmp(&b.0))
            });

            out
        })
        .unwrap_or_default()
}
