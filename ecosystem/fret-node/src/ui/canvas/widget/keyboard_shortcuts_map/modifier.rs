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
mod tests {
    use super::*;

    fn modifiers() -> fret_core::Modifiers {
        fret_core::Modifiers::default()
    }

    #[test]
    fn modifier_tab_focus_edge_command_respects_direction_and_a11y_gate() {
        assert_eq!(
            modifier_tab_focus_edge_command(false, fret_core::KeyCode::Tab, modifiers()),
            Some(CMD_NODE_GRAPH_FOCUS_NEXT_EDGE)
        );
        assert_eq!(
            modifier_tab_focus_edge_command(
                false,
                fret_core::KeyCode::Tab,
                fret_core::Modifiers {
                    shift: true,
                    ..modifiers()
                }
            ),
            Some(CMD_NODE_GRAPH_FOCUS_PREV_EDGE)
        );
        assert_eq!(
            modifier_tab_focus_edge_command(true, fret_core::KeyCode::Tab, modifiers()),
            None
        );
    }

    #[test]
    fn modifier_command_routes_edit_and_history_shortcuts() {
        assert_eq!(
            modifier_command(fret_core::KeyCode::KeyA, modifiers()),
            Some("edit.select_all")
        );
        assert_eq!(
            modifier_command(fret_core::KeyCode::KeyZ, modifiers()),
            Some(CMD_NODE_GRAPH_UNDO)
        );
        assert_eq!(
            modifier_command(
                fret_core::KeyCode::KeyZ,
                fret_core::Modifiers {
                    shift: true,
                    ..modifiers()
                }
            ),
            Some(CMD_NODE_GRAPH_REDO)
        );
        assert_eq!(
            modifier_command(fret_core::KeyCode::KeyD, modifiers()),
            Some(CMD_NODE_GRAPH_DUPLICATE)
        );
    }
}
