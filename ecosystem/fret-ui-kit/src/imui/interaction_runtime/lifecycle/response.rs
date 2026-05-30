use fret_ui::{ElementContext, GlobalElementId, UiHost};

use crate::imui::{KEY_ACTIVATED, KEY_DEACTIVATED, KEY_DEACTIVATED_AFTER_EDIT, ResponseExt};

#[derive(Debug, Default, Clone, Copy)]
struct ResponseLifecycleFrameState {
    was_active: bool,
    edited_during_active: bool,
}

pub(in super::super::super) fn populate_response_lifecycle_transients<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
    id: GlobalElementId,
    response: &mut ResponseExt,
) {
    response.set_activated(cx.take_transient_for(id, KEY_ACTIVATED));
    response.set_deactivated(cx.take_transient_for(id, KEY_DEACTIVATED));
    response.set_deactivated_after_edit(cx.take_transient_for(id, KEY_DEACTIVATED_AFTER_EDIT));
}

pub(in super::super::super) fn populate_response_lifecycle_from_active_state<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
    id: GlobalElementId,
    active_now: bool,
    edited_now: bool,
    response: &mut ResponseExt,
) {
    response.set_edited(edited_now);
    let (activated, deactivated, deactivated_after_edit) =
        cx.state_for(id, ResponseLifecycleFrameState::default, |st| {
            let activated = active_now && !st.was_active;
            let edited_during_session = if active_now || st.was_active {
                st.edited_during_active || edited_now
            } else {
                false
            };
            let deactivated = !active_now && st.was_active;
            let deactivated_after_edit = deactivated && edited_during_session;

            st.was_active = active_now;
            st.edited_during_active = if active_now {
                edited_during_session
            } else {
                false
            };

            (activated, deactivated, deactivated_after_edit)
        });

    response.merge_activated(activated);
    response.merge_deactivated(deactivated);
    response.merge_deactivated_after_edit(deactivated_after_edit);
}
