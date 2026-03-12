use super::super::*;

pub(in super::super) fn modifier_command(
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

pub(in super::super) fn modifier_tab_focus_edge_command(
    disable_keyboard_a11y: bool,
    key: fret_core::KeyCode,
    modifiers: fret_core::Modifiers,
) -> Option<&'static str> {
    if disable_keyboard_a11y || key != fret_core::KeyCode::Tab {
        return None;
    }

    Some(if modifiers.shift {
        CMD_NODE_GRAPH_FOCUS_PREV_EDGE
    } else {
        CMD_NODE_GRAPH_FOCUS_NEXT_EDGE
    })
}

#[cfg(test)]
mod tests;
