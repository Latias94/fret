//! Node graph editor composition helpers (UI-only).

use fret_core::{Rect, Size};
use fret_ui::{UiHost, retained_bridge::*};

/// Simple container that layers its children, filling the available bounds.
///
/// Convention:
/// - child 0: the canvas
/// - child 1: overlays (hosted outside the canvas render transform)
pub struct NodeGraphEditor;

impl NodeGraphEditor {
    pub fn new() -> Self {
        Self
    }
}

impl Default for NodeGraphEditor {
    fn default() -> Self {
        Self
    }
}

impl<H: UiHost> Widget<H> for NodeGraphEditor {
    fn layout(&mut self, cx: &mut LayoutCx<'_, H>) -> Size {
        for &child in cx.children {
            cx.layout_in(child, cx.bounds);
        }
        cx.bounds.size
    }

    fn paint(&mut self, cx: &mut PaintCx<'_, H>) {
        for &child in cx.children {
            if let Some(bounds) = cx.child_bounds(child) {
                cx.paint(child, bounds);
            }
        }
    }

    fn hit_test(&self, _bounds: Rect, _position: fret_core::Point) -> bool {
        false
    }
}
