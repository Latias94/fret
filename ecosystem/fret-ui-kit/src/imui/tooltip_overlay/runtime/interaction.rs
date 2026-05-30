use fret_core::{Point, Px, Rect};
use fret_runtime::Model;
use fret_ui::{ElementContext, UiHost};

use crate::declarative::ModelWatchExt;
use crate::declarative::scheduling;
use crate::imui::TooltipOptions;
use crate::primitives::tooltip as radix_tooltip;

pub(super) fn update_tooltip_runtime_interaction<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
    trigger_hovered_raw: bool,
    trigger_focused: bool,
    anchor_bounds: Option<Rect>,
    floating_bounds: Option<Rect>,
    open: &Model<bool>,
    event_models: &radix_tooltip::TooltipTriggerEventModels,
    last_pointer: Model<Option<Point>>,
    options: &TooltipOptions,
    disable_hoverable_content: bool,
) -> bool {
    let gates = radix_tooltip::tooltip_trigger_update_gates(
        cx,
        trigger_hovered_raw,
        trigger_focused,
        event_models,
    );
    let update = radix_tooltip::tooltip_update_interaction(
        cx,
        gates.trigger_hovered,
        gates.trigger_focused,
        gates.force_close,
        last_pointer,
        anchor_bounds,
        floating_bounds,
        radix_tooltip::TooltipInteractionConfig {
            disable_hoverable_content,
            open_delay_ticks_override: options.open_delay_frames_override.map(u64::from),
            close_delay_ticks_override: options.close_delay_frames_override.map(u64::from),
            safe_hover_buffer: Px(5.0),
        },
    );
    scheduling::set_continuous_frames(cx, update.wants_continuous_ticks);

    let open_now = cx.watch_model(open).layout().copied().unwrap_or(false);
    if open_now != update.open {
        let _ = cx
            .app
            .models_mut()
            .update(open, |value| *value = update.open);
    }

    update.open
}
