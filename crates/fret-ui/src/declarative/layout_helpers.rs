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

fn absolute_axis_has_definite_envelope(
    start: crate::element::InsetEdge,
    end: crate::element::InsetEdge,
    length: crate::element::Length,
) -> bool {
    matches!(length, crate::element::Length::Px(_))
        && absolute_envelope_edge_px(start).is_some()
        && absolute_envelope_edge_px(end).is_some()
}

fn absolute_size_has_no_extra_constraints(size: crate::element::SizeStyle) -> bool {
    size.min_width.is_none()
        && size.min_height.is_none()
        && size.max_width.is_none()
        && size.max_height.is_none()
}

fn absolute_envelope_edge_px(edge: crate::element::InsetEdge) -> Option<Px> {
    match edge {
        crate::element::InsetEdge::Px(px) => Some(px),
        crate::element::InsetEdge::Fill | crate::element::InsetEdge::Auto => Some(Px(0.0)),
        crate::element::InsetEdge::Fraction(_) => None,
    }
}

fn resolve_absolute_envelope_axis(
    start: crate::element::InsetEdge,
    end: crate::element::InsetEdge,
    length: crate::element::Length,
) -> Option<Px> {
    let start_px = absolute_envelope_edge_px(start)?;
    let end_px = absolute_envelope_edge_px(end)?;

    let crate::element::Length::Px(measured) = length else {
        return None;
    };
    Some(Px((start_px.0 + end_px.0 + measured.0).max(0.0)))
}

pub(super) fn absolute_child_envelope_size_if_definite(
    inset: crate::element::InsetStyle,
    size: crate::element::SizeStyle,
) -> Option<Size> {
    if !absolute_size_has_no_extra_constraints(size) {
        return None;
    }

    if !absolute_axis_has_definite_envelope(inset.left, inset.right, size.width)
        || !absolute_axis_has_definite_envelope(inset.top, inset.bottom, size.height)
    {
        return None;
    }

    Some(Size::new(
        resolve_absolute_envelope_axis(inset.left, inset.right, size.width).unwrap_or(Px(0.0)),
        resolve_absolute_envelope_axis(inset.top, inset.bottom, size.height).unwrap_or(Px(0.0)),
    ))
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
            let bounds = absolute_positioned_bounds_if_definite(base, inset, size, true)
                .unwrap_or_else(|| {
                    let measured = cx.layout_in_probe(child, base);
                    absolute_positioned_bounds(measured, base, inset, size, true)
                });
            layout_absolute_child_with_bounds(cx, child, bounds);
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

fn absolute_axis_is_definite(
    start: Option<Px>,
    end: Option<Px>,
    length: crate::element::Length,
) -> bool {
    (start.is_some() && end.is_some()) || !matches!(length, crate::element::Length::Auto)
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

fn absolute_positioned_bounds_if_definite(
    base: Rect,
    inset: crate::element::InsetStyle,
    size: crate::element::SizeStyle,
    clamp_auto_to_base: bool,
) -> Option<Rect> {
    let left = resolve_inset_edge(inset.left, base.size.width);
    let right = resolve_inset_edge(inset.right, base.size.width);
    let top = resolve_inset_edge(inset.top, base.size.height);
    let bottom = resolve_inset_edge(inset.bottom, base.size.height);

    if !absolute_axis_is_definite(left, right, size.width)
        || !absolute_axis_is_definite(top, bottom, size.height)
    {
        return None;
    }

    Some(absolute_positioned_bounds(
        Size::new(Px(0.0), Px(0.0)),
        base,
        inset,
        size,
        clamp_auto_to_base,
    ))
}

pub(super) fn layout_absolute_child_with_bounds<H: UiHost>(
    cx: &mut LayoutCx<'_, H>,
    child: NodeId,
    bounds: Rect,
) {
    cx.solve_barrier_child_root_if_needed(child, bounds);
    let _ = cx.layout_in(child, bounds);
}

pub(super) fn layout_absolute_child_with_definite_or_probe_bounds<H: UiHost>(
    cx: &mut LayoutCx<'_, H>,
    child: NodeId,
    base: Rect,
    probe: Rect,
    inset: crate::element::InsetStyle,
    size: crate::element::SizeStyle,
) {
    let bounds =
        absolute_positioned_bounds_if_definite(base, inset, size, false).unwrap_or_else(|| {
            let measured = cx.layout_in_probe(child, probe);
            absolute_positioned_bounds(measured, base, inset, size, false)
        });
    layout_absolute_child_with_bounds(cx, child, bounds);
}

pub(super) fn layout_absolute_child_with_probe_bounds<H: UiHost>(
    cx: &mut LayoutCx<'_, H>,
    child: NodeId,
    base: Rect,
    probe: Rect,
    inset: crate::element::InsetStyle,
    size: crate::element::SizeStyle,
) {
    layout_absolute_child_with_definite_or_probe_bounds(cx, child, base, probe, inset, size);
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
    use crate::element::{InsetEdge, InsetStyle, Length, SizeStyle};
    use crate::layout_constraints::LayoutSize;

    fn rect_xywh(x: f32, y: f32, w: f32, h: f32) -> Rect {
        Rect::new(fret_core::Point::new(Px(x), Px(y)), Size::new(Px(w), Px(h)))
    }

    #[test]
    fn absolute_definite_bounds_resolve_without_measured_size() {
        let base = rect_xywh(10.0, 20.0, 200.0, 100.0);
        let inset = InsetStyle {
            left: InsetEdge::Px(Px(12.0)),
            top: InsetEdge::Px(Px(8.0)),
            ..Default::default()
        };
        let size = SizeStyle {
            width: Length::Px(Px(40.0)),
            height: Length::Px(Px(16.0)),
            ..Default::default()
        };

        let bounds = absolute_positioned_bounds_if_definite(base, inset, size, false)
            .expect("explicit absolute size should not require a probe");

        assert_eq!(bounds.origin.x, Px(22.0));
        assert_eq!(bounds.origin.y, Px(28.0));
        assert_eq!(bounds.size.width, Px(40.0));
        assert_eq!(bounds.size.height, Px(16.0));
    }

    #[test]
    fn absolute_dual_insets_resolve_without_measured_size() {
        let base = rect_xywh(10.0, 20.0, 200.0, 100.0);
        let inset = InsetStyle {
            left: InsetEdge::Px(Px(12.0)),
            right: InsetEdge::Px(Px(18.0)),
            top: InsetEdge::Px(Px(8.0)),
            bottom: InsetEdge::Px(Px(14.0)),
        };
        let size = SizeStyle::default();

        let bounds = absolute_positioned_bounds_if_definite(base, inset, size, false)
            .expect("dual absolute insets should not require a probe");

        assert_eq!(bounds.origin.x, Px(22.0));
        assert_eq!(bounds.origin.y, Px(28.0));
        assert_eq!(bounds.size.width, Px(170.0));
        assert_eq!(bounds.size.height, Px(78.0));
    }

    #[test]
    fn absolute_auto_axis_still_requires_probe_measurement() {
        let base = rect_xywh(10.0, 20.0, 200.0, 100.0);
        let inset = InsetStyle {
            left: InsetEdge::Px(Px(12.0)),
            top: InsetEdge::Px(Px(8.0)),
            ..Default::default()
        };
        let size = SizeStyle {
            width: Length::Auto,
            height: Length::Px(Px(16.0)),
            ..Default::default()
        };

        assert!(
            absolute_positioned_bounds_if_definite(base, inset, size, false).is_none(),
            "an auto absolute axis must still use measured child size"
        );
    }

    #[test]
    fn absolute_definite_envelope_uses_explicit_size_without_child_measurement() {
        let inset = InsetStyle {
            left: InsetEdge::Px(Px(8.0)),
            right: InsetEdge::Px(Px(12.0)),
            top: InsetEdge::Px(Px(3.0)),
            bottom: InsetEdge::Auto,
        };
        let size = SizeStyle {
            width: Length::Px(Px(40.0)),
            height: Length::Px(Px(16.0)),
            ..Default::default()
        };

        let envelope = absolute_child_envelope_size_if_definite(inset, size)
            .expect("fixed absolute child envelope should be static");

        assert_eq!(envelope.width, Px(60.0));
        assert_eq!(envelope.height, Px(19.0));
    }

    #[test]
    fn absolute_fraction_inset_envelope_still_requires_measurement() {
        let inset = InsetStyle {
            left: InsetEdge::Fraction(0.25),
            right: InsetEdge::Px(Px(12.0)),
            top: InsetEdge::Px(Px(3.0)),
            bottom: InsetEdge::Auto,
        };
        let size = SizeStyle {
            width: Length::Px(Px(40.0)),
            height: Length::Px(Px(16.0)),
            ..Default::default()
        };

        assert!(
            absolute_child_envelope_size_if_definite(inset, size).is_none(),
            "fractional insets need the existing measured envelope math"
        );
    }

    #[test]
    fn absolute_constrained_size_envelope_still_requires_measurement() {
        let inset = InsetStyle::default();
        let size = SizeStyle {
            width: Length::Px(Px(40.0)),
            height: Length::Px(Px(16.0)),
            min_width: Some(Length::Px(Px(64.0))),
            ..Default::default()
        };

        assert!(
            absolute_child_envelope_size_if_definite(inset, size).is_none(),
            "extra size constraints stay on the measured path"
        );
    }

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
