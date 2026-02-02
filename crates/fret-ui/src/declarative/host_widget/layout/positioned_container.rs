use super::super::ElementHostWidget;
use crate::declarative::frame::layout_style_for_node;
use crate::declarative::layout_helpers::{
    PositionedLayoutStyle, clamp_to_constraints, layout_absolute_child_with_probe_bounds,
    layout_positioned_child, positioned_layout_style,
};
use crate::declarative::prelude::*;
use crate::layout_constraints::{AvailableSpace, LayoutConstraints, LayoutSize};

impl ElementHostWidget {
    pub(super) fn layout_positioned_container_impl<H: UiHost>(
        &mut self,
        cx: &mut LayoutCx<'_, H>,
        window: AppWindowId,
        layout: LayoutStyle,
    ) -> Size {
        // Probe within this container's own constrained size so measurement passes do not observe
        // an artificially "infinite" viewport (important for scroll/virtualized children) and so
        // absolute-positioned children measure against the same size budget used for placement.
        let probe_available = clamp_to_constraints(cx.available, layout, cx.available);
        let probe_bounds = Rect::new(cx.bounds.origin, probe_available);
        let probe_constraints = probe_constraints_for_size(probe_bounds.size);
        let mut child_measure_constraints = probe_constraints;
        if cx.available.width.0 <= 0.0 {
            child_measure_constraints.available.width = AvailableSpace::MaxContent;
        }
        if cx.available.height.0 <= 0.0 {
            child_measure_constraints.available.height = AvailableSpace::MaxContent;
        }
        let mut max_child = Size::new(Px(0.0), Px(0.0));
        let mut absolute_children: Vec<(NodeId, crate::element::InsetStyle)> = Vec::new();
        for &child in cx.children {
            let child_style = layout_style_for_node(cx.app, window, child);
            if child_style.position == crate::element::PositionStyle::Absolute {
                absolute_children.push((child, child_style.inset));
                continue;
            }
            let child_size = cx.measure_in(child, child_measure_constraints);
            max_child.width = Px(max_child.width.0.max(child_size.width.0));
            max_child.height = Px(max_child.height.0.max(child_size.height.0));
        }

        // If the container has only absolute-positioned children, it can collapse to zero during
        // intrinsic sizing probes (e.g. auto-height shells). That breaks hit-testing for
        // overflow-visible overlays (cmdk/listbox style popovers). When we see a zero available
        // size placeholder, include absolute children in the sizing estimate.
        if (cx.available.width.0 <= 0.0 || cx.available.height.0 <= 0.0)
            && (max_child.width.0 <= 0.0 || max_child.height.0 <= 0.0)
            && !absolute_children.is_empty()
        {
            let mut abs_constraints = probe_constraints;
            if cx.available.width.0 <= 0.0 {
                abs_constraints.available.width = AvailableSpace::MaxContent;
            }
            if cx.available.height.0 <= 0.0 {
                abs_constraints.available.height = AvailableSpace::MaxContent;
            }

            for (child, inset) in absolute_children.iter().copied() {
                let child_size = cx.measure_in(child, abs_constraints);
                let left = inset.left.map(|v| v.0);
                let right = inset.right.map(|v| v.0);
                let top = inset.top.map(|v| v.0);
                let bottom = inset.bottom.map(|v| v.0);

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

                if cx.available.width.0 <= 0.0 {
                    max_child.width = Px(max_child.width.0.max(required_w.0));
                }
                if cx.available.height.0 <= 0.0 {
                    max_child.height = Px(max_child.height.0.max(required_h.0));
                }
            }
        }

        // `clamp_to_constraints()` treats `available` as a hard maximum. During intrinsic sizing,
        // parent layouts may pass `available.{width,height} = 0` as a placeholder for "unknown",
        // which would incorrectly collapse auto-sized positioned containers to zero even when
        // children measure non-zero.
        //
        // When `available` is zero, use the measured child size as the effective available upper
        // bound so the container can shrink-wrap.
        let mut clamp_available = cx.available;
        if clamp_available.width.0 <= 0.0 {
            clamp_available.width = Px(max_child.width.0.max(0.0));
        }
        if clamp_available.height.0 <= 0.0 {
            clamp_available.height = Px(max_child.height.0.max(0.0));
        }

        let desired = clamp_to_constraints(max_child, layout, clamp_available);
        let base = Rect::new(cx.bounds.origin, desired);
        let probe_bounds = base;

        for &child in cx.children {
            let child_style = layout_style_for_node(cx.app, window, child);
            match positioned_layout_style(child_style) {
                PositionedLayoutStyle::Absolute(inset) => {
                    layout_absolute_child_with_probe_bounds(cx, child, base, probe_bounds, inset)
                }
                style => layout_positioned_child(cx, child, base, style),
            }
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
        let probe_available = clamp_to_constraints(cx.available, layout, cx.available);
        let probe_bounds = Rect::new(cx.bounds.origin, probe_available);
        let probe_constraints = probe_constraints_for_size(probe_bounds.size);
        let mut child_measure_constraints = probe_constraints;
        if cx.available.width.0 <= 0.0 {
            child_measure_constraints.available.width = AvailableSpace::MaxContent;
        }
        if cx.available.height.0 <= 0.0 {
            child_measure_constraints.available.height = AvailableSpace::MaxContent;
        }
        let mut max_child = Size::new(Px(0.0), Px(0.0));

        for &child in cx.children {
            let child_style = layout_style_for_node(cx.app, window, child);
            let child_size = cx.measure_in(child, child_measure_constraints);

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

        let mut clamp_available = cx.available;
        if clamp_available.width.0 <= 0.0 {
            clamp_available.width = Px(max_child.width.0.max(0.0));
        }
        if clamp_available.height.0 <= 0.0 {
            clamp_available.height = Px(max_child.height.0.max(0.0));
        }

        let desired = clamp_to_constraints(max_child, layout, clamp_available);
        let base = Rect::new(cx.bounds.origin, desired);
        let probe_bounds = base;

        for &child in cx.children {
            let child_style = layout_style_for_node(cx.app, window, child);
            match positioned_layout_style(child_style) {
                PositionedLayoutStyle::Absolute(inset) => {
                    layout_absolute_child_with_probe_bounds(cx, child, base, probe_bounds, inset)
                }
                style => layout_positioned_child(cx, child, base, style),
            }
        }
        desired
    }
}

fn probe_constraints_for_size(size: Size) -> LayoutConstraints {
    LayoutConstraints::new(
        LayoutSize::new(None, None),
        LayoutSize::new(
            AvailableSpace::Definite(size.width),
            AvailableSpace::Definite(size.height),
        ),
    )
}
