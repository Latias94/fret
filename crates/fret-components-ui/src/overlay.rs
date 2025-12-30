use fret_core::{Edges, Point, Px, Rect, Size};
use fret_ui::ElementCx;
use fret_ui::UiHost;
use fret_ui::elements::GlobalElementId;
use fret_ui::overlay_placement::{Align, Side, anchored_panel_bounds_sized, inset_rect};

fn prefer_visual_bounds(visual: Option<Rect>, layout: Option<Rect>) -> Option<Rect> {
    visual.or(layout)
}

/// Returns the best available anchor bounds for an element (prefers visual bounds when present).
///
/// This is intended for anchored overlays that must be render-transform aware.
pub fn anchor_bounds_for_element<H: UiHost>(
    cx: &ElementCx<'_, H>,
    element: GlobalElementId,
) -> Option<Rect> {
    prefer_visual_bounds(
        cx.last_visual_bounds_for_element(element),
        cx.last_bounds_for_element(element),
    )
}

/// Returns an "outer viewport" rect inset by a uniform window margin.
pub fn outer_bounds_with_window_margin(window_bounds: Rect, window_margin: Px) -> Rect {
    inset_rect(window_bounds, Edges::all(window_margin))
}

/// Wraps a point as a 1x1 rect anchor for placement solvers.
pub fn anchor_rect_from_point(point: Point) -> Rect {
    Rect::new(point, Size::new(Px(1.0), Px(1.0)))
}

pub fn estimated_element_size<H: UiHost>(
    cx: &ElementCx<'_, H>,
    element: GlobalElementId,
    fallback: Size,
) -> Size {
    cx.last_bounds_for_element(element)
        .map(|r| r.size)
        .unwrap_or(fallback)
}

pub fn anchored_panel_bounds_for_element<H: UiHost>(
    cx: &ElementCx<'_, H>,
    trigger: GlobalElementId,
    content: GlobalElementId,
    window_margin: Px,
    side_offset: Px,
    side: Side,
    align: Align,
    fallback_size: Size,
) -> Option<Rect> {
    let anchor = anchor_bounds_for_element(cx, trigger)?;
    let outer = outer_bounds_with_window_margin(cx.bounds, window_margin);
    let size = estimated_element_size(cx, content, fallback_size);
    Some(anchored_panel_bounds_sized(
        outer,
        anchor,
        size,
        side_offset,
        side,
        align,
    ))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn prefer_visual_bounds_prefers_visual() {
        let visual = Rect::new(Point::new(Px(10.0), Px(20.0)), Size::new(Px(1.0), Px(2.0)));
        let layout = Rect::new(Point::new(Px(30.0), Px(40.0)), Size::new(Px(3.0), Px(4.0)));
        assert_eq!(
            prefer_visual_bounds(Some(visual), Some(layout)),
            Some(visual)
        );
    }

    #[test]
    fn prefer_visual_bounds_falls_back_to_layout() {
        let layout = Rect::new(Point::new(Px(30.0), Px(40.0)), Size::new(Px(3.0), Px(4.0)));
        assert_eq!(prefer_visual_bounds(None, Some(layout)), Some(layout));
    }

    #[test]
    fn prefer_visual_bounds_none_when_missing() {
        assert_eq!(prefer_visual_bounds(None, None), None);
    }
}
