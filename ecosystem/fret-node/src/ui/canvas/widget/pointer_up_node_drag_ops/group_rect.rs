use std::collections::BTreeMap;

use fret_ui::UiHost;

use crate::core::GroupId;
use crate::ops::GraphOp;

use super::{NodeGraphCanvasMiddleware, NodeGraphCanvasWith};

pub(super) fn group_rect_ops<H: UiHost, M: NodeGraphCanvasMiddleware>(
    canvas: &mut NodeGraphCanvasWith<M>,
    host: &mut H,
    group_overrides: &BTreeMap<GroupId, crate::core::CanvasRect>,
) -> Vec<GraphOp> {
    canvas
        .graph
        .read_ref(host, |graph| {
            group_overrides
                .iter()
                .filter_map(|(&id, &to)| {
                    let from = graph.groups.get(&id).map(|g| g.rect)?;
                    (from != to).then_some(GraphOp::SetGroupRect { id, from, to })
                })
                .collect()
        })
        .ok()
        .unwrap_or_default()
}
