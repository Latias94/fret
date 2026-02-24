use super::prelude::*;

#[derive(Debug, Clone, Copy)]
pub(super) enum PositionedLayoutStyle {
    Static,
    Relative(crate::element::InsetStyle),
    Absolute(crate::element::InsetStyle),
}

pub(super) fn positioned_layout_style(layout: LayoutStyle) -> PositionedLayoutStyle {
    match layout.position {
        crate::element::PositionStyle::Static => PositionedLayoutStyle::Static,
        crate::element::PositionStyle::Relative => PositionedLayoutStyle::Relative(layout.inset),
        crate::element::PositionStyle::Absolute => PositionedLayoutStyle::Absolute(layout.inset),
    }
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
        PositionedLayoutStyle::Absolute(inset) => {
            let measured = cx.layout_in_probe(child, base);

            let resolve = |edge: crate::element::InsetEdge, basis: Px| -> Option<Px> {
                match edge {
                    crate::element::InsetEdge::Px(px) => Some(px),
                    crate::element::InsetEdge::Fill => Some(Px(basis.0.max(0.0))),
                    crate::element::InsetEdge::Fraction(f) => {
                        let f = if f.is_finite() { f.max(0.0) } else { 0.0 };
                        Some(Px((basis.0.max(0.0) * f).max(0.0)))
                    }
                    crate::element::InsetEdge::Auto => None,
                }
            };

            let left = resolve(inset.left, base.size.width);
            let right = resolve(inset.right, base.size.width);
            let top = resolve(inset.top, base.size.height);
            let bottom = resolve(inset.bottom, base.size.height);

            let left_px = left.unwrap_or(Px(0.0));
            let right_px = right.unwrap_or(Px(0.0));
            let top_px = top.unwrap_or(Px(0.0));
            let bottom_px = bottom.unwrap_or(Px(0.0));

            let w = if left.is_some() && right.is_some() {
                Px((base.size.width.0 - left_px.0 - right_px.0).max(0.0))
            } else {
                Px(measured.width.0.min(base.size.width.0.max(0.0)).max(0.0))
            };
            let h = if top.is_some() && bottom.is_some() {
                Px((base.size.height.0 - top_px.0 - bottom_px.0).max(0.0))
            } else {
                Px(measured.height.0.min(base.size.height.0.max(0.0)).max(0.0))
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

            let origin =
                fret_core::Point::new(Px(base.origin.x.0 + x.0), Px(base.origin.y.0 + y.0));
            let bounds = Rect::new(origin, Size::new(w, h));
            cx.solve_barrier_child_root_if_needed(child, bounds);
            let _ = cx.layout_in(child, bounds);
        }
    }
}

pub(super) fn layout_absolute_child_with_probe_bounds<H: UiHost>(
    cx: &mut LayoutCx<'_, H>,
    child: NodeId,
    base: Rect,
    probe: Rect,
    inset: crate::element::InsetStyle,
) {
    let measured = cx.layout_in_probe(child, probe);

    let resolve = |edge: crate::element::InsetEdge, basis: Px| -> Option<Px> {
        match edge {
            crate::element::InsetEdge::Px(px) => Some(px),
            crate::element::InsetEdge::Fill => Some(Px(basis.0.max(0.0))),
            crate::element::InsetEdge::Fraction(f) => {
                let f = if f.is_finite() { f.max(0.0) } else { 0.0 };
                Some(Px((basis.0.max(0.0) * f).max(0.0)))
            }
            crate::element::InsetEdge::Auto => None,
        }
    };

    let left = resolve(inset.left, base.size.width);
    let right = resolve(inset.right, base.size.width);
    let top = resolve(inset.top, base.size.height);
    let bottom = resolve(inset.bottom, base.size.height);

    let left_px = left.unwrap_or(Px(0.0));
    let right_px = right.unwrap_or(Px(0.0));
    let top_px = top.unwrap_or(Px(0.0));
    let bottom_px = bottom.unwrap_or(Px(0.0));

    let w = if left.is_some() && right.is_some() {
        Px((base.size.width.0 - left_px.0 - right_px.0).max(0.0))
    } else {
        Px(measured.width.0.max(0.0))
    };

    let h = if top.is_some() && bottom.is_some() {
        Px((base.size.height.0 - top_px.0 - bottom_px.0).max(0.0))
    } else {
        Px(measured.height.0.max(0.0))
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
    let bounds = Rect::new(origin, Size::new(w, h));
    cx.solve_barrier_child_root_if_needed(child, bounds);
    let _ = cx.layout_in(child, bounds);
}

pub(super) fn clamp_to_constraints(mut size: Size, style: LayoutStyle, available: Size) -> Size {
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

    size.width = Px(size.width.0.max(0.0).min(available.width.0.max(0.0)));
    size.height = Px(size.height.0.max(0.0).min(available.height.0.max(0.0)));

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

        size.width = Px(size.width.0.max(0.0).min(available.width.0.max(0.0)));
        size.height = Px(size.height.0.max(0.0).min(available.height.0.max(0.0)));
    }
    size
}
