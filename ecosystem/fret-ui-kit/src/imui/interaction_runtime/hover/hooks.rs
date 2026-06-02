use fret_ui::{ElementContext, GlobalElementId, UiHost};

use super::{hook_hover_change, hook_timer, read, shared_delay};

pub(in crate::imui) fn install_hover_query_hooks_for_pressable<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
    id: GlobalElementId,
    hovered_raw: bool,
    long_press_signal_model: Option<fret_runtime::Model<super::super::LongPressSignalState>>,
) -> read::HoverQueryDelayRead {
    let shared_delay_model = shared_delay::model_for_window(cx);
    hook_hover_change::install_hover_change_hook(cx, shared_delay_model.clone());
    hook_timer::install_hover_timer_hook(
        cx,
        id,
        shared_delay_model.clone(),
        long_press_signal_model,
    );

    read::read_hover_query_delay(cx, id, hovered_raw, &shared_delay_model)
}
