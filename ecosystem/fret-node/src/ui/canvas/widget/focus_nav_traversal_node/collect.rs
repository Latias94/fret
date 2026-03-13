use std::collections::HashSet;

use crate::ui::canvas::widget::*;

pub(super) fn ordered_selectable_nodes<M: NodeGraphCanvasMiddleware>(
    canvas: &NodeGraphCanvasWith<M>,
    host: &mut impl UiHost,
    snapshot: &ViewSnapshot,
) -> Vec<GraphNodeId> {
    canvas
        .graph
        .read_ref(host, |graph| {
            let mut out: Vec<GraphNodeId> = Vec::new();
            let mut used: HashSet<GraphNodeId> = HashSet::new();

            for id in &snapshot.draw_order {
                if NodeGraphCanvasWith::<M>::node_is_selectable(graph, &snapshot.interaction, *id)
                    && used.insert(*id)
                {
                    out.push(*id);
                }
            }

            let mut rest: Vec<GraphNodeId> = graph
                .nodes
                .keys()
                .copied()
                .filter(|id| {
                    NodeGraphCanvasWith::<M>::node_is_selectable(graph, &snapshot.interaction, *id)
                })
                .filter(|id| used.insert(*id))
                .collect();
            rest.sort_unstable();
            out.extend(rest);
            out
        })
        .ok()
        .unwrap_or_default()
}
