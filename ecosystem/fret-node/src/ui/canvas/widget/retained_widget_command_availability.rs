use super::*;

pub(super) fn command_availability<H: UiHost, M: NodeGraphCanvasMiddleware>(
    canvas: &NodeGraphCanvasWith<M>,
    cx: &mut CommandAvailabilityCx<'_, H>,
    command: &CommandId,
) -> CommandAvailability {
    if !super::retained_widget_command_availability_gate::should_handle_command(cx) {
        return CommandAvailability::NotHandled;
    }

    match command.as_str() {
        "edit.copy" | CMD_NODE_GRAPH_COPY => copy_availability(canvas, cx),
        "edit.cut" | CMD_NODE_GRAPH_CUT => cut_availability(canvas, cx),
        "edit.paste" | CMD_NODE_GRAPH_PASTE => paste_availability(cx),
        "edit.select_all" | CMD_NODE_GRAPH_SELECT_ALL => select_all_availability(canvas, cx),
        _ => CommandAvailability::NotHandled,
    }
}

fn copy_availability<H: UiHost, M: NodeGraphCanvasMiddleware>(
    canvas: &NodeGraphCanvasWith<M>,
    cx: &mut CommandAvailabilityCx<'_, H>,
) -> CommandAvailability {
    if !super::retained_widget_command_availability_gate::can_write_clipboard(cx) {
        return CommandAvailability::Blocked;
    }

    availability_from(
        super::retained_widget_command_availability_query::has_copyable_selection(canvas, cx),
    )
}

fn cut_availability<H: UiHost, M: NodeGraphCanvasMiddleware>(
    canvas: &NodeGraphCanvasWith<M>,
    cx: &mut CommandAvailabilityCx<'_, H>,
) -> CommandAvailability {
    if !super::retained_widget_command_availability_gate::can_write_clipboard(cx) {
        return CommandAvailability::Blocked;
    }

    availability_from(
        super::retained_widget_command_availability_query::has_any_selection(canvas, cx),
    )
}

fn paste_availability<H: UiHost>(cx: &mut CommandAvailabilityCx<'_, H>) -> CommandAvailability {
    if !super::retained_widget_command_availability_gate::can_paste(cx) {
        return CommandAvailability::Blocked;
    }

    CommandAvailability::Available
}

fn select_all_availability<H: UiHost, M: NodeGraphCanvasMiddleware>(
    canvas: &NodeGraphCanvasWith<M>,
    cx: &mut CommandAvailabilityCx<'_, H>,
) -> CommandAvailability {
    availability_from(
        super::retained_widget_command_availability_query::has_any_content(canvas, cx),
    )
}

fn availability_from(enabled: bool) -> CommandAvailability {
    if enabled {
        CommandAvailability::Available
    } else {
        CommandAvailability::Blocked
    }
}
