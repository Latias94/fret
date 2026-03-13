use super::super::*;

pub(super) fn apply_measured_sizes<M: NodeGraphCanvasMiddleware>(
    canvas: &mut NodeGraphCanvasWith<M>,
    measured: Vec<(GraphNodeId, (f32, f32))>,
) {
    let keep: std::collections::BTreeSet<GraphNodeId> = measured.iter().map(|(n, _)| *n).collect();

    let _ = canvas
        .auto_measured
        .update_if_changed(|node_sizes, _anchors| {
            let mut changed = false;

            node_sizes.retain(|id, _| {
                let ok = keep.contains(id);
                if !ok {
                    changed = true;
                }
                ok
            });

            for (node, size) in &measured {
                let needs = match node_sizes.get(node) {
                    Some(old) => (old.0 - size.0).abs() > 0.1 || (old.1 - size.1).abs() > 0.1,
                    None => true,
                };
                if needs {
                    node_sizes.insert(*node, *size);
                    changed = true;
                }
            }

            changed
        });
}
