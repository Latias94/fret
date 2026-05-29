use fret_core::MouseButton;
use fret_ui::action::UiActionHostExt as _;

pub(in crate::imui) fn mark_lifecycle_activated_on_left_pointer_down<
    H: fret_ui::action::UiActionHost + ?Sized,
>(
    host: &mut H,
    acx: fret_ui::action::ActionCx,
    button: MouseButton,
    lifecycle_model: &fret_runtime::Model<super::super::ImUiLifecycleSessionState>,
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
        host.record_transient_event(acx, crate::imui::KEY_ACTIVATED);
    }
}

pub(in crate::imui) fn mark_lifecycle_deactivated_on_left_pointer_up<
    H: fret_ui::action::UiActionHost + ?Sized,
>(
    host: &mut H,
    acx: fret_ui::action::ActionCx,
    button: MouseButton,
    lifecycle_model: &fret_runtime::Model<super::super::ImUiLifecycleSessionState>,
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
        host.record_transient_event(acx, crate::imui::KEY_DEACTIVATED);
        if after_edit {
            host.record_transient_event(acx, crate::imui::KEY_DEACTIVATED_AFTER_EDIT);
        }
    }
}
