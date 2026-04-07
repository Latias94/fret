use fret_core::{Rect, Size};
use fret_ui::layout_constraints::{AvailableSpace, LayoutConstraints, LayoutSize};
use fret_ui::{UiHost, retained_bridge::*};

use super::NodeGraphToolbarSize;
use super::layout_hidden_child_and_release_focus;

pub(super) fn resolve_toolbar_child_size<H: UiHost>(
    cx: &mut LayoutCx<'_, H>,
    size: NodeGraphToolbarSize,
    child: fret_core::NodeId,
) -> Size {
    match size {
        NodeGraphToolbarSize::Fixed(size) => size,
        NodeGraphToolbarSize::Auto => {
            let avail = cx.bounds.size;
            let constraints = LayoutConstraints::new(
                LayoutSize::new(None, None),
                LayoutSize::new(
                    AvailableSpace::Definite(avail.width),
                    AvailableSpace::Definite(avail.height),
                ),
            );
            cx.measure_in(child, constraints)
        }
    }
}

pub(super) fn hide_toolbar_child<H: UiHost>(
    cx: &mut LayoutCx<'_, H>,
    child: Option<fret_core::NodeId>,
    canvas_node: fret_core::NodeId,
) {
    if let Some(child) = child {
        layout_hidden_child_and_release_focus(cx, child, canvas_node);
    }
}

pub(super) fn layout_toolbar_child<H: UiHost>(
    cx: &mut LayoutCx<'_, H>,
    child: Option<fret_core::NodeId>,
    canvas_node: fret_core::NodeId,
    size: NodeGraphToolbarSize,
    positioned_rect_for: impl FnOnce(Size) -> Rect,
) -> Option<Rect> {
    let child = child?;
    let size = resolve_toolbar_child_size(cx, size, child);
    if size.width.0 <= 0.0 && size.height.0 <= 0.0 {
        hide_toolbar_child(cx, Some(child), canvas_node);
        return None;
    }

    let rect = positioned_rect_for(size);
    cx.layout_in(child, rect);
    Some(rect)
}

pub(super) fn paint_toolbar_children<H: UiHost>(cx: &mut PaintCx<'_, H>) {
    for &child in cx.children {
        if let Some(bounds) = cx.child_bounds(child) {
            cx.paint(child, bounds);
        }
    }
}
