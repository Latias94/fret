use std::sync::Arc;

use fret_ui::action::ActivateReason;
use fret_ui::element::{
    AnyElement, CanvasCachePolicy, CanvasProps, LayoutStyle, Length, PressableA11y, PressableProps,
    SizeStyle,
};
use fret_ui::{ElementContext, UiHost};

use super::commands::DebugDrawCommand;
use super::paint::paint_debug_draw_commands;
use super::{DebugDrawOptions, ResponseExt};

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

fn debug_draw_pressable_element<H: UiHost>(
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
        let behavior = crate::imui::item_behavior::install_pressable_item_behavior_with_options(
            cx,
            id,
            crate::imui::item_behavior::PressableItemBehaviorOptions {
                report_pointer_click: true,
            },
        );
        let lifecycle_model_for_activate = behavior.lifecycle_model.clone();

        cx.pressable_on_activate(crate::on_activate(move |host, acx, reason| {
            if reason == ActivateReason::Keyboard {
                crate::imui::mark_lifecycle_instant_if_inactive(
                    host,
                    acx,
                    &lifecycle_model_for_activate,
                    false,
                );
            }
            host.record_transient_event(acx, crate::imui::KEY_CLICKED);
            host.notify(acx);
        }));

        let clicked = cx.take_transient_for(id, crate::imui::KEY_CLICKED);
        crate::imui::item_behavior::populate_pressable_item_response(
            cx,
            id,
            state,
            &behavior,
            crate::imui::item_behavior::PressableItemResponseInput {
                enabled,
                clicked,
                changed: false,
                lifecycle_edited: false,
            },
            response,
        );

        vec![debug_draw_canvas_element(
            cx,
            commands,
            debug_draw_fill_layout(),
            clip_to_bounds,
            options.test_id,
        )]
    })
}

fn debug_draw_fill_layout() -> LayoutStyle {
    LayoutStyle {
        size: SizeStyle {
            width: Length::Fill,
            height: Length::Fill,
            ..Default::default()
        },
        ..Default::default()
    }
}

fn debug_draw_canvas_element<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
    commands: Arc<[DebugDrawCommand]>,
    layout: LayoutStyle,
    clip_to_bounds: bool,
    test_id: Option<Arc<str>>,
) -> AnyElement {
    let mut props = CanvasProps {
        layout,
        cache_policy: CanvasCachePolicy::smooth_default(),
        prepaint: false,
    };
    props.cache_policy.shared_text.keep_frames = 30;
    props.cache_policy.path.keep_frames = 30;

    let mut element = cx.canvas(props, move |painter| {
        if clip_to_bounds {
            let bounds = painter.bounds();
            painter.with_clip_rect(bounds, |painter| {
                paint_debug_draw_commands(painter, &commands)
            });
        } else {
            paint_debug_draw_commands(painter, &commands);
        }
    });
    if let Some(test_id) = test_id {
        element = element.test_id(test_id);
    }
    element
}
