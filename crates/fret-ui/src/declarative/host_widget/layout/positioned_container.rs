use super::super::ElementHostWidget;
use crate::declarative::frame::layout_style_for_node;
use crate::declarative::layout_helpers::{
    PositionedLayoutStyle, absolute_child_envelope_size, absolute_child_envelope_size_if_definite,
    clamp_to_constraints, clamp_to_constraints_with_overflow_context,
    layout_absolute_child_with_probe_bounds, layout_positioned_child, positioned_layout_style,
};
use crate::declarative::prelude::*;
use crate::layout_constraints::AvailableSpace;

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
        let probe_constraints = cx.probe_constraints_for_size(probe_bounds.size);
        let mut child_measure_constraints = probe_constraints;
        if cx.available.width.0 <= 0.0 {
            child_measure_constraints.available.width = AvailableSpace::MaxContent;
        }
        if cx.available.height.0 <= 0.0 {
            child_measure_constraints.available.height = AvailableSpace::MaxContent;
        }
        let mut max_child = Size::new(Px(0.0), Px(0.0));
        let mut non_absolute_sizes: Vec<(NodeId, Size)> = Vec::new();
        let mut absolute_children: Vec<(
            NodeId,
            crate::element::InsetStyle,
            crate::element::SizeStyle,
        )> = Vec::new();
        for &child in cx.children {
            let child_style = layout_style_for_node(cx.app, window, child);
            if child_style.position == crate::element::PositionStyle::Absolute {
                absolute_children.push((child, child_style.inset, child_style.size));
                continue;
            }
            let child_size = cx.measure_in(child, child_measure_constraints);
            non_absolute_sizes.push((child, child_size));
            max_child.width = Px(max_child.width.0.max(child_size.width.0));
            max_child.height = Px(max_child.height.0.max(child_size.height.0));
        }

        if !absolute_children.is_empty() {
            let mut abs_constraints = probe_constraints;
            let envelope_width = matches!(layout.size.width, Length::Auto)
                || cx.available.width.0 <= 0.0
                || max_child.width.0 <= 0.0;
            let envelope_height = matches!(layout.size.height, Length::Auto)
                || cx.available.height.0 <= 0.0
                || max_child.height.0 <= 0.0;
            if envelope_width {
                abs_constraints.available.width = AvailableSpace::MaxContent;
            }
            if envelope_height {
                abs_constraints.available.height = AvailableSpace::MaxContent;
            }

            for (child, inset, size) in absolute_children.iter().copied() {
                let required = absolute_child_envelope_size_if_definite(inset, size)
                    .unwrap_or_else(|| {
                        let child_size = cx.measure_in(child, abs_constraints);
                        absolute_child_envelope_size(child_size, inset)
                    });

                if envelope_width {
                    max_child.width = Px(max_child.width.0.max(required.width.0));
                }
                if envelope_height {
                    max_child.height = Px(max_child.height.0.max(required.height.0));
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

        let desired = clamp_to_constraints_with_overflow_context(
            max_child,
            layout,
            clamp_available,
            cx.overflow_ctx,
        );
        let base = Rect::new(cx.bounds.origin, desired);
        let probe_bounds = base;

        for &child in cx.children {
            let child_style = layout_style_for_node(cx.app, window, child);
            match positioned_layout_style(child_style) {
                PositionedLayoutStyle::Absolute { inset, size } => {
                    layout_absolute_child_with_probe_bounds(
                        cx,
                        child,
                        base,
                        probe_bounds,
                        inset,
                        size,
                    )
                }
                PositionedLayoutStyle::Static => {
                    let child_size = non_absolute_sizes
                        .iter()
                        .find_map(|(id, size)| (*id == child).then_some(*size))
                        .unwrap_or(Size::new(Px(0.0), Px(0.0)));
                    let child_size = static_child_size_for_base(child_style, child_size, base.size);
                    let _ = cx.layout_in(child, Rect::new(base.origin, child_size));
                }
                PositionedLayoutStyle::Relative(inset) => {
                    let child_size = non_absolute_sizes
                        .iter()
                        .find_map(|(id, size)| (*id == child).then_some(*size))
                        .unwrap_or(Size::new(Px(0.0), Px(0.0)));
                    let child_size = static_child_size_for_base(child_style, child_size, base.size);
                    layout_positioned_child(
                        cx,
                        child,
                        Rect::new(base.origin, child_size),
                        PositionedLayoutStyle::Relative(inset),
                    );
                }
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
        let probe_constraints = cx.probe_constraints_for_size(probe_bounds.size);
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
            let required = if child_style.position == crate::element::PositionStyle::Absolute {
                absolute_child_envelope_size_if_definite(child_style.inset, child_style.size)
                    .unwrap_or_else(|| {
                        let child_size = cx.measure_in(child, child_measure_constraints);
                        absolute_child_envelope_size(child_size, child_style.inset)
                    })
            } else {
                cx.measure_in(child, child_measure_constraints)
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

        let desired = clamp_to_constraints_with_overflow_context(
            max_child,
            layout,
            clamp_available,
            cx.overflow_ctx,
        );
        let base = Rect::new(cx.bounds.origin, desired);
        let probe_bounds = base;

        for &child in cx.children {
            let child_style = layout_style_for_node(cx.app, window, child);
            match positioned_layout_style(child_style) {
                PositionedLayoutStyle::Absolute { inset, size } => {
                    layout_absolute_child_with_probe_bounds(
                        cx,
                        child,
                        base,
                        probe_bounds,
                        inset,
                        size,
                    )
                }
                style => layout_positioned_child(cx, child, base, style),
            }
        }
        desired
    }
}

fn static_child_size_for_base(style: LayoutStyle, measured: Size, base: Size) -> Size {
    fn axis(length: Length, measured: Px, base: Px) -> Px {
        match length {
            Length::Fill => Px(base.0.max(0.0)),
            Length::Fraction(f) => {
                let f = if f.is_finite() { f.max(0.0) } else { 0.0 };
                Px((base.0.max(0.0) * f).max(0.0))
            }
            Length::Auto | Length::Px(_) => measured,
        }
    }

    Size::new(
        axis(style.size.width, measured.width, base.width),
        axis(style.size.height, measured.height, base.height),
    )
}

// Intentionally omitted: probe constraint construction is context-dependent (scroll overflow
// contexts may override the scroll axis to `MaxContent`). Use `LayoutCx::probe_constraints_for_size`.
