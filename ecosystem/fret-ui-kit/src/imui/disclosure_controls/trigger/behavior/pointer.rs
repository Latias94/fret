use fret_core::{MouseButton, Point};
use fret_runtime::Model;
use fret_ui::action::PressablePointerUpResult;
use fret_ui::{ElementContext, UiHost};

mod context;
mod double_click;
mod down;

pub(super) fn install_disclosure_trigger_pointer<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
    context_anchor_model: Model<Option<Point>>,
) {
    down::install_disclosure_trigger_pointer_down(cx);
    cx.pressable_on_pointer_up(std::sync::Arc::new(move |host, acx, up| {
        if up.is_click && up.button == MouseButton::Right {
            context::record_disclosure_trigger_context_menu(host, acx, &context_anchor_model, up);
            return PressablePointerUpResult::SkipActivate;
        }

        if up.is_click && up.button == MouseButton::Left && up.click_count == 2 {
            double_click::record_disclosure_trigger_double_click(host, acx);
        }

        PressablePointerUpResult::Continue
    }));
}
