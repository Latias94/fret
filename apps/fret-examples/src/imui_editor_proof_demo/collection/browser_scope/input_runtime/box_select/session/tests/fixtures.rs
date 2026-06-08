use std::sync::Arc;

use fret_core::{
    Modifiers, MouseButton, MouseButtons, Point, PointerCancelReason, PointerId, PointerType, Px,
};
use fret_runtime::TickId;
use fret_ui::action::{PointerCancelCx, PointerDownCx, PointerMoveCx, PointerUpCx};

use crate::imui_editor_proof_demo::collection::box_select::ProofCollectionBoxSelectSession;

pub(super) fn point(x: f32, y: f32) -> Point {
    Point::new(Px(x), Px(y))
}

pub(super) fn pointer_down(
    button: MouseButton,
    position: Point,
    hit_is_pressable: bool,
    modifiers: Modifiers,
) -> PointerDownCx {
    PointerDownCx {
        pointer_id: PointerId(7),
        position,
        position_local: position,
        position_window: Some(position),
        tick_id: TickId(0),
        pixels_per_point: 1.0,
        button,
        modifiers,
        click_count: 1,
        pointer_type: PointerType::Mouse,
        hit_is_text_input: false,
        hit_is_pressable,
        hit_pressable_target: None,
        hit_pressable_target_in_descendant_subtree: false,
    }
}

pub(super) fn pointer_move(pointer_id: PointerId, position: Point, left: bool) -> PointerMoveCx {
    PointerMoveCx {
        pointer_id,
        position,
        position_local: position,
        position_window: Some(position),
        tick_id: TickId(0),
        pixels_per_point: 1.0,
        velocity_window: None,
        buttons: MouseButtons {
            left,
            ..Default::default()
        },
        modifiers: Modifiers::default(),
        pointer_type: PointerType::Mouse,
    }
}

pub(super) fn pointer_up(pointer_id: PointerId, position: Point) -> PointerUpCx {
    PointerUpCx {
        pointer_id,
        position,
        position_local: position,
        position_window: Some(position),
        tick_id: TickId(0),
        pixels_per_point: 1.0,
        velocity_window: None,
        button: MouseButton::Left,
        modifiers: Modifiers::default(),
        is_click: false,
        click_count: 1,
        pointer_type: PointerType::Mouse,
        down_hit_pressable_target: None,
        down_hit_pressable_target_in_descendant_subtree: false,
    }
}

pub(super) fn pointer_cancel(pointer_id: PointerId) -> PointerCancelCx {
    PointerCancelCx {
        pointer_id,
        position: None,
        position_local: None,
        position_window: None,
        tick_id: TickId(0),
        pixels_per_point: 1.0,
        buttons: MouseButtons::default(),
        modifiers: Modifiers::default(),
        pointer_type: PointerType::Mouse,
        reason: PointerCancelReason::LeftWindow,
    }
}

pub(super) fn session(pointer_id: PointerId) -> ProofCollectionBoxSelectSession {
    ProofCollectionBoxSelectSession {
        pointer_id,
        origin_local: point(0.0, 0.0),
        current_local: point(0.0, 0.0),
        baseline_selected: vec![Arc::from("stone-albedo")],
        append_mode: false,
        threshold_met: false,
    }
}
