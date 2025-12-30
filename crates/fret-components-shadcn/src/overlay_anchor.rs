use fret_core::Rect;
use fret_ui::{ElementCx, GlobalElementId, UiHost};

fn prefer_visual_bounds(visual: Option<Rect>, layout: Option<Rect>) -> Option<Rect> {
    visual.or(layout)
}

pub(crate) fn anchor_bounds_for_element<H: UiHost>(
    cx: &ElementCx<'_, H>,
    element: GlobalElementId,
) -> Option<Rect> {
    prefer_visual_bounds(
        cx.last_visual_bounds_for_element(element),
        cx.last_bounds_for_element(element),
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use fret_core::{Point, Px, Size};

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
