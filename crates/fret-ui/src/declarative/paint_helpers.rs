use super::prelude::*;

pub(super) fn scrollbar_thumb_rect(
    track: Rect,
    viewport_h: Px,
    content_h: Px,
    offset_y: Px,
) -> Option<Rect> {
    let viewport_h = Px(viewport_h.0.max(0.0));
    let content_h = Px(content_h.0.max(0.0));
    let max_offset = Px((content_h.0 - viewport_h.0).max(0.0));
    if max_offset.0 <= 0.0 || track.size.height.0 <= 0.0 {
        return None;
    }

    let track_h = track.size.height.0;
    // Minimum of 18 matches macOS minimum and Radix ScrollArea defaults.
    let min_thumb_h = 18.0f32.min(track_h);
    let ratio = (viewport_h.0 / content_h.0).clamp(0.0, 1.0);
    let thumb_h = (track_h * ratio).max(min_thumb_h).min(track_h);
    let max_thumb_y = (track_h - thumb_h).max(0.0);

    let t = (offset_y.0.max(0.0).min(max_offset.0)) / max_offset.0;
    let y = track.origin.y.0 + max_thumb_y * t;

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
) -> Option<Rect> {
    let viewport_w = Px(viewport_w.0.max(0.0));
    let content_w = Px(content_w.0.max(0.0));
    let max_offset = Px((content_w.0 - viewport_w.0).max(0.0));
    if max_offset.0 <= 0.0 || track.size.width.0 <= 0.0 {
        return None;
    }

    let track_w = track.size.width.0;
    // Minimum of 18 matches macOS minimum and Radix ScrollArea defaults.
    let min_thumb_w = 18.0f32.min(track_w);
    let ratio = (viewport_w.0 / content_w.0).clamp(0.0, 1.0);
    let thumb_w = (track_w * ratio).max(min_thumb_w).min(track_w);
    let max_thumb_x = (track_w - thumb_w).max(0.0);

    let t = (offset_x.0.max(0.0).min(max_offset.0)) / max_offset.0;
    let x = track.origin.x.0 + max_thumb_x * t;

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
