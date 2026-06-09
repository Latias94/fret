use fret_ui::action::{ActionCx, UiPointerActionHost};

pub(super) fn record_disclosure_trigger_double_click(
    host: &mut dyn UiPointerActionHost,
    acx: ActionCx,
) {
    host.record_transient_event(acx, crate::imui::KEY_DOUBLE_CLICKED);
    host.notify(acx);
}
