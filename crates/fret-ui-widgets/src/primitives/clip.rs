use fret_core::{SceneOp, Size};
use fret_ui::{LayoutCx, PaintCx, UiHost, Widget};

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

impl<H: UiHost> Widget<H> for Clip {
    fn layout(&mut self, cx: &mut LayoutCx<'_, H>) -> Size {
        for &child in cx.children {
            let _ = cx.layout_in(child, cx.bounds);
        }
        cx.available
    }

    fn paint(&mut self, cx: &mut PaintCx<'_, H>) {
        cx.scene.push(SceneOp::PushClipRect { rect: cx.bounds });
        for &child in cx.children {
            if let Some(bounds) = cx.child_bounds(child) {
                cx.paint(child, bounds);
            } else {
                cx.paint(child, cx.bounds);
            }
        }
        cx.scene.push(SceneOp::PopClip);
    }
}
