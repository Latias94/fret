//! Popper content skeleton helpers (Radix `@radix-ui/react-popper` outcomes).
//!
//! Radix Popper uses an extra wrapper layer to:
//! - position the floating content relative to an anchor
//! - expand hit-test bounds when an arrow protrudes outside the panel rect
//!
//! This module provides a small, reusable skeleton for the wrapper container so wrappers can
//! avoid duplicating the same absolute-layout boilerplate.

use fret_core::{Edges, Px, Rect};
use fret_ui::element::{
    AnyElement, ContainerProps, InsetStyle, LayoutStyle, Length, Overflow, PositionStyle, SizeStyle,
};
use fret_ui::{ElementContext, UiHost};

/// Render a popper wrapper container positioned at `placed` but expanded by `wrapper_insets`.
///
/// The wrapper uses `overflow: visible` so an arrow can protrude outside the panel rect while
/// remaining hit-testable by overlay systems that rely on the overlay root bounds.
pub fn popper_wrapper_at<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
    placed: Rect,
    wrapper_insets: Edges,
    f: impl FnOnce(&mut ElementContext<'_, H>) -> Vec<AnyElement>,
) -> AnyElement {
    let layout = LayoutStyle {
        position: PositionStyle::Absolute,
        inset: InsetStyle {
            left: Some(Px(placed.origin.x.0 - wrapper_insets.left.0)),
            top: Some(Px(placed.origin.y.0 - wrapper_insets.top.0)),
            ..Default::default()
        },
        size: SizeStyle {
            width: Length::Px(Px(placed.size.width.0
                + wrapper_insets.left.0
                + wrapper_insets.right.0)),
            height: Length::Px(Px(placed.size.height.0
                + wrapper_insets.top.0
                + wrapper_insets.bottom.0)),
            ..Default::default()
        },
        overflow: Overflow::Visible,
        ..Default::default()
    };

    cx.container(
        ContainerProps {
            layout,
            ..Default::default()
        },
        f,
    )
}
