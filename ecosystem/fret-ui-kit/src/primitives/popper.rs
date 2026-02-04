//! Popper / floating placement helpers (Radix `@radix-ui/react-popper` outcomes).
//!
//! This primitive is a thin, stable wrapper around `fret-ui`'s deterministic placement solver
//! (`fret_ui::overlay_placement`). It is intentionally pure and testable.

use fret_core::{Edges, Point, Px, Rect, Size};
use fret_ui::overlay_placement::{
    AnchoredPanelLayout, AnchoredPanelOptions, CollisionOptions, anchored_panel_layout_ex,
    anchored_panel_layout_sized_ex, inset_rect, intersect_rect,
};

pub use fret_ui::overlay_placement::{
    Align, ArrowLayout, ArrowOptions, LayoutDirection, Offset, ShiftOptions, Side, StickyMode,
};

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct PopperAvailableMetrics {
    pub available_width: Px,
    pub available_height: Px,
    pub anchor_width: Px,
    pub anchor_height: Px,
}

/// Build `AnchoredPanelOptions` for popper-like floating content.
///
/// Radix `PopperContent` effectively adds an extra main-axis offset when an arrow is present
/// (the arrow protrudes outside the panel rect), and supports a cross-axis alignment offset.
///
/// In Fret we keep `side_offset` (gap between anchor and panel) separate from the arrow
/// protrusion, so callers pass `arrow_protrusion` here and keep `side_offset` for the solver.
pub fn anchored_panel_options_for_popper_content(
    direction: LayoutDirection,
    arrow_protrusion: Px,
    align_offset: Px,
    arrow: Option<ArrowOptions>,
) -> AnchoredPanelOptions {
    AnchoredPanelOptions {
        direction,
        offset: Offset {
            main_axis: arrow_protrusion,
            cross_axis: align_offset,
            // Radix maps `alignOffset` to Floating UI's `alignmentAxis` offset, which flips sign
            // for `*-end` placements (and flips under RTL for vertical placements).
            alignment_axis: Some(align_offset),
        },
        // Radix uses Floating UI `shift({ crossAxis: false })` by default for popper content.
        shift: ShiftOptions {
            main_axis: true,
            cross_axis: false,
        },
        arrow,
        collision: Default::default(),
        sticky: Default::default(),
    }
}

/// Placement policy for Radix-like `PopperContent`.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct PopperContentPlacement {
    pub direction: LayoutDirection,
    pub side: Side,
    pub align: Align,
    pub side_offset: Px,
    pub align_offset: Px,
    pub arrow: Option<ArrowOptions>,
    pub arrow_protrusion: Px,
    /// Whether to enable cross-axis shifting while resolving collisions.
    ///
    /// Radix typically uses Floating UI's `shift({ crossAxis: false })` by default, but some
    /// primitives opt into cross-axis shifting for better clamping behavior.
    pub shift_cross_axis: bool,
    pub collision_padding: Edges,
    pub collision_boundary: Option<Rect>,
    pub hide_when_detached: bool,
    pub sticky: StickyMode,
}

impl PopperContentPlacement {
    pub fn new(direction: LayoutDirection, side: Side, align: Align, side_offset: Px) -> Self {
        Self {
            direction,
            side,
            align,
            side_offset,
            align_offset: Px(0.0),
            arrow: None,
            arrow_protrusion: Px(0.0),
            shift_cross_axis: false,
            collision_padding: Edges::all(Px(0.0)),
            collision_boundary: None,
            hide_when_detached: false,
            sticky: StickyMode::Partial,
        }
    }

    pub fn with_align_offset(mut self, align_offset: Px) -> Self {
        self.align_offset = align_offset;
        self
    }

    pub fn with_arrow(mut self, arrow: Option<ArrowOptions>, arrow_protrusion: Px) -> Self {
        self.arrow = arrow;
        self.arrow_protrusion = arrow_protrusion;
        self
    }

    pub fn with_shift_cross_axis(mut self, cross_axis: bool) -> Self {
        self.shift_cross_axis = cross_axis;
        self
    }

    pub fn with_collision_padding(mut self, collision_padding: Edges) -> Self {
        self.collision_padding = collision_padding;
        self
    }

    pub fn with_collision_boundary(mut self, collision_boundary: Option<Rect>) -> Self {
        self.collision_boundary = collision_boundary;
        self
    }

    pub fn with_hide_when_detached(mut self, hide_when_detached: bool) -> Self {
        self.hide_when_detached = hide_when_detached;
        self
    }

    pub fn with_sticky(mut self, sticky: StickyMode) -> Self {
        self.sticky = sticky;
        self
    }

    /// Returns `true` when the anchor is fully clipped by the effective collision boundary.
    ///
    /// This approximates Floating UI's `hide({ strategy: 'referenceHidden' })` middleware as used by
    /// Radix (`hideWhenDetached`).
    pub fn reference_hidden(self, outer: Rect, anchor: Rect) -> bool {
        if !self.hide_when_detached {
            return false;
        }

        let mut boundary = outer;
        if let Some(extra_boundary) = self.collision_boundary {
            boundary = fret_ui::overlay_placement::intersect_rect(boundary, extra_boundary);
        }
        boundary = fret_ui::overlay_placement::inset_rect(boundary, self.collision_padding);

        let intersection = fret_ui::overlay_placement::intersect_rect(boundary, anchor);
        intersection.size.width.0 <= 0.0 || intersection.size.height.0 <= 0.0
    }

    pub fn options(self) -> AnchoredPanelOptions {
        let mut options = anchored_panel_options_for_popper_content(
            self.direction,
            self.arrow_protrusion,
            self.align_offset,
            self.arrow,
        );
        options.shift.cross_axis = self.shift_cross_axis;
        options.collision = CollisionOptions {
            padding: self.collision_padding,
            boundary: self.collision_boundary,
        };
        options.sticky = self.sticky;
        options
    }
}

/// Computes a Radix-style `PopperContent` layout from a placement policy.
pub fn popper_content_layout_sized(
    outer: Rect,
    anchor: Rect,
    desired: Size,
    placement: PopperContentPlacement,
) -> AnchoredPanelLayout {
    popper_layout_sized(
        outer,
        anchor,
        desired,
        placement.side_offset,
        placement.side,
        placement.align,
        placement.options(),
    )
}

/// Computes a Radix-style `PopperContent` layout without clamping the panel `Size` to available
/// space.
///
/// This matches primitives where the floating rect can overflow the collision boundary on the
/// main axis while preserving its intrinsic size, but still wants collision shifting/clamping to
/// run for the final origin.
pub fn popper_content_layout_size_unclamped(
    outer: Rect,
    anchor: Rect,
    desired: Size,
    placement: PopperContentPlacement,
) -> AnchoredPanelLayout {
    anchored_panel_layout_ex(
        outer,
        anchor,
        desired,
        placement.side_offset,
        placement.side,
        placement.align,
        placement.options(),
    )
}

/// Computes a Radix-style `PopperContent` layout without clamping the panel size to available space.
///
/// This matches Radix/Floating UI behavior for primitives that allow the floating rect to overflow
/// the viewport while preserving its intrinsic size (e.g. NavigationMenu viewport in mobile mode).
pub fn popper_content_layout_unclamped(
    outer: Rect,
    anchor: Rect,
    desired: Size,
    placement: PopperContentPlacement,
) -> AnchoredPanelLayout {
    let mut options = placement.options();
    // Allow the panel to overflow the collision boundary on the main axis (no shift/clamp).
    options.shift.main_axis = false;
    anchored_panel_layout_ex(
        outer,
        anchor,
        desired,
        placement.side_offset,
        placement.side,
        placement.align,
        options,
    )
}

/// Compute Radix-like "available metrics" exposed by Floating UI's `size()` middleware.
///
/// Radix writes these to CSS variables:
/// - `--radix-popper-available-width`
/// - `--radix-popper-available-height`
/// - `--radix-popper-anchor-width`
/// - `--radix-popper-anchor-height`
///
/// Fret exposes the same concepts as a structured return value.
pub fn popper_available_metrics(
    outer: Rect,
    anchor: Rect,
    layout: &AnchoredPanelLayout,
    direction: LayoutDirection,
) -> PopperAvailableMetrics {
    let rect = layout.rect;
    let width = rect.size.width.0.max(0.0);
    let height = rect.size.height.0.max(0.0);

    let outer_left = outer.origin.x.0;
    let outer_top = outer.origin.y.0;
    let outer_right = outer_left + outer.size.width.0.max(0.0);
    let outer_bottom = outer_top + outer.size.height.0.max(0.0);

    let rect_left = rect.origin.x.0;
    let rect_top = rect.origin.y.0;
    let rect_right = rect_left + rect.size.width.0.max(0.0);
    let rect_bottom = rect_top + rect.size.height.0.max(0.0);

    // Signed overflow values:
    // - positive: overflows the boundary
    // - negative: remaining space within the boundary
    let overflow_left = outer_left - rect_left;
    let overflow_top = outer_top - rect_top;
    let overflow_right = rect_right - outer_right;
    let overflow_bottom = rect_bottom - outer_bottom;

    let maximum_clipping_width = (width - overflow_left - overflow_right).max(0.0);
    let maximum_clipping_height = (height - overflow_top - overflow_bottom).max(0.0);

    let alignment = match layout.align {
        Align::Center => None,
        other => Some(other),
    };

    let side = layout.side;

    let (height_side, width_side) = match side {
        Side::Top | Side::Bottom => {
            let height_side = side;
            let width_side = match alignment {
                Some(Align::Start) => {
                    if direction == LayoutDirection::Rtl {
                        Side::Right
                    } else {
                        Side::Left
                    }
                }
                Some(Align::End) => {
                    if direction == LayoutDirection::Rtl {
                        Side::Left
                    } else {
                        Side::Right
                    }
                }
                _ => Side::Right,
            };
            (height_side, width_side)
        }
        Side::Left | Side::Right => {
            let width_side = side;
            let height_side = match alignment {
                Some(Align::End) => Side::Top,
                _ => Side::Bottom,
            };
            (height_side, width_side)
        }
    };

    let overflow_for_side = |side: Side| match side {
        Side::Top => overflow_top,
        Side::Bottom => overflow_bottom,
        Side::Left => overflow_left,
        Side::Right => overflow_right,
    };

    let overflow_available_height = (height - overflow_for_side(height_side))
        .min(maximum_clipping_height)
        .max(0.0);
    let overflow_available_width = (width - overflow_for_side(width_side))
        .min(maximum_clipping_width)
        .max(0.0);

    // Radix shift config:
    // - `mainAxis: true`
    // - `crossAxis: false`
    // For Top/Bottom, this enables shifting along X. For Left/Right, along Y.
    let shift_enabled_x = matches!(side, Side::Top | Side::Bottom);
    let shift_enabled_y = matches!(side, Side::Left | Side::Right);

    let mut available_height = overflow_available_height;
    let mut available_width = overflow_available_width;

    if shift_enabled_x {
        available_width = maximum_clipping_width;
    }
    if shift_enabled_y {
        available_height = maximum_clipping_height;
    }

    PopperAvailableMetrics {
        available_width: Px(available_width),
        available_height: Px(available_height),
        anchor_width: anchor.size.width,
        anchor_height: anchor.size.height,
    }
}

pub fn popper_desired_width(outer: Rect, anchor: Rect, min_width: Px) -> Px {
    Px(anchor.size.width.0.max(min_width.0).min(outer.size.width.0))
}

/// Compute Radix-like `size()` middleware metrics for a `PopperContentPlacement`.
///
/// Radix uses Floating UI's `size()` middleware to expose:
/// - `--radix-popper-available-width`
/// - `--radix-popper-available-height`
/// - `--radix-popper-anchor-width`
/// - `--radix-popper-anchor-height`
///
/// In Fret, we expose the same concepts as `PopperAvailableMetrics`. This helper computes them
/// without requiring callers to duplicate the "probe layout" step.
pub fn popper_available_metrics_for_placement(
    outer: Rect,
    anchor: Rect,
    min_width: Px,
    placement: PopperContentPlacement,
) -> PopperAvailableMetrics {
    let desired_w = popper_desired_width(outer, anchor, min_width);
    let probe_desired = Size::new(desired_w, outer.size.height);
    let layout = popper_content_layout_sized(outer, anchor, probe_desired, placement);
    // `size()` middleware metrics are computed against the *collision boundary*, i.e. after Radix
    // applies collision padding + boundary overrides. Match that by using the effective boundary
    // here (the placement solver already applies the same collision options when producing
    // `layout`).
    let mut boundary = outer;
    if let Some(extra_boundary) = placement.collision_boundary {
        boundary = intersect_rect(boundary, extra_boundary);
    }
    boundary = inset_rect(boundary, placement.collision_padding);
    popper_available_metrics(boundary, anchor, &layout, placement.direction)
}

/// Computes an anchored popper layout (rect + optional arrow) with deterministic flip/clamp rules.
pub fn popper_layout_sized(
    outer: Rect,
    anchor: Rect,
    desired: Size,
    side_offset: Px,
    side: Side,
    align: Align,
    options: AnchoredPanelOptions,
) -> AnchoredPanelLayout {
    anchored_panel_layout_sized_ex(outer, anchor, desired, side_offset, side, align, options)
}

fn opposite_side(side: Side) -> Side {
    match side {
        Side::Top => Side::Bottom,
        Side::Bottom => Side::Top,
        Side::Left => Side::Right,
        Side::Right => Side::Left,
    }
}

/// Computes a Radix-style transform origin for popper content animations.
///
/// Radix exposes this via a CSS variable (e.g. `--radix-tooltip-content-transform-origin`). We
/// approximate the same concept in a pure, geometry-driven way so component wrappers can scale
/// and/or slide from the edge that faces the anchor.
///
/// Returns a point in the same coordinate space as `layout.rect` (i.e. window/overlay coordinates).
pub fn popper_content_transform_origin(
    layout: &AnchoredPanelLayout,
    anchor: Rect,
    arrow_size: Option<Px>,
) -> Point {
    let rect = layout.rect;
    let anchor_center = Point::new(
        Px(anchor.origin.x.0 + anchor.size.width.0 * 0.5),
        Px(anchor.origin.y.0 + anchor.size.height.0 * 0.5),
    );

    let face = layout
        .arrow
        .map(|a| a.side)
        .unwrap_or_else(|| opposite_side(layout.side));

    let arrow_hidden = should_hide_arrow(layout);

    let (mut x, mut y) = match face {
        Side::Top => (Px(rect.size.width.0 * 0.5), Px(0.0)),
        Side::Bottom => (Px(rect.size.width.0 * 0.5), rect.size.height),
        Side::Left => (Px(0.0), Px(rect.size.height.0 * 0.5)),
        Side::Right => (rect.size.width, Px(rect.size.height.0 * 0.5)),
    };

    if let (Some(arrow), Some(arrow_size)) = (layout.arrow, arrow_size) {
        if !arrow_hidden {
            let cross_x = Px((arrow.offset.0 + arrow_size.0 * 0.5).clamp(0.0, rect.size.width.0));
            let cross_y = Px((arrow.offset.0 + arrow_size.0 * 0.5).clamp(0.0, rect.size.height.0));
            match face {
                Side::Top | Side::Bottom => x = cross_x,
                Side::Left | Side::Right => y = cross_y,
            }
        } else {
            // Radix hides the arrow when it can't be centered. When that happens, their
            // transform-origin math uses the placed alignment (`0%/50%/100%`) instead of the arrow
            // geometry.
            let align_x = match layout.align {
                Align::Start => Px(0.0),
                Align::Center => Px(rect.size.width.0 * 0.5),
                Align::End => rect.size.width,
            };
            let align_y = match layout.align {
                Align::Start => Px(0.0),
                Align::Center => Px(rect.size.height.0 * 0.5),
                Align::End => rect.size.height,
            };
            match face {
                Side::Top | Side::Bottom => x = align_x,
                Side::Left | Side::Right => y = align_y,
            }
        }
    } else {
        match face {
            Side::Top | Side::Bottom => {
                x = Px((anchor_center.x.0 - rect.origin.x.0).clamp(0.0, rect.size.width.0));
            }
            Side::Left | Side::Right => {
                y = Px((anchor_center.y.0 - rect.origin.y.0).clamp(0.0, rect.size.height.0));
            }
        }
    }

    Point::new(Px(rect.origin.x.0 + x.0), Px(rect.origin.y.0 + y.0))
}

pub fn should_hide_arrow(layout: &AnchoredPanelLayout) -> bool {
    layout
        .arrow
        .is_some_and(|arrow| arrow.center_offset.0.abs() > 0.01)
}

/// Default arrow protrusion used by shadcn/Radix-style diamonds.
///
/// A rotated square of side `s` has a half-diagonal of `s * sqrt(2) / 2 ≈ s * 0.707`.
/// We intentionally bias slightly higher for the common "diamond + border" look.
pub fn default_arrow_protrusion(arrow_size: Px) -> Px {
    Px(arrow_size.0 * 0.75)
}

/// Build Radix-style "diamond arrow" placement options.
///
/// Returns `(arrow_options, arrow_protrusion)`.
pub fn diamond_arrow_options(
    enabled: bool,
    arrow_size: Px,
    arrow_padding: Px,
) -> (Option<ArrowOptions>, Px) {
    if !enabled {
        return (None, Px(0.0));
    }

    (
        Some(ArrowOptions {
            size: Size::new(arrow_size, arrow_size),
            padding: Edges::all(arrow_padding),
        }),
        default_arrow_protrusion(arrow_size),
    )
}

/// Returns wrapper insets that keep the arrow hit-testable by expanding the overlay container.
///
/// This is useful when the overlay system uses the overlay root bounds for hit-testing
/// (outside-press / hover regions), and the arrow visually protrudes outside the panel rect.
pub fn wrapper_insets_for_arrow(layout: &AnchoredPanelLayout, protrusion: Px) -> Edges {
    if should_hide_arrow(layout) {
        return Edges::all(Px(0.0));
    }

    let Some(arrow) = layout.arrow else {
        return Edges::all(Px(0.0));
    };

    match arrow.side {
        Side::Top => Edges {
            top: protrusion,
            ..Edges::all(Px(0.0))
        },
        Side::Bottom => Edges {
            bottom: protrusion,
            ..Edges::all(Px(0.0))
        },
        Side::Left => Edges {
            left: protrusion,
            ..Edges::all(Px(0.0))
        },
        Side::Right => Edges {
            right: protrusion,
            ..Edges::all(Px(0.0))
        },
    }
}

#[cfg(test)]
mod tests {
    use fret_core::{Point, Rect, Size};

    use super::*;

    #[test]
    fn wrapper_insets_are_zero_without_arrow() {
        let layout = AnchoredPanelLayout {
            rect: Rect::new(Point::new(Px(0.0), Px(0.0)), Size::new(Px(10.0), Px(10.0))),
            side: Side::Bottom,
            align: Align::Center,
            arrow: None,
        };
        assert_eq!(
            wrapper_insets_for_arrow(&layout, Px(9.0)),
            Edges::all(Px(0.0))
        );
    }

    #[test]
    fn wrapper_insets_follow_arrow_side() {
        let mut layout = AnchoredPanelLayout {
            rect: Rect::new(Point::new(Px(0.0), Px(0.0)), Size::new(Px(10.0), Px(10.0))),
            side: Side::Bottom,
            align: Align::Center,
            arrow: Some(ArrowLayout {
                side: Side::Top,
                offset: Px(1.0),
                alignment_offset: Px(0.0),
                center_offset: Px(0.0),
            }),
        };

        assert_eq!(wrapper_insets_for_arrow(&layout, Px(7.0)).top, Px(7.0));

        layout.arrow = Some(ArrowLayout {
            side: Side::Left,
            offset: Px(1.0),
            alignment_offset: Px(0.0),
            center_offset: Px(0.0),
        });
        assert_eq!(wrapper_insets_for_arrow(&layout, Px(7.0)).left, Px(7.0));
    }

    #[test]
    fn wrapper_insets_are_zero_when_arrow_is_hidden() {
        let layout = AnchoredPanelLayout {
            rect: Rect::new(Point::new(Px(0.0), Px(0.0)), Size::new(Px(10.0), Px(10.0))),
            side: Side::Bottom,
            align: Align::Center,
            arrow: Some(ArrowLayout {
                side: Side::Top,
                offset: Px(1.0),
                alignment_offset: Px(0.0),
                center_offset: Px(10.0),
            }),
        };

        assert_eq!(
            wrapper_insets_for_arrow(&layout, Px(7.0)),
            Edges::all(Px(0.0))
        );
    }

    #[test]
    fn popper_layout_sized_returns_arrow_layout_when_configured() {
        let outer = Rect::new(
            Point::new(Px(0.0), Px(0.0)),
            Size::new(Px(200.0), Px(200.0)),
        );
        let anchor = Rect::new(
            Point::new(Px(50.0), Px(60.0)),
            Size::new(Px(40.0), Px(10.0)),
        );
        let desired = Size::new(Px(120.0), Px(80.0));

        let layout = popper_layout_sized(
            outer,
            anchor,
            desired,
            Px(8.0),
            Side::Bottom,
            Align::Center,
            AnchoredPanelOptions {
                direction: LayoutDirection::Ltr,
                offset: Offset::default(),
                shift: Default::default(),
                arrow: Some(ArrowOptions {
                    size: Size::new(Px(12.0), Px(12.0)),
                    padding: Edges::all(Px(8.0)),
                }),
                collision: Default::default(),
                sticky: Default::default(),
            },
        );

        let arrow = layout.arrow.expect("arrow layout");
        assert_eq!(arrow.side, Side::Top);
    }

    #[test]
    fn transform_origin_tracks_arrow_on_anchor_edge() {
        let outer = Rect::new(
            Point::new(Px(0.0), Px(0.0)),
            Size::new(Px(200.0), Px(200.0)),
        );
        let anchor = Rect::new(
            Point::new(Px(50.0), Px(60.0)),
            Size::new(Px(40.0), Px(10.0)),
        );
        let desired = Size::new(Px(120.0), Px(80.0));
        let arrow_size = Px(12.0);

        let layout = popper_layout_sized(
            outer,
            anchor,
            desired,
            Px(8.0),
            Side::Bottom,
            Align::Center,
            AnchoredPanelOptions {
                direction: LayoutDirection::Ltr,
                offset: Offset::default(),
                shift: Default::default(),
                arrow: Some(ArrowOptions {
                    size: Size::new(arrow_size, arrow_size),
                    padding: Edges::all(Px(8.0)),
                }),
                collision: Default::default(),
                sticky: Default::default(),
            },
        );

        let origin = popper_content_transform_origin(&layout, anchor, Some(arrow_size));
        let arrow = layout.arrow.expect("expected arrow layout");
        assert_eq!(origin.y, layout.rect.origin.y);
        assert_eq!(
            origin.x,
            Px(layout.rect.origin.x.0 + arrow.offset.0 + arrow_size.0 * 0.5)
        );
    }

    #[test]
    fn transform_origin_tracks_anchor_center_without_arrow() {
        let outer = Rect::new(
            Point::new(Px(0.0), Px(0.0)),
            Size::new(Px(200.0), Px(200.0)),
        );
        let anchor = Rect::new(
            Point::new(Px(50.0), Px(60.0)),
            Size::new(Px(40.0), Px(10.0)),
        );
        let desired = Size::new(Px(120.0), Px(80.0));

        let layout = popper_layout_sized(
            outer,
            anchor,
            desired,
            Px(8.0),
            Side::Bottom,
            Align::Center,
            AnchoredPanelOptions {
                direction: LayoutDirection::Ltr,
                offset: Offset::default(),
                shift: Default::default(),
                arrow: None,
                collision: Default::default(),
                sticky: Default::default(),
            },
        );

        let origin = popper_content_transform_origin(&layout, anchor, None);
        assert_eq!(origin.y, layout.rect.origin.y);

        let anchor_center_x = anchor.origin.x.0 + anchor.size.width.0 * 0.5;
        let x_in_panel =
            (anchor_center_x - layout.rect.origin.x.0).clamp(0.0, layout.rect.size.width.0);
        assert_eq!(origin.x, Px(layout.rect.origin.x.0 + x_in_panel));
    }

    #[test]
    fn transform_origin_uses_alignment_when_arrow_is_hidden() {
        let layout = AnchoredPanelLayout {
            rect: Rect::new(
                Point::new(Px(10.0), Px(20.0)),
                Size::new(Px(100.0), Px(50.0)),
            ),
            side: Side::Bottom,
            align: Align::End,
            arrow: Some(ArrowLayout {
                side: Side::Top,
                offset: Px(1.0),
                alignment_offset: Px(0.0),
                center_offset: Px(10.0),
            }),
        };

        let origin = popper_content_transform_origin(&layout, Rect::default(), Some(Px(12.0)));
        assert_eq!(origin.y, Px(20.0));
        assert_eq!(origin.x, Px(110.0));
    }

    #[test]
    fn popper_content_placement_passes_collision_padding_to_solver() {
        let outer = Rect::new(
            Point::new(Px(0.0), Px(0.0)),
            Size::new(Px(200.0), Px(100.0)),
        );
        let anchor = Rect::new(
            Point::new(Px(10.0), Px(40.0)),
            Size::new(Px(40.0), Px(10.0)),
        );
        let desired = Size::new(Px(120.0), Px(40.0));

        let layout = popper_content_layout_sized(
            outer,
            anchor,
            desired,
            PopperContentPlacement::new(LayoutDirection::Ltr, Side::Bottom, Align::Start, Px(0.0))
                .with_collision_padding(Edges {
                    bottom: Px(20.0),
                    ..Edges::all(Px(0.0))
                }),
        );

        assert_eq!(layout.side, Side::Top);
    }

    #[test]
    fn popper_content_reference_hidden_false_when_disabled() {
        let outer = Rect::new(
            Point::new(Px(0.0), Px(0.0)),
            Size::new(Px(100.0), Px(100.0)),
        );
        let anchor_outside = Rect::new(
            Point::new(Px(200.0), Px(200.0)),
            Size::new(Px(10.0), Px(10.0)),
        );
        let placement =
            PopperContentPlacement::new(LayoutDirection::Ltr, Side::Bottom, Align::Start, Px(0.0));

        assert!(!placement.reference_hidden(outer, anchor_outside));
    }

    #[test]
    fn popper_content_reference_hidden_true_when_anchor_outside_boundary() {
        let outer = Rect::new(
            Point::new(Px(0.0), Px(0.0)),
            Size::new(Px(100.0), Px(100.0)),
        );
        let anchor_outside = Rect::new(
            Point::new(Px(200.0), Px(200.0)),
            Size::new(Px(10.0), Px(10.0)),
        );
        let placement =
            PopperContentPlacement::new(LayoutDirection::Ltr, Side::Bottom, Align::Start, Px(0.0))
                .with_hide_when_detached(true);

        assert!(placement.reference_hidden(outer, anchor_outside));
    }

    #[test]
    fn available_metrics_track_anchor_and_available_space() {
        let outer = Rect::new(
            Point::new(Px(0.0), Px(0.0)),
            Size::new(Px(100.0), Px(100.0)),
        );
        let anchor = Rect::new(
            Point::new(Px(10.0), Px(20.0)),
            Size::new(Px(30.0), Px(40.0)),
        );
        let layout = AnchoredPanelLayout {
            rect: Rect::new(
                Point::new(Px(40.0), Px(40.0)),
                Size::new(Px(20.0), Px(10.0)),
            ),
            side: Side::Bottom,
            align: Align::Center,
            arrow: None,
        };

        let m = popper_available_metrics(outer, anchor, &layout, LayoutDirection::Ltr);
        assert_eq!(m.anchor_width, Px(30.0));
        assert_eq!(m.anchor_height, Px(40.0));
        assert_eq!(m.available_width, Px(100.0));
        assert_eq!(m.available_height, Px(60.0));
    }
}
