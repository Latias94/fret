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

    pub(super) fn layout_hover_region_impl<H: UiHost>(
        &mut self,
        cx: &mut LayoutCx<'_, H>,
        window: AppWindowId,
        layout: LayoutStyle,
    ) -> Size {
        // Unlike a general positioned container, a hover region should track hover across its
        // children even if they are absolutely positioned. That implies the hover region's bounds
        // must include absolute children (common in overlay triggers).
        //
        // We conservatively account for absolute insets:
        // - If `left`/`top` is set, treat it as an offset into the hover region.
        // - If `right`/`bottom` is set without `left`/`top`, require enough size to place the
        //   child without going negative (`right + child_size`).
        //
        // This keeps the hover region's hit-test bounds stable without forcing it to fill the
        // viewport.
        let probe_bounds = Rect::new(cx.bounds.origin, cx.available);
        let mut max_child = Size::new(Px(0.0), Px(0.0));

        for &child in cx.children {
            let child_style = layout_style_for_node(cx.app, window, child);
            let child_size = cx.layout_in(child, probe_bounds);

            let required = if child_style.position == crate::element::PositionStyle::Absolute {
                let left = child_style.inset.left.map(|v| v.0);
                let right = child_style.inset.right.map(|v| v.0);
                let top = child_style.inset.top.map(|v| v.0);
                let bottom = child_style.inset.bottom.map(|v| v.0);

                let required_w = match (left, right) {
                    (Some(l), Some(r)) => Px(l + r + child_size.width.0),
                    (Some(l), None) => Px(l + child_size.width.0),
                    (None, Some(r)) => Px(r + child_size.width.0),
                    (None, None) => child_size.width,
                };

                let required_h = match (top, bottom) {
                    (Some(t), Some(b)) => Px(t + b + child_size.height.0),
                    (Some(t), None) => Px(t + child_size.height.0),
                    (None, Some(b)) => Px(b + child_size.height.0),
                    (None, None) => child_size.height,
                };

                Size::new(required_w, required_h)
            } else {
                child_size
            };

            max_child.width = Px(max_child.width.0.max(required.width.0));
            max_child.height = Px(max_child.height.0.max(required.height.0));
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
