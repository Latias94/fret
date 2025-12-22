use crate::{
    UiHost,
    widget::{LayoutCx, PaintCx, Widget},
};
use fret_core::Size;

pub struct Stack;

impl Stack {
    pub fn new() -> Self {
        Self
    }
}

impl Default for Stack {
    fn default() -> Self {
        Self::new()
    }
}

impl<H: UiHost> Widget<H> for Stack {
    fn layout(&mut self, cx: &mut LayoutCx<'_, H>) -> Size {
        for &child in cx.children {
            let _ = cx.layout_in(child, cx.bounds);
        }
        cx.available
    }

    fn paint(&mut self, cx: &mut PaintCx<'_, H>) {
        for &child in cx.children {
            if let Some(bounds) = cx.child_bounds(child) {
                cx.paint(child, bounds);
            } else {
                cx.paint(child, cx.bounds);
            }
        }
    }
}
