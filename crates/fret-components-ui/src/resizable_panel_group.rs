use fret_core::{Axis, Event, Px, Rect, Size};
use fret_runtime::Model;
use fret_ui::{EventCx, LayoutCx, PaintCx, ResizableSplit, UiHost, Widget as UiWidget};

/// A shadcn-inspired resizable panel group primitive.
///
/// This is a thin component-level wrapper around `fret_ui::ResizableSplit` that provides a stable
/// naming surface (`ResizablePanelGroup`) for application UI kits.
///
/// Notes:
/// - Current implementation supports exactly two child panels (like `ResizableSplit`).
/// - Future work can generalize this into a multi-panel group without forcing app-level rewrites.
pub struct ResizablePanelGroup {
    inner: ResizableSplit,
    last_bounds: Rect,
}

impl ResizablePanelGroup {
    pub fn new(axis: Axis, fraction: Model<f32>) -> Self {
        Self {
            inner: ResizableSplit::new(axis, fraction),
            last_bounds: Rect::default(),
        }
    }

    pub fn horizontal(fraction: Model<f32>) -> Self {
        Self::new(Axis::Horizontal, fraction)
    }

    pub fn vertical(fraction: Model<f32>) -> Self {
        Self::new(Axis::Vertical, fraction)
    }

    pub fn with_min_px(mut self, min_px: Px) -> Self {
        self.inner = self.inner.with_min_px(min_px);
        self
    }

    pub fn with_hit_thickness(mut self, thickness: Px) -> Self {
        self.inner = self.inner.with_hit_thickness(thickness);
        self
    }

    pub fn with_paint_device_px(mut self, px: f32) -> Self {
        self.inner = self.inner.with_paint_device_px(px);
        self
    }
}

impl<H: UiHost> UiWidget<H> for ResizablePanelGroup {
    fn event(&mut self, cx: &mut EventCx<'_, H>, event: &Event) {
        self.last_bounds = cx.bounds;
        <ResizableSplit as UiWidget<H>>::event(&mut self.inner, cx, event);
    }

    fn layout(&mut self, cx: &mut LayoutCx<'_, H>) -> Size {
        self.last_bounds = cx.bounds;
        <ResizableSplit as UiWidget<H>>::layout(&mut self.inner, cx)
    }

    fn paint(&mut self, cx: &mut PaintCx<'_, H>) {
        self.last_bounds = cx.bounds;
        <ResizableSplit as UiWidget<H>>::paint(&mut self.inner, cx);
    }
}

