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
