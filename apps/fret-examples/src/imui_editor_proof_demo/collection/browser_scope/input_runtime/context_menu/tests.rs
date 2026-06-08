use super::*;

use fret_core::Px;

mod fixtures;

use fixtures::pointer_up;

#[test]
fn context_menu_anchor_prefers_window_position() {
    let local = Point::new(Px(10.0), Px(20.0));
    let window = Point::new(Px(30.0), Px(40.0));
    let up = pointer_up(MouseButton::Right, true, local, Some(window), None, false);

    assert_eq!(
        proof_collection_browser_scope_context_menu_anchor_from_up(&up),
        Some(window)
    );
}

#[test]
fn context_menu_anchor_falls_back_to_pointer_position() {
    let position = Point::new(Px(10.0), Px(20.0));
    let up = pointer_up(MouseButton::Right, true, position, None, None, false);

    assert_eq!(
        proof_collection_browser_scope_context_menu_anchor_from_up(&up),
        Some(position)
    );
}

#[test]
fn context_menu_anchor_ignores_non_right_or_non_click_up() {
    let position = Point::new(Px(10.0), Px(20.0));

    assert_eq!(
        proof_collection_browser_scope_context_menu_anchor_from_up(&pointer_up(
            MouseButton::Left,
            true,
            position,
            Some(position),
            None,
            false,
        )),
        None
    );
    assert_eq!(
        proof_collection_browser_scope_context_menu_anchor_from_up(&pointer_up(
            MouseButton::Right,
            false,
            position,
            Some(position),
            None,
            false,
        )),
        None
    );
}

#[test]
fn context_menu_anchor_ignores_direct_pressable_clicks() {
    let position = Point::new(Px(10.0), Px(20.0));
    let up = pointer_up(
        MouseButton::Right,
        true,
        position,
        Some(position),
        Some(fret_ui::GlobalElementId(7)),
        false,
    );

    assert_eq!(
        proof_collection_browser_scope_context_menu_anchor_from_up(&up),
        None
    );
}

#[test]
fn context_menu_anchor_ignores_pressable_descendant_clicks() {
    let position = Point::new(Px(10.0), Px(20.0));
    let up = pointer_up(
        MouseButton::Right,
        true,
        position,
        Some(position),
        None,
        true,
    );

    assert_eq!(
        proof_collection_browser_scope_context_menu_anchor_from_up(&up),
        None
    );
}
