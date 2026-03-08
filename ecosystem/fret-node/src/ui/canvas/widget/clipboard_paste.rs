use super::*;

pub(super) fn apply_paste_text<H: UiHost, M: NodeGraphCanvasMiddleware>(
    canvas: &mut NodeGraphCanvasWith<M>,
    host: &mut H,
    window: Option<AppWindowId>,
    text: &str,
    at: CanvasPoint,
) {
    let Some(fragment) =
        super::clipboard_paste_parse::parse_clipboard_fragment(canvas, host, window, text)
    else {
        return;
    };
    let Some(offset) = super::clipboard_paste_parse::paste_offset_for_fragment(&fragment, at)
    else {
        return;
    };

    let tx = super::clipboard_paste_transaction::build_paste_transaction(
        canvas,
        host,
        &fragment,
        PasteTuning { offset },
        None,
    );
    let inserted = super::clipboard_paste_selection::inserted_entities(&tx);
    if !canvas.apply_ops_result(host, window, tx.ops) {
        return;
    }

    super::clipboard_paste_selection::apply_inserted_selection(canvas, host, inserted);
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

    let tx = super::clipboard_paste_transaction::build_paste_transaction(
        canvas,
        host,
        &fragment,
        PasteTuning {
            offset: CanvasPoint { x: 24.0, y: 24.0 },
        },
        Some("Duplicate".to_string()),
    );
    let inserted = super::clipboard_paste_selection::inserted_entities(&tx);
    if !canvas.commit_transaction(host, window, &tx) {
        return;
    }

    super::clipboard_paste_selection::apply_inserted_selection(canvas, host, inserted);
}
