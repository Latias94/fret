use fret_core::Size;
use fret_ui_app::{App, GenericWidget, LayoutCx, PaintCx};

#[derive(Debug, Clone, Copy)]
pub struct CenteredOverlayLayout {
    pub width: fret_core::Px,
    pub height: fret_core::Px,
}

impl CenteredOverlayLayout {
    pub fn new(width: fret_core::Px, height: fret_core::Px) -> Self {
        Self { width, height }
    }
}

impl GenericWidget<App> for CenteredOverlayLayout {
    fn layout(&mut self, cx: &mut LayoutCx<'_>) -> Size {
        let Some((&backdrop, rest)) = cx.children.split_first() else {
            return cx.available;
        };
        let backdrop_bounds = cx.bounds;
        let _ = cx.layout_in(backdrop, backdrop_bounds);

        if let Some(&panel) = rest.first() {
            let w = self.width.0.min(cx.available.width.0).max(0.0);
            let h = self.height.0.min(cx.available.height.0).max(0.0);

            let mut panel_bounds = cx.bounds;
            panel_bounds.origin.x =
                fret_core::Px(cx.bounds.origin.x.0 + (cx.available.width.0 - w) * 0.5);
            panel_bounds.origin.y =
                fret_core::Px(cx.bounds.origin.y.0 + (cx.available.height.0 - h) * 0.5);
            panel_bounds.size = Size::new(fret_core::Px(w), fret_core::Px(h));
            let _ = cx.layout_in(panel, panel_bounds);
        }
        cx.available
    }

    fn paint(&mut self, cx: &mut PaintCx<'_>) {
        for &child in cx.children {
            if let Some(bounds) = cx.child_bounds(child) {
                cx.paint(child, bounds);
            } else {
                cx.paint(child, cx.bounds);
            }
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub struct CornerOverlayLayout {
    pub width: fret_core::Px,
    pub height: fret_core::Px,
    pub margin: fret_core::Px,
}

impl CornerOverlayLayout {
    pub fn top_left(width: fret_core::Px, height: fret_core::Px) -> Self {
        Self {
            width,
            height,
            margin: fret_core::Px(12.0),
        }
    }

    pub fn with_margin(mut self, margin: fret_core::Px) -> Self {
        self.margin = margin;
        self
    }
}

impl GenericWidget<App> for CornerOverlayLayout {
    fn layout(&mut self, cx: &mut LayoutCx<'_>) -> Size {
        let Some(&child) = cx.children.first() else {
            return cx.available;
        };

        let w = self.width.0.min(cx.available.width.0).max(0.0);
        let h = self.height.0.min(cx.available.height.0).max(0.0);

        let mut bounds = cx.bounds;
        bounds.origin.x = fret_core::Px(cx.bounds.origin.x.0 + self.margin.0);
        bounds.origin.y = fret_core::Px(cx.bounds.origin.y.0 + self.margin.0);
        bounds.size = Size::new(fret_core::Px(w), fret_core::Px(h));

        let _ = cx.layout_in(child, bounds);
        cx.available
    }

    fn paint(&mut self, cx: &mut PaintCx<'_>) {
        for &child in cx.children {
            if let Some(bounds) = cx.child_bounds(child) {
                cx.paint(child, bounds);
            } else {
                cx.paint(child, cx.bounds);
            }
        }
    }
}
