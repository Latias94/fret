use super::*;

pub(super) fn modifier_command(
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

pub(super) fn modifier_tab_focus_edge_command(
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

pub(super) fn plain_tab_focus_command(modifiers: fret_core::Modifiers) -> &'static str {
    if modifiers.shift {
        CMD_NODE_GRAPH_FOCUS_PREV
    } else {
        CMD_NODE_GRAPH_FOCUS_NEXT
    }
}

pub(super) fn arrow_nudge_command(
    key: fret_core::KeyCode,
    modifiers: fret_core::Modifiers,
) -> Option<&'static str> {
    match (key, modifiers.shift) {
        (fret_core::KeyCode::ArrowLeft, false) => Some(CMD_NODE_GRAPH_NUDGE_LEFT),
        (fret_core::KeyCode::ArrowRight, false) => Some(CMD_NODE_GRAPH_NUDGE_RIGHT),
        (fret_core::KeyCode::ArrowUp, false) => Some(CMD_NODE_GRAPH_NUDGE_UP),
        (fret_core::KeyCode::ArrowDown, false) => Some(CMD_NODE_GRAPH_NUDGE_DOWN),
        (fret_core::KeyCode::ArrowLeft, true) => Some(CMD_NODE_GRAPH_NUDGE_LEFT_FAST),
        (fret_core::KeyCode::ArrowRight, true) => Some(CMD_NODE_GRAPH_NUDGE_RIGHT_FAST),
        (fret_core::KeyCode::ArrowUp, true) => Some(CMD_NODE_GRAPH_NUDGE_UP_FAST),
        (fret_core::KeyCode::ArrowDown, true) => Some(CMD_NODE_GRAPH_NUDGE_DOWN_FAST),
        _ => None,
    }
}

pub(super) fn is_arrow_key(key: fret_core::KeyCode) -> bool {
    matches!(
        key,
        fret_core::KeyCode::ArrowLeft
            | fret_core::KeyCode::ArrowRight
            | fret_core::KeyCode::ArrowUp
            | fret_core::KeyCode::ArrowDown
    )
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

    #[test]
    fn plain_tab_and_arrow_maps_follow_shift_variant() {
        assert_eq!(
            plain_tab_focus_command(modifiers()),
            CMD_NODE_GRAPH_FOCUS_NEXT
        );
        assert_eq!(
            plain_tab_focus_command(fret_core::Modifiers {
                shift: true,
                ..modifiers()
            }),
            CMD_NODE_GRAPH_FOCUS_PREV
        );
        assert_eq!(
            arrow_nudge_command(fret_core::KeyCode::ArrowLeft, modifiers()),
            Some(CMD_NODE_GRAPH_NUDGE_LEFT)
        );
        assert_eq!(
            arrow_nudge_command(
                fret_core::KeyCode::ArrowDown,
                fret_core::Modifiers {
                    shift: true,
                    ..modifiers()
                }
            ),
            Some(CMD_NODE_GRAPH_NUDGE_DOWN_FAST)
        );
    }

    #[test]
    fn is_arrow_key_only_accepts_arrow_family() {
        assert!(is_arrow_key(fret_core::KeyCode::ArrowUp));
        assert!(!is_arrow_key(fret_core::KeyCode::KeyA));
    }
}
