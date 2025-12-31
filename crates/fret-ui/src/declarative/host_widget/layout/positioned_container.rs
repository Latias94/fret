use super::super::ElementHostWidget;
use crate::declarative::frame::layout_style_for_node;
use crate::declarative::layout_helpers::{
    clamp_to_constraints, layout_positioned_child, positioned_layout_style,
};
use crate::declarative::prelude::*;

impl ElementHostWidget {
    pub(super) fn layout_positioned_container_impl<H: UiHost>(
        &mut self,
        cx: &mut LayoutCx<'_, H>,
        window: AppWindowId,
        layout: LayoutStyle,
    ) -> Size {
        // Probe within the available height budget so measurement passes do not observe an
        // artificially "infinite" viewport (important for scroll/virtualized children).
        let probe_bounds = Rect::new(cx.bounds.origin, cx.available);
        let mut max_child = Size::new(Px(0.0), Px(0.0));
        for &child in cx.children {
            let child_style = layout_style_for_node(cx.app, window, child);
            if child_style.position == crate::element::PositionStyle::Absolute {
                continue;
            }
            let child_size = cx.layout_in(child, probe_bounds);
            max_child.width = Px(max_child.width.0.max(child_size.width.0));
            max_child.height = Px(max_child.height.0.max(child_size.height.0));
        }

        let desired = clamp_to_constraints(max_child, layout, cx.available);
        let base = Rect::new(cx.bounds.origin, desired);
        for &child in cx.children {
            let child_style = layout_style_for_node(cx.app, window, child);
            layout_positioned_child(cx, child, base, positioned_layout_style(child_style));
        }
        desired
    }
}
