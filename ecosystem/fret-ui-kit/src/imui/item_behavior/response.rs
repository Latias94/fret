use fret_ui::element::PressableState;
use fret_ui::{ElementContext, GlobalElementId, UiHost};

use super::{PressableItemBehavior, PressableItemResponseInput};

pub(in crate::imui) fn populate_pressable_item_response<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
    id: GlobalElementId,
    state: PressableState,
    behavior: &PressableItemBehavior,
    input: PressableItemResponseInput,
    response: &mut super::super::ResponseExt,
) {
    response.set_secondary_clicked(cx.take_transient_for(id, super::super::KEY_SECONDARY_CLICKED));
    response.set_double_clicked(cx.take_transient_for(id, super::super::KEY_DOUBLE_CLICKED));
    response.set_long_pressed(cx.take_transient_for(id, super::super::KEY_LONG_PRESSED));
    response.set_press_holding(
        cx.read_model(
            &behavior.long_press_signal_model,
            fret_ui::Invalidation::Paint,
            |_app, value| value.holding,
        )
        .unwrap_or(false),
    );
    response.set_context_menu_requested(
        cx.take_transient_for(id, super::super::KEY_CONTEXT_MENU_REQUESTED),
    );
    response.set_context_menu_anchor(
        cx.read_model(
            &behavior.context_anchor_model,
            fret_ui::Invalidation::Paint,
            |_app, v| *v,
        )
        .unwrap_or(None),
    );
    response.set_pointer_clicked(cx.take_transient_for(id, super::super::KEY_POINTER_CLICKED));
    if response.pointer_clicked()
        && let Some(pointer_click_modifiers_model) = behavior.pointer_click_modifiers_model.as_ref()
    {
        response.set_pointer_click_modifiers(
            cx.read_model(
                pointer_click_modifiers_model,
                fret_ui::Invalidation::Paint,
                |_app, modifiers| *modifiers,
            )
            .unwrap_or_default(),
        );
    }
    super::super::populate_pressable_drag_response(cx, id, response);
    let hover_delay = super::super::install_hover_query_hooks_for_pressable(
        cx,
        id,
        state.hovered_raw,
        Some(behavior.long_press_signal_model.clone()),
    );
    super::super::populate_pressable_response(
        cx,
        id,
        state,
        hover_delay,
        &behavior.active_item_model,
        input.clicked,
        input.changed,
        state.pressed,
        input.lifecycle_edited,
        input.enabled,
        response,
    );
}
