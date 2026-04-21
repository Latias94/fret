use fret_core::MouseButton;
use fret_ui::action::UiActionHostExt as _;
use fret_ui::{ElementContext, GlobalElementId, UiHost};

#[derive(Debug, Default, Clone, Copy)]
struct ResponseLifecycleFrameState {
    was_active: bool,
    edited_during_active: bool,
}

pub(in super::super) fn mark_lifecycle_activated_on_left_pointer_down<
    H: fret_ui::action::UiActionHost + ?Sized,
>(
    host: &mut H,
    acx: fret_ui::action::ActionCx,
    button: MouseButton,
    lifecycle_model: &fret_runtime::Model<super::ImUiLifecycleSessionState>,
) {
    if button != MouseButton::Left {
        return;
    }
    let mut should_fire = false;
    let _ = host.update_model(lifecycle_model, |st| {
        if !st.active {
            st.active = true;
            st.edited_during_active = false;
            should_fire = true;
        }
    });
    if should_fire {
        host.record_transient_event(acx, super::super::KEY_ACTIVATED);
    }
}

pub(in super::super) fn mark_lifecycle_deactivated_on_left_pointer_up<
    H: fret_ui::action::UiActionHost + ?Sized,
>(
    host: &mut H,
    acx: fret_ui::action::ActionCx,
    button: MouseButton,
    lifecycle_model: &fret_runtime::Model<super::ImUiLifecycleSessionState>,
) {
    if button != MouseButton::Left {
        return;
    }
    let mut should_fire = false;
    let mut after_edit = false;
    let _ = host.update_model(lifecycle_model, |st| {
        if st.active {
            should_fire = true;
            after_edit = st.edited_during_active;
            st.active = false;
            st.edited_during_active = false;
        }
    });
    if should_fire {
        host.record_transient_event(acx, super::super::KEY_DEACTIVATED);
        if after_edit {
            host.record_transient_event(acx, super::super::KEY_DEACTIVATED_AFTER_EDIT);
        }
    }
}

pub(in super::super) fn mark_lifecycle_edit<H: fret_ui::action::UiActionHost + ?Sized>(
    host: &mut H,
    acx: fret_ui::action::ActionCx,
    lifecycle_model: &fret_runtime::Model<super::ImUiLifecycleSessionState>,
) {
    let active = host
        .models_mut()
        .read(lifecycle_model, |st| st.active)
        .ok()
        .unwrap_or(false);

    if active {
        let _ = host.update_model(lifecycle_model, |st| {
            st.edited_during_active = true;
        });
        return;
    }

    host.record_transient_event(acx, super::super::KEY_ACTIVATED);
    host.record_transient_event(acx, super::super::KEY_DEACTIVATED);
    host.record_transient_event(acx, super::super::KEY_DEACTIVATED_AFTER_EDIT);
}

pub(in super::super) fn mark_lifecycle_instant_if_inactive<
    H: fret_ui::action::UiActionHost + ?Sized,
>(
    host: &mut H,
    acx: fret_ui::action::ActionCx,
    lifecycle_model: &fret_runtime::Model<super::ImUiLifecycleSessionState>,
    edited: bool,
) {
    let active = host
        .models_mut()
        .read(lifecycle_model, |st| st.active)
        .ok()
        .unwrap_or(false);
    if active {
        if edited {
            let _ = host.update_model(lifecycle_model, |st| {
                st.edited_during_active = true;
            });
        }
        return;
    }

    host.record_transient_event(acx, super::super::KEY_ACTIVATED);
    host.record_transient_event(acx, super::super::KEY_DEACTIVATED);
    if edited {
        host.record_transient_event(acx, super::super::KEY_DEACTIVATED_AFTER_EDIT);
    }
}

pub(in super::super) fn populate_response_lifecycle_transients<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
    id: GlobalElementId,
    response: &mut super::super::ResponseExt,
) {
    response.activated = cx.take_transient_for(id, super::super::KEY_ACTIVATED);
    response.deactivated = cx.take_transient_for(id, super::super::KEY_DEACTIVATED);
    response.deactivated_after_edit =
        cx.take_transient_for(id, super::super::KEY_DEACTIVATED_AFTER_EDIT);
}

pub(in super::super) fn populate_response_lifecycle_from_active_state<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
    id: GlobalElementId,
    active_now: bool,
    edited_now: bool,
    response: &mut super::super::ResponseExt,
) {
    response.edited = edited_now;
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

    response.activated |= activated;
    response.deactivated |= deactivated;
    response.deactivated_after_edit |= deactivated_after_edit;
}
