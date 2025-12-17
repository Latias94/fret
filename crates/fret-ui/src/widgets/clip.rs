use crate::widget::{LayoutCx, PaintCx, Widget};
use fret_core::{SceneOp, Size};

pub struct Clip;

impl Clip {
    pub fn new() -> Self {
        Self
    }
}

impl Default for Clip {
    fn default() -> Self {
        Self::new()
    }
}

impl Widget for Clip {
    fn layout(&mut self, cx: &mut LayoutCx<'_>) -> Size {
        for &child in cx.children {
            let _ = cx.layout_in(child, cx.bounds);
        }
        cx.available
    }

    fn paint(&mut self, cx: &mut PaintCx<'_>) {
        cx.scene.push(SceneOp::PushClipRect { rect: cx.bounds });
        for &child in cx.children {
            cx.paint(child, cx.bounds);
        }
        cx.scene.push(SceneOp::PopClip);
    }
}
