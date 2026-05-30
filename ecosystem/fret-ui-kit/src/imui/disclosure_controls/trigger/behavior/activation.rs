use fret_runtime::Model;
use fret_ui::{ElementContext, UiHost};

pub(super) fn install_disclosure_trigger_activation<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
    open_model: Model<bool>,
    has_children: bool,
) {
    cx.pressable_on_activate(crate::on_activate(move |host, action_cx, _reason| {
        host.record_transient_event(action_cx, crate::imui::KEY_CLICKED);
        if has_children {
            let _ = host
                .models_mut()
                .update(&open_model, |value| *value = !*value);
        }
        host.notify(action_cx);
    }));
}
