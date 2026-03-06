use super::prelude::*;

pub(super) fn with_scoped_foreground<H: UiHost, R>(
    cx: &mut PaintCx<'_, H>,
    foreground: Option<Color>,
    f: impl FnOnce(&mut PaintCx<'_, H>) -> R,
) -> R {
    let Some(foreground) = foreground else {
        return f(cx);
    };

    let prev = cx.paint_style;
    let mut next = prev;
    next.foreground = Some(foreground);
    cx.paint_style = next;

    let out = f(cx);
    cx.paint_style = prev;
    out
}

pub(super) fn scrollbar_track_padding_px(track_main_axis: f32, padding: Px) -> f32 {
    padding.0.max(0.0).min(track_main_axis.max(0.0) * 0.5)
}

pub(super) fn scrollbar_thumb_rect(
    track: Rect,
    viewport_h: Px,
    content_h: Px,
    offset_y: Px,
    track_padding: Px,
) -> Option<Rect> {
    let viewport_h = Px(viewport_h.0.max(0.0));
    let content_h = Px(content_h.0.max(0.0));
    let max_offset = Px((content_h.0 - viewport_h.0).max(0.0));
    if max_offset.0 <= 0.0 || track.size.height.0 <= 0.0 {
        return None;
    }

    let track_h = track.size.height.0;
    let pad = scrollbar_track_padding_px(track_h, track_padding);
    let inner_track_h = (track_h - pad * 2.0).max(0.0);
    if inner_track_h <= 0.0 {
        return None;
    }
    // Minimum of 18 matches macOS minimum and Radix ScrollArea defaults.
    let min_thumb_h = 18.0f32.min(inner_track_h);
    let ratio = (viewport_h.0 / content_h.0).clamp(0.0, 1.0);
    let thumb_h = (inner_track_h * ratio).max(min_thumb_h).min(inner_track_h);
    let max_thumb_y = (inner_track_h - thumb_h).max(0.0);

    let t = (offset_y.0.max(0.0).min(max_offset.0)) / max_offset.0;
    let y = track.origin.y.0 + pad + max_thumb_y * t;

    Some(Rect::new(
        fret_core::Point::new(track.origin.x, Px(y)),
        Size::new(track.size.width, Px(thumb_h)),
    ))
}

pub(super) fn scrollbar_thumb_rect_horizontal(
    track: Rect,
    viewport_w: Px,
    content_w: Px,
    offset_x: Px,
    track_padding: Px,
) -> Option<Rect> {
    let viewport_w = Px(viewport_w.0.max(0.0));
    let content_w = Px(content_w.0.max(0.0));
    let max_offset = Px((content_w.0 - viewport_w.0).max(0.0));
    if max_offset.0 <= 0.0 || track.size.width.0 <= 0.0 {
        return None;
    }

    let track_w = track.size.width.0;
    let pad = scrollbar_track_padding_px(track_w, track_padding);
    let inner_track_w = (track_w - pad * 2.0).max(0.0);
    if inner_track_w <= 0.0 {
        return None;
    }
    // Minimum of 18 matches macOS minimum and Radix ScrollArea defaults.
    let min_thumb_w = 18.0f32.min(inner_track_w);
    let ratio = (viewport_w.0 / content_w.0).clamp(0.0, 1.0);
    let thumb_w = (inner_track_w * ratio).max(min_thumb_w).min(inner_track_w);
    let max_thumb_x = (inner_track_w - thumb_w).max(0.0);

    let t = (offset_x.0.max(0.0).min(max_offset.0)) / max_offset.0;
    let x = track.origin.x.0 + pad + max_thumb_x * t;

    Some(Rect::new(
        fret_core::Point::new(Px(x), track.origin.y),
        Size::new(Px(thumb_w), track.size.height),
    ))
}

pub(super) fn paint_children_clipped_if<H: UiHost>(
    cx: &mut PaintCx<'_, H>,
    clip: bool,
    corner_radii: Option<fret_core::Corners>,
) {
    if clip {
        if let Some(radii) = corner_radii
            && (radii.top_left.0 > 0.0
                || radii.top_right.0 > 0.0
                || radii.bottom_right.0 > 0.0
                || radii.bottom_left.0 > 0.0)
        {
            cx.scene.push(SceneOp::PushClipRRect {
                rect: cx.bounds,
                corner_radii: radii,
            });
        } else {
            cx.scene.push(SceneOp::PushClipRect { rect: cx.bounds });
        }
    }

    for &child in cx.children {
        if let Some(bounds) = cx.child_bounds(child) {
            cx.paint(child, bounds);
        } else {
            cx.paint(child, cx.bounds);
        }
    }

    if clip {
        cx.scene.push(SceneOp::PopClip);
    }
}
