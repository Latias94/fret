use std::sync::Arc;

use fret_ui::element::AnyElement;
use fret_ui::{ElementContext, UiHost};

use super::commands::DebugDrawCommand;
use super::{DebugDrawOptions, ResponseExt};
use canvas::debug_draw_canvas_element;
use pressable::debug_draw_pressable_element;

mod behavior;
mod canvas;
mod pressable;

pub(super) fn debug_draw_element<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
    commands: Arc<[DebugDrawCommand]>,
    options: DebugDrawOptions,
    response: &mut ResponseExt,
) -> AnyElement {
    if options.interaction.enabled {
        return debug_draw_pressable_element(cx, commands, options, response);
    }

    debug_draw_canvas_element(
        cx,
        commands,
        options.layout,
        options.clip_to_bounds,
        options.test_id,
    )
}
