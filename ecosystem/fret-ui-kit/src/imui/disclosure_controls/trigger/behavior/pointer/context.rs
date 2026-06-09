use fret_core::Point;
use fret_runtime::Model;
use fret_ui::action::{ActionCx, PointerUpCx, UiActionHostExt as _, UiPointerActionHost};

pub(super) fn record_disclosure_trigger_context_menu(
    host: &mut dyn UiPointerActionHost,
    acx: ActionCx,
    context_anchor_model: &Model<Option<Point>>,
    up: PointerUpCx,
) {
    let _ = host.update_model(context_anchor_model, |value| *value = Some(up.position));
    host.record_transient_event(acx, crate::imui::KEY_SECONDARY_CLICKED);
    host.record_transient_event(acx, crate::imui::KEY_CONTEXT_MENU_REQUESTED);
    host.notify(acx);
}
