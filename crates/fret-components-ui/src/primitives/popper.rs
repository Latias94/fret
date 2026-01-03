//! Popper / floating placement helpers (Radix `@radix-ui/react-popper` outcomes).
//!
//! This primitive is a thin, stable wrapper around `fret-ui`'s deterministic placement solver
//! (`fret_ui::overlay_placement`). It is intentionally pure and testable.

use fret_core::{Edges, Px, Rect, Size};
use fret_ui::overlay_placement::{
    AnchoredPanelLayout, AnchoredPanelOptions, Side, anchored_panel_layout_sized_ex,
};

pub use fret_ui::overlay_placement::{Align, ArrowLayout, ArrowOptions, LayoutDirection, Offset};

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
            alignment_axis: None,
        },
        arrow,
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

    pub fn options(self) -> AnchoredPanelOptions {
        anchored_panel_options_for_popper_content(
            self.direction,
            self.arrow_protrusion,
            self.align_offset,
            self.arrow,
        )
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

/// Default arrow protrusion used by shadcn/Radix-style diamonds.
///
/// A rotated square of side `s` has a half-diagonal of `s * sqrt(2) / 2 ≈ s * 0.707`.
/// We intentionally bias slightly higher for the common "diamond + border" look.
pub fn default_arrow_protrusion(arrow_size: Px) -> Px {
    Px(arrow_size.0 * 0.75)
}

/// Returns wrapper insets that keep the arrow hit-testable by expanding the overlay container.
///
/// This is useful when the overlay system uses the overlay root bounds for hit-testing
/// (outside-press / hover regions), and the arrow visually protrudes outside the panel rect.
pub fn wrapper_insets_for_arrow(layout: &AnchoredPanelLayout, protrusion: Px) -> Edges {
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
            }),
        };

        assert_eq!(wrapper_insets_for_arrow(&layout, Px(7.0)).top, Px(7.0));

        layout.arrow = Some(ArrowLayout {
            side: Side::Left,
            offset: Px(1.0),
            alignment_offset: Px(0.0),
        });
        assert_eq!(wrapper_insets_for_arrow(&layout, Px(7.0)).left, Px(7.0));
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
                arrow: Some(ArrowOptions {
                    size: Size::new(Px(12.0), Px(12.0)),
                    padding: Edges::all(Px(8.0)),
                }),
            },
        );

        let arrow = layout.arrow.expect("arrow layout");
        assert_eq!(arrow.side, Side::Top);
    }
}
