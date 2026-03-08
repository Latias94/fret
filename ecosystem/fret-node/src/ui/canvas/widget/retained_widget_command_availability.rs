use super::*;

pub(super) fn command_availability<H: UiHost, M: NodeGraphCanvasMiddleware>(
    canvas: &NodeGraphCanvasWith<M>,
    cx: &mut CommandAvailabilityCx<'_, H>,
    command: &CommandId,
) -> CommandAvailability {
    if cx.focus != Some(cx.node) {
        return CommandAvailability::NotHandled;
    }

    let clipboard_read = cx.input_ctx.caps.clipboard.text.read;
    let clipboard_write = cx.input_ctx.caps.clipboard.text.write;
    match command.as_str() {
        "edit.copy" | CMD_NODE_GRAPH_COPY => {
            if !clipboard_write {
                return CommandAvailability::Blocked;
            }

            let has_copyable_selection = canvas
                .view_state
                .read_ref(cx.app, |state| {
                    !state.selected_nodes.is_empty() || !state.selected_groups.is_empty()
                })
                .ok()
                .unwrap_or(false);

            if has_copyable_selection {
                CommandAvailability::Available
            } else {
                CommandAvailability::Blocked
            }
        }
        "edit.cut" | CMD_NODE_GRAPH_CUT => {
            if !clipboard_write {
                return CommandAvailability::Blocked;
            }

            let has_any_selection = canvas
                .view_state
                .read_ref(cx.app, |state| {
                    !state.selected_nodes.is_empty()
                        || !state.selected_edges.is_empty()
                        || !state.selected_groups.is_empty()
                })
                .ok()
                .unwrap_or(false);

            if has_any_selection {
                CommandAvailability::Available
            } else {
                CommandAvailability::Blocked
            }
        }
        "edit.paste" | CMD_NODE_GRAPH_PASTE => {
            if !clipboard_read || cx.window.is_none() {
                return CommandAvailability::Blocked;
            }
            CommandAvailability::Available
        }
        "edit.select_all" | CMD_NODE_GRAPH_SELECT_ALL => {
            let has_any_content = canvas
                .graph
                .read_ref(cx.app, |graph| {
                    !graph.nodes.is_empty() || !graph.groups.is_empty()
                })
                .ok()
                .unwrap_or(false);

            if has_any_content {
                CommandAvailability::Available
            } else {
                CommandAvailability::Blocked
            }
        }
        _ => CommandAvailability::NotHandled,
    }
}
