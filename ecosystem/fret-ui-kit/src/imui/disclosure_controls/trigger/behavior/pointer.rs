use std::sync::Arc;

use fret_core::{MouseButton, Point};
use fret_runtime::Model;
use fret_ui::action::{PressablePointerDownResult, PressablePointerUpResult, UiActionHostExt as _};
use fret_ui::{ElementContext, UiHost};

pub(super) fn install_disclosure_trigger_pointer<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
    context_anchor_model: Model<Option<Point>>,
) {
    cx.pressable_on_pointer_down(Arc::new(|_host, _acx, _down| {
        PressablePointerDownResult::Continue
    }));
    cx.pressable_on_pointer_up(Arc::new(move |host, acx, up| {
        if up.is_click && up.button == MouseButton::Right {
            let _ = host.update_model(&context_anchor_model, |value| *value = Some(up.position));
            host.record_transient_event(acx, crate::imui::KEY_SECONDARY_CLICKED);
            host.record_transient_event(acx, crate::imui::KEY_CONTEXT_MENU_REQUESTED);
            host.notify(acx);
            return PressablePointerUpResult::SkipActivate;
        }

        if up.is_click && up.button == MouseButton::Left && up.click_count == 2 {
            host.record_transient_event(acx, crate::imui::KEY_DOUBLE_CLICKED);
            host.notify(acx);
        }

        PressablePointerUpResult::Continue
    }));
}
