use fret_ui::action::UiActionHostExt as _;

pub(in crate::imui) fn mark_lifecycle_edit<H: fret_ui::action::UiActionHost + ?Sized>(
    host: &mut H,
    acx: fret_ui::action::ActionCx,
    lifecycle_model: &fret_runtime::Model<super::super::ImUiLifecycleSessionState>,
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

    host.record_transient_event(acx, crate::imui::KEY_ACTIVATED);
    host.record_transient_event(acx, crate::imui::KEY_DEACTIVATED);
    host.record_transient_event(acx, crate::imui::KEY_DEACTIVATED_AFTER_EDIT);
}
