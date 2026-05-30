use std::sync::Arc;

use fret_ui::element::{AnyElement, PressableA11y, PressableProps};
use fret_ui::{ElementContext, UiHost};

use super::super::commands::DebugDrawCommand;
use super::super::{DebugDrawOptions, ResponseExt};
use super::behavior;
use super::canvas::{debug_draw_canvas_element, debug_draw_fill_layout};

pub(super) fn debug_draw_pressable_element<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
    commands: Arc<[DebugDrawCommand]>,
    options: DebugDrawOptions,
    response: &mut ResponseExt,
) -> AnyElement {
    let interaction = options.interaction.clone();
    let enabled = interaction.enabled && !crate::imui::imui_is_disabled(cx);
    let mut props = PressableProps {
        layout: options.layout,
        enabled,
        focusable: enabled && interaction.focusable,
        a11y: PressableA11y {
            label: interaction.a11y_label,
            ..Default::default()
        },
        ..Default::default()
    };
    props.focus_ring = None;

    let clip_to_bounds = options.clip_to_bounds;
    cx.pressable_with_id(props, move |cx, state, id| {
        behavior::install_debug_draw_pressable_behavior(cx, id, state, enabled, response);

        vec![debug_draw_canvas_element(
            cx,
            commands,
            debug_draw_fill_layout(),
            clip_to_bounds,
            options.test_id,
        )]
    })
}
