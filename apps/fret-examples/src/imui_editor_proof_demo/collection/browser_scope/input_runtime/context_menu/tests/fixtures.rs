use fret_core::{Modifiers, MouseButton, Point, PointerId, PointerType};
use fret_runtime::TickId;
use fret_ui::action::PointerUpCx;

pub(super) fn pointer_up(
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
        down_hit_is_text_input: false,
        down_hit_is_pressable: false,
    }
}
