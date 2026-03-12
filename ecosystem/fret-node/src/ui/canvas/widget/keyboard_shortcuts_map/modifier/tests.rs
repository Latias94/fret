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
