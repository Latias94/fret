use super::prelude::*;

#[derive(Debug, Clone, Copy)]
pub(super) enum PositionedLayoutStyle {
    Static,
    Relative(crate::element::InsetStyle),
    Absolute {
        inset: crate::element::InsetStyle,
        size: crate::element::SizeStyle,
    },
}

pub(super) fn positioned_layout_style(layout: LayoutStyle) -> PositionedLayoutStyle {
    match layout.position {
        crate::element::PositionStyle::Static => PositionedLayoutStyle::Static,
        crate::element::PositionStyle::Relative => PositionedLayoutStyle::Relative(layout.inset),
        crate::element::PositionStyle::Absolute => PositionedLayoutStyle::Absolute {
            inset: layout.inset,
            size: layout.size,
        },
    }
}

pub(super) fn absolute_child_envelope_size(
    child_size: Size,
    inset: crate::element::InsetStyle,
) -> Size {
    fn edge_components(edge: crate::element::InsetEdge) -> (f32, f32) {
        match edge {
            crate::element::InsetEdge::Px(px) => (px.0, 0.0),
            crate::element::InsetEdge::Fraction(f) => {
                let f = if f.is_finite() { f.max(0.0) } else { 0.0 };
                (0.0, f)
            }
            crate::element::InsetEdge::Auto | crate::element::InsetEdge::Fill => (0.0, 0.0),
        }
    }

    fn axis_extent(
        child_extent: Px,
        start: crate::element::InsetEdge,
        end: crate::element::InsetEdge,
    ) -> Px {
        let (start_px, start_fraction) = edge_components(start);
        let (end_px, end_fraction) = edge_components(end);
        let required = start_px + end_px + child_extent.0;
        let denom = 1.0 - start_fraction - end_fraction;
        let extent = if denom > f32::EPSILON {
            (required / denom).max(0.0)
        } else {
            required.max(0.0)
        };

        if start_fraction > 0.0 || end_fraction > 0.0 {
            Px(extent.ceil())
        } else {
            Px(extent)
        }
    }

    Size::new(
        axis_extent(child_size.width, inset.left, inset.right),
        axis_extent(child_size.height, inset.top, inset.bottom),
    )
}

pub(super) fn layout_positioned_child<H: UiHost>(
    cx: &mut LayoutCx<'_, H>,
    child: NodeId,
    base: Rect,
    style: PositionedLayoutStyle,
) {
    match style {
        PositionedLayoutStyle::Static => {
            cx.solve_barrier_child_root_if_needed(child, base);
            let _ = cx.layout_in(child, base);
        }
        PositionedLayoutStyle::Relative(inset) => {
            let resolve = |edge: crate::element::InsetEdge, basis: Px| -> Px {
                match edge {
                    crate::element::InsetEdge::Px(px) => px,
                    crate::element::InsetEdge::Fill => Px(basis.0.max(0.0)),
                    crate::element::InsetEdge::Fraction(f) => {
                        let f = if f.is_finite() { f.max(0.0) } else { 0.0 };
                        Px((basis.0.max(0.0) * f).max(0.0))
                    }
                    crate::element::InsetEdge::Auto => Px(0.0),
                }
            };

            let left = resolve(inset.left, base.size.width);
            let right = resolve(inset.right, base.size.width);
            let top = resolve(inset.top, base.size.height);
            let bottom = resolve(inset.bottom, base.size.height);

            let dx = left.0 - right.0;
            let dy = top.0 - bottom.0;
            let origin = fret_core::Point::new(Px(base.origin.x.0 + dx), Px(base.origin.y.0 + dy));
            let bounds = Rect::new(origin, base.size);
            cx.solve_barrier_child_root_if_needed(child, bounds);
            let _ = cx.layout_in(child, bounds);
        }
        PositionedLayoutStyle::Absolute { inset, size } => {
            let measured = cx.layout_in_probe(child, base);
            let bounds = absolute_positioned_bounds(measured, base, inset, size, true);
            cx.solve_barrier_child_root_if_needed(child, bounds);
            let _ = cx.layout_in(child, bounds);
        }
    }
}

fn resolve_inset_edge(edge: crate::element::InsetEdge, basis: Px) -> Option<Px> {
    match edge {
        crate::element::InsetEdge::Px(px) => Some(px),
        crate::element::InsetEdge::Fill => Some(Px(basis.0.max(0.0))),
        crate::element::InsetEdge::Fraction(f) => {
            let f = if f.is_finite() { f.max(0.0) } else { 0.0 };
            Some(Px((basis.0.max(0.0) * f).max(0.0)))
        }
        crate::element::InsetEdge::Auto => None,
    }
}

fn resolve_absolute_length(
    length: crate::element::Length,
    measured: Px,
    basis: Px,
    clamp_auto_to_base: bool,
) -> Px {
    match length {
        crate::element::Length::Auto if clamp_auto_to_base => {
            Px(measured.0.min(basis.0.max(0.0)).max(0.0))
        }
        crate::element::Length::Auto => Px(measured.0.max(0.0)),
        crate::element::Length::Px(px) => Px(px.0.max(0.0)),
        crate::element::Length::Fill => Px(basis.0.max(0.0)),
        crate::element::Length::Fraction(f) => {
            let f = if f.is_finite() { f.max(0.0) } else { 0.0 };
            Px((basis.0.max(0.0) * f).max(0.0))
        }
    }
}

fn absolute_positioned_bounds(
    measured: Size,
    base: Rect,
    inset: crate::element::InsetStyle,
    size: crate::element::SizeStyle,
    clamp_auto_to_base: bool,
) -> Rect {
    let left = resolve_inset_edge(inset.left, base.size.width);
    let right = resolve_inset_edge(inset.right, base.size.width);
    let top = resolve_inset_edge(inset.top, base.size.height);
    let bottom = resolve_inset_edge(inset.bottom, base.size.height);

    let left_px = left.unwrap_or(Px(0.0));
    let right_px = right.unwrap_or(Px(0.0));
    let top_px = top.unwrap_or(Px(0.0));
    let bottom_px = bottom.unwrap_or(Px(0.0));

    let w = if left.is_some() && right.is_some() {
        Px((base.size.width.0 - left_px.0 - right_px.0).max(0.0))
    } else {
        resolve_absolute_length(
            size.width,
            measured.width,
            base.size.width,
            clamp_auto_to_base,
        )
    };
    let h = if top.is_some() && bottom.is_some() {
        Px((base.size.height.0 - top_px.0 - bottom_px.0).max(0.0))
    } else {
        resolve_absolute_length(
            size.height,
            measured.height,
            base.size.height,
            clamp_auto_to_base,
        )
    };

    let x = if left.is_some() {
        left_px
    } else if right.is_some() {
        Px((base.size.width.0 - right_px.0 - w.0).max(0.0))
    } else {
        Px(0.0)
    };
    let y = if top.is_some() {
        top_px
    } else if bottom.is_some() {
        Px((base.size.height.0 - bottom_px.0 - h.0).max(0.0))
    } else {
        Px(0.0)
    };

    let origin = fret_core::Point::new(Px(base.origin.x.0 + x.0), Px(base.origin.y.0 + y.0));
    Rect::new(origin, Size::new(w, h))
}

pub(super) fn layout_absolute_child_with_probe_bounds<H: UiHost>(
    cx: &mut LayoutCx<'_, H>,
    child: NodeId,
    base: Rect,
    probe: Rect,
    inset: crate::element::InsetStyle,
    size: crate::element::SizeStyle,
) {
    let measured = cx.layout_in_probe(child, probe);
    let bounds = absolute_positioned_bounds(measured, base, inset, size, false);
    cx.solve_barrier_child_root_if_needed(child, bounds);
    let _ = cx.layout_in(child, bounds);
}

pub(super) fn clamp_to_constraints(size: Size, style: LayoutStyle, available: Size) -> Size {
    clamp_to_constraints_with_overflow_context(
        size,
        style,
        available,
        crate::layout::overflow::LayoutOverflowContext::default_for_layout(),
    )
}

pub(super) fn clamp_to_constraints_with_overflow_context(
    mut size: Size,
    style: LayoutStyle,
    available: Size,
    overflow_ctx: crate::layout::overflow::LayoutOverflowContext,
) -> Size {
    let resolve_constraint = |l: Length, base: Px| -> Option<Px> {
        match l {
            Length::Auto => None,
            Length::Px(px) => Some(Px(px.0.max(0.0))),
            Length::Fill => Some(Px(base.0.max(0.0))),
            Length::Fraction(f) => {
                let f = if f.is_finite() { f.max(0.0) } else { 0.0 };
                Some(Px((base.0 * f).max(0.0)))
            }
        }
    };

    let width_auto = matches!(style.size.width, Length::Auto);
    let height_auto = matches!(style.size.height, Length::Auto);

    match style.size.width {
        Length::Px(px) => size.width = Px(px.0.max(0.0)),
        Length::Fill => size.width = available.width,
        Length::Fraction(f) => {
            let f = if f.is_finite() { f.max(0.0) } else { 0.0 };
            size.width = Px((available.width.0 * f).max(0.0));
        }
        Length::Auto => {}
    }
    match style.size.height {
        Length::Px(px) => size.height = Px(px.0.max(0.0)),
        Length::Fill => size.height = available.height,
        Length::Fraction(f) => {
            let f = if f.is_finite() { f.max(0.0) } else { 0.0 };
            size.height = Px((available.height.0 * f).max(0.0));
        }
        Length::Auto => {}
    }

    if let Some(min_w) = style
        .size
        .min_width
        .and_then(|l| resolve_constraint(l, available.width))
    {
        size.width = Px(size.width.0.max(min_w.0.max(0.0)));
    }
    if let Some(min_h) = style
        .size
        .min_height
        .and_then(|l| resolve_constraint(l, available.height))
    {
        size.height = Px(size.height.0.max(min_h.0.max(0.0)));
    }
    if let Some(max_w) = style
        .size
        .max_width
        .and_then(|l| resolve_constraint(l, available.width))
    {
        size.width = Px(size.width.0.min(max_w.0.max(0.0)));
    }
    if let Some(max_h) = style
        .size
        .max_height
        .and_then(|l| resolve_constraint(l, available.height))
    {
        size.height = Px(size.height.0.min(max_h.0.max(0.0)));
    }

    let clamp_w = !width_auto || !overflow_ctx.allow_overflow_on_auto.width;
    let clamp_h = !height_auto || !overflow_ctx.allow_overflow_on_auto.height;
    size.width = if clamp_w {
        Px(size.width.0.max(0.0).min(available.width.0.max(0.0)))
    } else {
        Px(size.width.0.max(0.0))
    };
    size.height = if clamp_h {
        Px(size.height.0.max(0.0).min(available.height.0.max(0.0)))
    } else {
        Px(size.height.0.max(0.0))
    };

    if let Some(ratio) = style.aspect_ratio
        && ratio.is_finite()
        && ratio > 0.0
    {
        if height_auto && !width_auto {
            size.height = Px((size.width.0 / ratio).max(0.0));
        } else if width_auto && !height_auto {
            size.width = Px((size.height.0 * ratio).max(0.0));
        }

        if let Some(min_w) = style
            .size
            .min_width
            .and_then(|l| resolve_constraint(l, available.width))
        {
            size.width = Px(size.width.0.max(min_w.0.max(0.0)));
        }
        if let Some(min_h) = style
            .size
            .min_height
            .and_then(|l| resolve_constraint(l, available.height))
        {
            size.height = Px(size.height.0.max(min_h.0.max(0.0)));
        }
        if let Some(max_w) = style
            .size
            .max_width
            .and_then(|l| resolve_constraint(l, available.width))
        {
            size.width = Px(size.width.0.min(max_w.0.max(0.0)));
        }
        if let Some(max_h) = style
            .size
            .max_height
            .and_then(|l| resolve_constraint(l, available.height))
        {
            size.height = Px(size.height.0.min(max_h.0.max(0.0)));
        }

        size.width = if clamp_w {
            Px(size.width.0.max(0.0).min(available.width.0.max(0.0)))
        } else {
            Px(size.width.0.max(0.0))
        };
        size.height = if clamp_h {
            Px(size.height.0.max(0.0).min(available.height.0.max(0.0)))
        } else {
            Px(size.height.0.max(0.0))
        };
    }
    size
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::layout_constraints::LayoutSize;

    #[test]
    fn clamp_to_constraints_auto_clamps_to_available_by_default() {
        let size = Size::new(Px(100.0), Px(300.0));
        let available = Size::new(Px(200.0), Px(100.0));
        let style = LayoutStyle::default();
        let out = clamp_to_constraints(size, style, available);
        assert_eq!(out.width, Px(100.0));
        assert_eq!(out.height, Px(100.0));
    }

    #[test]
    fn clamp_to_constraints_auto_can_overflow_under_overflow_context() {
        let size = Size::new(Px(100.0), Px(300.0));
        let available = Size::new(Px(200.0), Px(100.0));
        let style = LayoutStyle::default();
        let overflow_ctx = crate::layout::overflow::LayoutOverflowContext {
            allow_overflow_on_auto: LayoutSize::new(false, true),
            ..crate::layout::overflow::LayoutOverflowContext::default_for_layout()
        };
        let out = clamp_to_constraints_with_overflow_context(size, style, available, overflow_ctx);
        assert_eq!(out.width, Px(100.0));
        assert_eq!(out.height, Px(300.0));
    }
}
