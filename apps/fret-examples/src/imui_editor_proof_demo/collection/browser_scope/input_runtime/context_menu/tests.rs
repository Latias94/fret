use super::*;

use fret_core::{Modifiers, PointerId, PointerType, Px};
use fret_runtime::TickId;

fn pointer_up(
    button: MouseButton,
    is_click: bool,
    position: Point,
    position_window: Option<Point>,
    down_hit_pressable_target: Option<fret_ui::GlobalElementId>,
    down_hit_pressable_target_in_descendant_subtree: bool,
) -> PointerUpCx {
    PointerUpCx {
        pointer_id: PointerId(0),
        position,
        position_local: position,
        position_window,
        tick_id: TickId(0),
        pixels_per_point: 1.0,
        velocity_window: None,
        button,
        modifiers: Modifiers::default(),
        is_click,
        click_count: 1,
        pointer_type: PointerType::Mouse,
        down_hit_pressable_target,
        down_hit_pressable_target_in_descendant_subtree,
    }
}

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
