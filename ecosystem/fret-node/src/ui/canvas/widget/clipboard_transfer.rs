use super::*;

pub(super) fn copy_selection_to_clipboard<H: UiHost, M: NodeGraphCanvasMiddleware>(
    canvas: &mut NodeGraphCanvasWith<M>,
    host: &mut H,
    window: Option<AppWindowId>,
    selected_nodes: &[GraphNodeId],
    selected_groups: &[crate::core::GroupId],
) {
    let Some(window) = window else {
        return;
    };

    if selected_nodes.is_empty() && selected_groups.is_empty() {
        return;
    }

    let text = canvas
        .graph
        .read_ref(host, |graph| {
            let fragment = GraphFragment::from_selection(
                graph,
                selected_nodes.to_vec(),
                selected_groups.to_vec(),
            );
            fragment.to_clipboard_text().unwrap_or_default()
        })
        .ok()
        .unwrap_or_default();
    if text.is_empty() {
        return;
    }

    let token = host.next_clipboard_token();
    host.push_effect(Effect::ClipboardWriteText {
        window,
        token,
        text,
    });
}

pub(super) fn request_paste_at_canvas<H: UiHost, M: NodeGraphCanvasMiddleware>(
    canvas: &mut NodeGraphCanvasWith<M>,
    host: &mut H,
    window: Option<AppWindowId>,
    at: CanvasPoint,
) {
    let Some(window) = window else {
        return;
    };

    let token = host.next_clipboard_token();
    canvas.interaction.pending_paste = Some(PendingPaste { token, at });
    host.push_effect(Effect::ClipboardReadText { window, token });
}
