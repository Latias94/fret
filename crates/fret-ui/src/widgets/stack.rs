use crate::widget::{LayoutCx, PaintCx, Widget};
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

impl Widget for Stack {
    fn layout(&mut self, cx: &mut LayoutCx<'_>) -> Size {
        for &child in cx.children {
            let _ = cx.layout(child, cx.available);
        }
        cx.available
    }

    fn paint(&mut self, cx: &mut PaintCx<'_>) {
        for &child in cx.children {
            cx.paint(child, cx.bounds);
        }
    }
}

