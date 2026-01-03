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

/// Returns the wrapper layout used by [`popper_wrapper_at`].
pub fn popper_wrapper_layout(placed: Rect, wrapper_insets: Edges) -> LayoutStyle {
    LayoutStyle {
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
    }
}

/// Returns the inner panel layout used inside a popper wrapper.
///
/// The returned layout is positioned relative to the wrapper origin (which is already expanded by
/// `wrapper_insets`), so the panel's top-left starts at `(wrapper_insets.left, wrapper_insets.top)`.
pub fn popper_panel_layout(placed: Rect, wrapper_insets: Edges, overflow: Overflow) -> LayoutStyle {
    LayoutStyle {
        position: PositionStyle::Absolute,
        inset: InsetStyle {
            left: Some(wrapper_insets.left),
            top: Some(wrapper_insets.top),
            ..Default::default()
        },
        size: SizeStyle {
            width: Length::Px(placed.size.width),
            height: Length::Px(placed.size.height),
            ..Default::default()
        },
        overflow,
        ..Default::default()
    }
}

/// Render a popper inner panel container inside the wrapper.
#[track_caller]
pub fn popper_panel_at<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
    placed: Rect,
    wrapper_insets: Edges,
    overflow: Overflow,
    f: impl FnOnce(&mut ElementContext<'_, H>) -> Vec<AnyElement>,
) -> AnyElement {
    cx.container(
        ContainerProps {
            layout: popper_panel_layout(placed, wrapper_insets, overflow),
            ..Default::default()
        },
        f,
    )
}

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
    cx.container(
        ContainerProps {
            layout: popper_wrapper_layout(placed, wrapper_insets),
            ..Default::default()
        },
        f,
    )
}
