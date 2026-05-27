use std::sync::Arc;

use fret_core::{PointerType, Px};
use fret_ui::{ElementContext, GlobalElementId, UiHost};

use crate::primitives::tooltip as radix_tooltip;

pub(super) fn install_pointer_move_open_gate_for<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
    trigger: GlobalElementId,
    models: radix_tooltip::TooltipTriggerEventModels,
    pointer_in_transit_buffer: Px,
    last_pointer: fret_runtime::Model<Option<fret_core::Point>>,
) {
    cx.pressable_add_on_pointer_move_for(
        trigger,
        Arc::new(move |host, action_cx, mv| {
            if mv.pointer_type == PointerType::Touch {
                return false;
            }

            let _ = host
                .models_mut()
                .update(&last_pointer, |value| *value = Some(mv.position));

            let geometry = host
                .models_mut()
                .read(&models.pointer_transit_geometry, |value| *value)
                .ok()
                .flatten();
            if let Some((anchor, floating)) = geometry
                && radix_tooltip::tooltip_pointer_in_transit(
                    mv.position,
                    anchor,
                    floating,
                    pointer_in_transit_buffer,
                )
            {
                return false;
            }

            let already = host
                .models_mut()
                .read(&models.has_pointer_move_opened, |value| *value)
                .ok()
                .unwrap_or(false);
            if !already {
                let _ = host
                    .models_mut()
                    .update(&models.has_pointer_move_opened, |value| *value = true);
                host.request_redraw(action_cx.window);
            }

            false
        }),
    );
}
