use super::*;

pub(super) fn handle_modifier_shortcuts<H: UiHost>(
    cx: &mut EventCx<'_, H>,
    snapshot: &ViewSnapshot,
    key: fret_core::KeyCode,
    modifiers: fret_core::Modifiers,
) -> bool {
    if !modifiers.ctrl && !modifiers.meta {
        return false;
    }

    if !snapshot.interaction.disable_keyboard_a11y && key == fret_core::KeyCode::Tab {
        let command = if modifiers.shift {
            CMD_NODE_GRAPH_FOCUS_PREV_EDGE
        } else {
            CMD_NODE_GRAPH_FOCUS_NEXT_EDGE
        };
        super::keyboard_shortcuts::dispatch_command(cx, command);
        return true;
    }

    let Some(command) = modifier_command(key, modifiers) else {
        return false;
    };
    super::keyboard_shortcuts::dispatch_command(cx, command);
    true
}

pub(super) fn handle_tab_navigation<H: UiHost, M: NodeGraphCanvasMiddleware>(
    canvas: &mut NodeGraphCanvasWith<M>,
    cx: &mut EventCx<'_, H>,
    snapshot: &ViewSnapshot,
    key: fret_core::KeyCode,
    modifiers: fret_core::Modifiers,
) -> bool {
    if snapshot.interaction.disable_keyboard_a11y
        || key != fret_core::KeyCode::Tab
        || modifiers.ctrl
        || modifiers.meta
        || modifiers.alt
        || modifiers.alt_gr
    {
        return false;
    }

    if canvas.interaction.searcher.is_some() || canvas.interaction.context_menu.is_some() {
        return true;
    }

    let command = if modifiers.shift {
        CMD_NODE_GRAPH_FOCUS_PREV
    } else {
        CMD_NODE_GRAPH_FOCUS_NEXT
    };
    super::keyboard_shortcuts::dispatch_command(cx, command);
    true
}

pub(super) fn handle_arrow_nudging<H: UiHost>(
    cx: &mut EventCx<'_, H>,
    snapshot: &ViewSnapshot,
    key: fret_core::KeyCode,
    modifiers: fret_core::Modifiers,
) -> bool {
    if !is_arrow_key(key) || modifiers.ctrl || modifiers.meta || modifiers.alt || modifiers.alt_gr {
        return false;
    }

    if snapshot.interaction.disable_keyboard_a11y {
        return true;
    }

    if snapshot.selected_nodes.is_empty() && snapshot.selected_groups.is_empty() {
        return true;
    }

    let command = match (key, modifiers.shift) {
        (fret_core::KeyCode::ArrowLeft, false) => CMD_NODE_GRAPH_NUDGE_LEFT,
        (fret_core::KeyCode::ArrowRight, false) => CMD_NODE_GRAPH_NUDGE_RIGHT,
        (fret_core::KeyCode::ArrowUp, false) => CMD_NODE_GRAPH_NUDGE_UP,
        (fret_core::KeyCode::ArrowDown, false) => CMD_NODE_GRAPH_NUDGE_DOWN,
        (fret_core::KeyCode::ArrowLeft, true) => CMD_NODE_GRAPH_NUDGE_LEFT_FAST,
        (fret_core::KeyCode::ArrowRight, true) => CMD_NODE_GRAPH_NUDGE_RIGHT_FAST,
        (fret_core::KeyCode::ArrowUp, true) => CMD_NODE_GRAPH_NUDGE_UP_FAST,
        (fret_core::KeyCode::ArrowDown, true) => CMD_NODE_GRAPH_NUDGE_DOWN_FAST,
        _ => return true,
    };
    super::keyboard_shortcuts::dispatch_command(cx, command);
    true
}

pub(super) fn handle_delete_shortcut<H: UiHost>(
    cx: &mut EventCx<'_, H>,
    snapshot: &ViewSnapshot,
    key: fret_core::KeyCode,
) -> bool {
    if !snapshot.interaction.delete_key.matches(key) {
        return false;
    }

    super::keyboard_shortcuts::dispatch_command(cx, CMD_NODE_GRAPH_DELETE_SELECTION);
    true
}

fn modifier_command(
    key: fret_core::KeyCode,
    modifiers: fret_core::Modifiers,
) -> Option<&'static str> {
    match key {
        fret_core::KeyCode::KeyA => Some("edit.select_all"),
        fret_core::KeyCode::KeyZ => Some(if modifiers.shift {
            CMD_NODE_GRAPH_REDO
        } else {
            CMD_NODE_GRAPH_UNDO
        }),
        fret_core::KeyCode::KeyY => Some(CMD_NODE_GRAPH_REDO),
        fret_core::KeyCode::KeyC => Some("edit.copy"),
        fret_core::KeyCode::KeyX => Some("edit.cut"),
        fret_core::KeyCode::KeyV => Some("edit.paste"),
        fret_core::KeyCode::KeyD => Some(CMD_NODE_GRAPH_DUPLICATE),
        _ => None,
    }
}

fn is_arrow_key(key: fret_core::KeyCode) -> bool {
    matches!(
        key,
        fret_core::KeyCode::ArrowLeft
            | fret_core::KeyCode::ArrowRight
            | fret_core::KeyCode::ArrowUp
            | fret_core::KeyCode::ArrowDown
    )
}
