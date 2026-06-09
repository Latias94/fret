use fret_ui::element::PressableState;
use fret_ui::{ElementContext, GlobalElementId, UiHost};

use crate::imui::{KEY_CLICKED, ResponseExt};

pub(super) fn populate_disclosure_trigger_core_response<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
    trigger_id: GlobalElementId,
    state: &PressableState,
    trigger_response: &mut ResponseExt,
) {
    trigger_response.set_core_hovered(state.hovered);
    trigger_response.set_core_pressed(state.pressed);
    trigger_response.set_core_focused(state.focused);
    trigger_response.set_nav_highlighted(
        state.focused && fret_ui::focus_visible::is_focus_visible(cx.app, Some(cx.window)),
    );
    trigger_response.set_id(Some(trigger_id));
    trigger_response.set_core_clicked(cx.take_transient_for(trigger_id, KEY_CLICKED));
    trigger_response.set_core_rect(cx.last_bounds_for_element(trigger_id));
}
