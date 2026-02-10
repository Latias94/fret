use super::*;

pub(crate) fn collect_subtree_nodes(ui: &UiTree<App>, root: NodeId, out: &mut Vec<NodeId>) {
    out.push(root);
    for child in ui.children(root) {
        collect_subtree_nodes(ui, child, out);
    }
}

pub(crate) fn find_node_with_bounds_close(
    ui: &UiTree<App>,
    root: NodeId,
    expected: WebRect,
    tol: f32,
) -> Option<(NodeId, Rect)> {
    let mut nodes = Vec::new();
    collect_subtree_nodes(ui, root, &mut nodes);

    for id in nodes {
        let Some(bounds) = ui.debug_node_bounds(id) else {
            continue;
        };
        let close = (bounds.origin.x.0 - expected.x).abs() <= tol
            && (bounds.origin.y.0 - expected.y).abs() <= tol
            && (bounds.size.width.0 - expected.w).abs() <= tol
            && (bounds.size.height.0 - expected.h).abs() <= tol;
        if close {
            return Some((id, bounds));
        }
    }
    None
}

pub(crate) fn find_node_with_size_close(
    ui: &UiTree<App>,
    root: NodeId,
    expected_w: f32,
    expected_h: f32,
    tol: f32,
) -> Option<Rect> {
    let mut nodes = Vec::new();
    collect_subtree_nodes(ui, root, &mut nodes);

    let mut best: Option<Rect> = None;
    let mut best_score = f32::INFINITY;
    let mut best_area = f32::INFINITY;

    for id in nodes {
        let Some(bounds) = ui.debug_node_bounds(id) else {
            continue;
        };
        let dw = (bounds.size.width.0 - expected_w).abs();
        let dh = (bounds.size.height.0 - expected_h).abs();
        if dw > tol || dh > tol {
            continue;
        }

        let score = dw + dh;
        let area = bounds.size.width.0 * bounds.size.height.0;
        if score < best_score || (score == best_score && area < best_area) {
            best = Some(bounds);
            best_score = score;
            best_area = area;
        }
    }

    best
}
