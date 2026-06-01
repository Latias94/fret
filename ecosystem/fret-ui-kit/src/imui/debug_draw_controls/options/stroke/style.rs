use fret_core::{PathStyle, StrokeCapV1, StrokeJoinV1, StrokeStyle, StrokeStyleV2};

use super::DebugDrawStrokeStyle;

pub(in crate::imui::debug_draw_controls) fn debug_draw_stroke_is_visible(
    style: DebugDrawStrokeStyle,
) -> bool {
    style.width.0 > 0.0
}

pub(in crate::imui::debug_draw_controls) fn debug_draw_stroke_path_style(
    style: DebugDrawStrokeStyle,
) -> PathStyle {
    if style.join == StrokeJoinV1::Miter
        && style.cap == StrokeCapV1::Butt
        && style.miter_limit == 4.0
        && style.dash.is_none()
    {
        PathStyle::Stroke(StrokeStyle { width: style.width })
    } else {
        PathStyle::StrokeV2(StrokeStyleV2 {
            width: style.width,
            join: style.join,
            cap: style.cap,
            miter_limit: style.miter_limit,
            dash: style.dash,
        })
    }
}
