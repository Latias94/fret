use super::super::*;

pub(in super::super) fn plain_tab_focus_command(modifiers: fret_core::Modifiers) -> &'static str {
    if modifiers.shift {
        CMD_NODE_GRAPH_FOCUS_PREV
    } else {
        CMD_NODE_GRAPH_FOCUS_NEXT
    }
}

pub(in super::super) fn arrow_nudge_command(
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

pub(in super::super) fn is_arrow_key(key: fret_core::KeyCode) -> bool {
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
