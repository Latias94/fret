use fret_core::Px;
use fret_interaction::drag::DragThreshold as InteractionDragThreshold;
use fret_ui::{ElementContext, GlobalElementId, UiHost};

mod active_item;
mod long_press_timer;
mod pointer_region;
mod pressable;
mod response;

pub(in super::super) use active_item::{
    clear_active_item_on_left_pointer_up, mark_active_item_on_left_pointer_down,
};
pub(in super::super) use pointer_region::{
    finish_pointer_region_drag, handle_pointer_region_drag_move_with_threshold,
    prepare_pointer_region_drag_on_left_down,
};
pub(in super::super) use pressable::{
    finish_pressable_drag_on_pointer_up, handle_pressable_drag_move_with_threshold,
    prepare_pressable_drag_on_pointer_down,
};
pub(in super::super) use response::populate_pressable_drag_response;

pub(in super::super) fn drag_kind_for_element(
    element: GlobalElementId,
) -> fret_runtime::DragKindId {
    fret_runtime::DragKindId(super::super::DRAG_KIND_MASK | element.0)
}

pub(in super::super) fn drag_threshold_for<H: UiHost>(
    cx: &ElementContext<'_, H>,
) -> InteractionDragThreshold {
    let theme = fret_ui::Theme::global(&*cx.app);
    let px = theme
        .metric_by_key(crate::theme_tokens::metric::COMPONENT_IMUI_DRAG_THRESHOLD_PX)
        .unwrap_or(Px(super::super::DEFAULT_DRAG_THRESHOLD_PX));
    InteractionDragThreshold::new(px)
}
