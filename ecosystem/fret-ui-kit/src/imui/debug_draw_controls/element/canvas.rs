use std::sync::Arc;

use fret_ui::element::{
    AnyElement, CanvasCachePolicy, CanvasProps, LayoutStyle, Length, SizeStyle,
};
use fret_ui::{ElementContext, UiHost};

use super::super::commands::DebugDrawCommand;
use super::super::paint::paint_debug_draw_commands;

pub(super) fn debug_draw_fill_layout() -> LayoutStyle {
    LayoutStyle {
        size: SizeStyle {
            width: Length::Fill,
            height: Length::Fill,
            ..Default::default()
        },
        ..Default::default()
    }
}

pub(super) fn debug_draw_canvas_element<H: UiHost>(
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
