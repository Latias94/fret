use fret_core::MouseButton;
use fret_ui::action::UiActionHostExt as _;

pub(in crate::imui) fn mark_active_item_on_left_pointer_down(
    host: &mut dyn fret_ui::action::UiPointerActionHost,
    acx: fret_ui::action::ActionCx,
    button: MouseButton,
    active_item_model: &fret_runtime::Model<super::super::ImUiActiveItemState>,
    request_focus: bool,
) {
    if button != MouseButton::Left {
        return;
    }
    if request_focus {
        host.request_focus(acx.target);
    }
    mark_active_item_for_target(host, acx, active_item_model);
    host.notify(acx);
}

pub(in crate::imui) fn clear_active_item_on_left_pointer_up(
    host: &mut dyn fret_ui::action::UiPointerActionHost,
    acx: fret_ui::action::ActionCx,
    button: MouseButton,
    active_item_model: &fret_runtime::Model<super::super::ImUiActiveItemState>,
) {
    if button != MouseButton::Left {
        return;
    }
    clear_active_item_for_target(host, acx, active_item_model);
    host.notify(acx);
}

pub(super) fn mark_active_item_for_target(
    host: &mut dyn fret_ui::action::UiPointerActionHost,
    acx: fret_ui::action::ActionCx,
    active_item_model: &fret_runtime::Model<super::super::ImUiActiveItemState>,
) {
    let _ = host.update_model(active_item_model, |st| {
        st.active = Some(acx.target);
    });
}

pub(super) fn clear_active_item_for_target(
    host: &mut dyn fret_ui::action::UiPointerActionHost,
    acx: fret_ui::action::ActionCx,
    active_item_model: &fret_runtime::Model<super::super::ImUiActiveItemState>,
) {
    let _ = host.update_model(active_item_model, |st| {
        if st.active == Some(acx.target) {
            st.active = None;
        }
    });
}
