use super::*;

pub(super) fn apply_paste_text<H: UiHost, M: NodeGraphCanvasMiddleware>(
    canvas: &mut NodeGraphCanvasWith<M>,
    host: &mut H,
    window: Option<AppWindowId>,
    text: &str,
    at: CanvasPoint,
) {
    let Some(fragment) = parse_clipboard_fragment(canvas, host, window, text) else {
        return;
    };
    let Some(offset) = paste_offset_for_fragment(&fragment, at) else {
        return;
    };

    let tx = build_paste_transaction(canvas, host, &fragment, PasteTuning { offset }, None);
    let inserted = inserted_entities(&tx);
    if !canvas.apply_ops_result(host, window, tx.ops) {
        return;
    }

    apply_inserted_selection(canvas, host, inserted);
}

pub(super) fn duplicate_selection<H: UiHost, M: NodeGraphCanvasMiddleware>(
    canvas: &mut NodeGraphCanvasWith<M>,
    host: &mut H,
    window: Option<AppWindowId>,
    selected_nodes: &[GraphNodeId],
    selected_groups: &[crate::core::GroupId],
) {
    if selected_nodes.is_empty() && selected_groups.is_empty() {
        return;
    }

    let fragment = canvas
        .graph
        .read_ref(host, |graph| {
            GraphFragment::from_selection(graph, selected_nodes.to_vec(), selected_groups.to_vec())
        })
        .ok()
        .unwrap_or_default();

    let tx = build_paste_transaction(
        canvas,
        host,
        &fragment,
        PasteTuning {
            offset: CanvasPoint { x: 24.0, y: 24.0 },
        },
        Some("Duplicate".to_string()),
    );
    let inserted = inserted_entities(&tx);
    if !canvas.commit_transaction(host, window, &tx) {
        return;
    }

    apply_inserted_selection(canvas, host, inserted);
}

fn parse_clipboard_fragment<H: UiHost, M: NodeGraphCanvasMiddleware>(
    canvas: &mut NodeGraphCanvasWith<M>,
    host: &mut H,
    window: Option<AppWindowId>,
    text: &str,
) -> Option<GraphFragment> {
    match GraphFragment::from_clipboard_text(text) {
        Ok(fragment) => Some(fragment),
        Err(_) => {
            canvas.show_toast(
                host,
                window,
                DiagnosticSeverity::Info,
                "clipboard does not contain a fret-node fragment",
            );
            None
        }
    }
}

fn paste_offset_for_fragment(fragment: &GraphFragment, at: CanvasPoint) -> Option<CanvasPoint> {
    let mut min_x = f32::INFINITY;
    let mut min_y = f32::INFINITY;
    for node in fragment.nodes.values() {
        min_x = min_x.min(node.pos.x);
        min_y = min_y.min(node.pos.y);
    }
    for group in fragment.groups.values() {
        min_x = min_x.min(group.rect.origin.x);
        min_y = min_y.min(group.rect.origin.y);
    }
    if !min_x.is_finite() || !min_y.is_finite() {
        return None;
    }

    Some(CanvasPoint {
        x: at.x - min_x,
        y: at.y - min_y,
    })
}

fn build_paste_transaction<H: UiHost, M: NodeGraphCanvasMiddleware>(
    canvas: &mut NodeGraphCanvasWith<M>,
    host: &mut H,
    fragment: &GraphFragment,
    tuning: PasteTuning,
    label: Option<String>,
) -> GraphTransaction {
    let remapper = IdRemapper::new(IdRemapSeed::new_random());
    let mut tx = fragment.to_paste_transaction(&remapper, tuning);
    if !fragment.imports.is_empty() {
        canvas
            .graph
            .read_ref(host, |graph| {
                tx.ops.retain(|op| {
                    !matches!(op, GraphOp::AddImport { id, .. } if graph.imports.contains_key(id))
                });
            })
            .ok();
    }
    tx.label = label;
    tx
}

#[derive(Default)]
struct InsertedEntities {
    nodes: Vec<GraphNodeId>,
    groups: Vec<crate::core::GroupId>,
}

fn inserted_entities(tx: &GraphTransaction) -> InsertedEntities {
    InsertedEntities {
        nodes: tx
            .ops
            .iter()
            .filter_map(|op| match op {
                GraphOp::AddNode { id, .. } => Some(*id),
                _ => None,
            })
            .collect(),
        groups: tx
            .ops
            .iter()
            .filter_map(|op| match op {
                GraphOp::AddGroup { id, .. } => Some(*id),
                _ => None,
            })
            .collect(),
    }
}

fn apply_inserted_selection<H: UiHost, M: NodeGraphCanvasMiddleware>(
    canvas: &mut NodeGraphCanvasWith<M>,
    host: &mut H,
    inserted: InsertedEntities,
) {
    if inserted.nodes.is_empty() && inserted.groups.is_empty() {
        return;
    }

    canvas.update_view_state(host, |state| {
        state.selected_edges.clear();
        state.selected_nodes = inserted.nodes.clone();
        state.selected_groups = inserted.groups.clone();
        for id in &inserted.nodes {
            state.draw_order.retain(|x| x != id);
            state.draw_order.push(*id);
        }
        for id in &inserted.groups {
            state.group_draw_order.retain(|x| x != id);
            state.group_draw_order.push(*id);
        }
    });
}
