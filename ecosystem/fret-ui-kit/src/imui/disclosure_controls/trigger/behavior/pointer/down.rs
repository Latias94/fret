use std::sync::Arc;

use fret_ui::action::PressablePointerDownResult;
use fret_ui::{ElementContext, UiHost};

pub(super) fn install_disclosure_trigger_pointer_down<H: UiHost>(cx: &mut ElementContext<'_, H>) {
    cx.pressable_on_pointer_down(Arc::new(|_host, _acx, _down| {
        PressablePointerDownResult::Continue
    }));
}
