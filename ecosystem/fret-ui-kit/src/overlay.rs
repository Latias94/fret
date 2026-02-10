use fret_core::{Edges, Point, Px, Rect, Size};
use fret_ui::UiHost;
use fret_ui::elements::GlobalElementId;
use fret_ui::overlay_placement::{
    Align, AnchoredPanelLayout, AnchoredPanelOptions, Side, anchored_panel_bounds_sized, inset_rect,
};
use fret_ui::{ElementContext, Invalidation};

fn prefer_visual_bounds(visual: Option<Rect>, layout: Option<Rect>) -> Option<Rect> {
    visual.or(layout)
}

/// Returns the best available anchor bounds for an element (prefers visual bounds when present).
///
/// This is intended for anchored overlays that must be render-transform aware.
pub fn anchor_bounds_for_element<H: UiHost>(
    cx: &ElementContext<'_, H>,
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

/// Returns an "outer viewport" rect inset by a uniform window margin, observing the committed
/// environment snapshot (ADR 1171).
#[track_caller]
pub fn outer_bounds_with_window_margin_for_environment<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
    invalidation: Invalidation,
    window_margin: Px,
) -> Rect {
    outer_bounds_with_window_margin(cx.environment_viewport_bounds(invalidation), window_margin)
}

/// Wraps a point as a 1x1 rect anchor for placement solvers.
pub fn anchor_rect_from_point(point: Point) -> Rect {
    Rect::new(point, Size::new(Px(1.0), Px(1.0)))
}

pub fn estimated_element_size<H: UiHost>(
    cx: &ElementContext<'_, H>,
    element: GlobalElementId,
    fallback: Size,
) -> Size {
    cx.last_bounds_for_element(element)
        .map(|r| r.size)
        .unwrap_or(fallback)
}

#[allow(clippy::too_many_arguments)]
pub fn anchored_panel_bounds_for_element<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
    trigger: GlobalElementId,
    content: GlobalElementId,
    window_margin: Px,
    side_offset: Px,
    side: Side,
    align: Align,
    fallback_size: Size,
) -> Option<Rect> {
    let anchor = anchor_bounds_for_element(cx, trigger)?;
    let outer =
        outer_bounds_with_window_margin_for_environment(cx, Invalidation::Layout, window_margin);
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

/// Computes an anchored popper layout (rect + optional arrow) with deterministic flip/clamp rules.
pub fn popper_layout_sized(
    outer: Rect,
    anchor: Rect,
    desired: Size,
    side_offset: Px,
    side: Side,
    align: Align,
    options: AnchoredPanelOptions,
) -> AnchoredPanelLayout {
    crate::primitives::popper::popper_layout_sized(
        outer,
        anchor,
        desired,
        side_offset,
        side,
        align,
        options,
    )
}

/// Computes an anchored popper layout for an element, using last-frame layout/visual bounds.
///
/// - `outer` is derived from `cx.bounds` inset by `window_margin`.
/// - `desired` is derived from last-frame content size (falls back to `fallback_size`).
#[allow(clippy::too_many_arguments)]
pub fn popper_layout_for_element<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
    anchor: GlobalElementId,
    content: GlobalElementId,
    window_margin: Px,
    fallback_size: Size,
    side_offset: Px,
    side: Side,
    align: Align,
    options: AnchoredPanelOptions,
) -> Option<AnchoredPanelLayout> {
    let anchor = anchor_bounds_for_element(cx, anchor)?;
    let outer =
        outer_bounds_with_window_margin_for_environment(cx, Invalidation::Layout, window_margin);
    let size = estimated_element_size(cx, content, fallback_size);
    Some(popper_layout_sized(
        outer,
        anchor,
        size,
        side_offset,
        side,
        align,
        options,
    ))
}

#[cfg(test)]
mod tests {
    use super::*;
    use fret_ui::overlay_placement::{ArrowOptions, LayoutDirection, Offset};

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

    #[test]
    fn popper_layout_for_element_returns_arrow_layout_when_configured() {
        let outer = Rect::new(
            Point::new(Px(0.0), Px(0.0)),
            Size::new(Px(200.0), Px(200.0)),
        );
        let anchor = Rect::new(
            Point::new(Px(50.0), Px(60.0)),
            Size::new(Px(40.0), Px(10.0)),
        );
        let desired = Size::new(Px(120.0), Px(80.0));

        let layout = popper_layout_sized(
            outer,
            anchor,
            desired,
            Px(8.0),
            Side::Bottom,
            Align::Center,
            AnchoredPanelOptions {
                direction: LayoutDirection::Ltr,
                offset: Offset::default(),
                shift: Default::default(),
                arrow: Some(ArrowOptions {
                    size: Size::new(Px(12.0), Px(12.0)),
                    padding: Edges::all(Px(8.0)),
                }),
                collision: Default::default(),
                sticky: Default::default(),
            },
        );

        let arrow = layout.arrow.expect("arrow layout");
        assert_eq!(arrow.side, Side::Top);
    }
}
