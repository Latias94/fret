//! Popper content skeleton helpers (Radix `@radix-ui/react-popper` outcomes).
//!
//! Radix Popper uses an extra wrapper layer to:
//! - position the floating content relative to an anchor
//! - expand hit-test bounds when an arrow protrudes outside the panel rect
//!
//! This module provides a small, reusable skeleton for the wrapper container so wrappers can
//! avoid duplicating the same absolute-layout boilerplate.

use fret_core::{Edges, Point, Px, Rect};
use fret_ui::element::{
    AnyElement, ContainerProps, HoverRegionProps, InsetStyle, LayoutStyle, Length, Overflow,
    PositionStyle, SizeStyle,
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

/// Returns an autosizing wrapper layout positioned at `origin`.
///
/// Unlike [`popper_wrapper_layout`], this layout does **not** force the wrapper size to match an
/// externally computed `placed.size`. This is useful for primitives whose placement does not
/// require knowing the floating panel size up-front (e.g. bottom-start anchors), allowing the
/// wrapper bounds to be determined by its children (intrinsic sizing).
pub fn popper_wrapper_layout_autosize(origin: Point) -> LayoutStyle {
    LayoutStyle {
        position: PositionStyle::Absolute,
        inset: InsetStyle {
            left: Some(origin.x),
            top: Some(origin.y),
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

/// Render an autosizing popper wrapper positioned at `origin`.
///
/// This wrapper uses `overflow: visible` and relies on its children to determine its size.
pub fn popper_wrapper_at_autosize<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
    origin: Point,
    f: impl FnOnce(&mut ElementContext<'_, H>) -> Vec<AnyElement>,
) -> AnyElement {
    cx.container(
        ContainerProps {
            layout: popper_wrapper_layout_autosize(origin),
            ..Default::default()
        },
        f,
    )
}

/// Render a popper wrapper that contains a panel, and optionally returns additional wrapper
/// children (like an arrow element) alongside the panel.
///
/// This keeps the panel element *scoped under the wrapper* (correct identity), while still
/// allowing the wrapper to render siblings like arrows.
#[track_caller]
pub fn popper_wrapper_at_with_panel<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
    placed: Rect,
    wrapper_insets: Edges,
    overflow: Overflow,
    panel: impl FnOnce(&mut ElementContext<'_, H>) -> Vec<AnyElement>,
    wrapper_children: impl FnOnce(&mut ElementContext<'_, H>, AnyElement) -> Vec<AnyElement>,
) -> AnyElement {
    popper_wrapper_at(cx, placed, wrapper_insets, |cx| {
        let panel = popper_panel_at(cx, placed, wrapper_insets, overflow, panel);
        wrapper_children(cx, panel)
    })
}

/// Render a popper wrapper as a hover region, nesting the panel under the wrapper while still
/// allowing wrapper siblings (like arrows) alongside the panel.
///
/// This is useful for hover-driven overlays (`Tooltip` / `HoverCard`) that need a hit-testable
/// wrapper node with expanded bounds (arrow protrusion) while still keeping the panel element
/// scoped beneath it (correct identity).
#[track_caller]
pub fn popper_hover_region_at_with_panel<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
    placed: Rect,
    wrapper_insets: Edges,
    overflow: Overflow,
    panel: impl FnOnce(&mut ElementContext<'_, H>) -> Vec<AnyElement>,
    wrapper_children: impl FnOnce(&mut ElementContext<'_, H>, bool, AnyElement) -> Vec<AnyElement>,
) -> AnyElement {
    cx.hover_region(
        HoverRegionProps {
            layout: popper_wrapper_layout(placed, wrapper_insets),
        },
        move |cx, hovered| {
            let panel = popper_panel_at(cx, placed, wrapper_insets, overflow, panel);
            wrapper_children(cx, hovered, panel)
        },
    )
}

/// Render a Radix-style popper wrapper + inner panel in one helper.
///
/// This avoids repeating the common pattern:
/// - wrapper absolute positioning + hit-test expansion (arrow insets),
/// - panel absolute positioning inside the wrapper.
#[track_caller]
pub fn popper_wrapper_panel_at<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
    placed: Rect,
    wrapper_insets: Edges,
    overflow: Overflow,
    f: impl FnOnce(&mut ElementContext<'_, H>) -> Vec<AnyElement>,
) -> AnyElement {
    popper_wrapper_at_with_panel(cx, placed, wrapper_insets, overflow, f, |_cx, panel| {
        vec![panel]
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use fret_core::{Point, Px, Size};
    use fret_ui::element::{Length, PositionStyle};

    #[test]
    fn wrapper_layout_expands_by_insets_and_offsets_origin() {
        let placed = Rect::new(
            Point::new(Px(10.0), Px(20.0)),
            Size::new(Px(30.0), Px(40.0)),
        );
        let insets = Edges {
            top: Px(1.0),
            right: Px(2.0),
            bottom: Px(3.0),
            left: Px(4.0),
        };

        let layout = popper_wrapper_layout(placed, insets);
        assert_eq!(layout.position, PositionStyle::Absolute);
        assert_eq!(layout.inset.left, Some(Px(6.0)));
        assert_eq!(layout.inset.top, Some(Px(19.0)));
        match layout.size.width {
            Length::Px(px) => assert_eq!(px, Px(36.0)),
            _ => panic!("expected px width"),
        }
        match layout.size.height {
            Length::Px(px) => assert_eq!(px, Px(44.0)),
            _ => panic!("expected px height"),
        }
    }

    #[test]
    fn panel_layout_starts_at_insets_and_uses_placed_size() {
        let placed = Rect::new(
            Point::new(Px(10.0), Px(20.0)),
            Size::new(Px(30.0), Px(40.0)),
        );
        let insets = Edges {
            top: Px(1.0),
            right: Px(2.0),
            bottom: Px(3.0),
            left: Px(4.0),
        };

        let layout = popper_panel_layout(placed, insets, Overflow::Clip);
        assert_eq!(layout.position, PositionStyle::Absolute);
        assert_eq!(layout.inset.left, Some(Px(4.0)));
        assert_eq!(layout.inset.top, Some(Px(1.0)));
        match layout.size.width {
            Length::Px(px) => assert_eq!(px, Px(30.0)),
            _ => panic!("expected px width"),
        }
        match layout.size.height {
            Length::Px(px) => assert_eq!(px, Px(40.0)),
            _ => panic!("expected px height"),
        }
        assert_eq!(layout.overflow, Overflow::Clip);
    }

    #[test]
    fn wrapper_panel_helper_nests_panel_inside_wrapper() {
        let mut app = fret_app::App::new();
        let window = fret_core::AppWindowId::default();
        let bounds = Rect::new(
            Point::new(Px(0.0), Px(0.0)),
            Size::new(Px(200.0), Px(120.0)),
        );

        let placed = Rect::new(
            Point::new(Px(10.0), Px(20.0)),
            Size::new(Px(30.0), Px(40.0)),
        );
        let insets = Edges::all(Px(5.0));

        let wrapper = fret_ui::elements::with_element_cx(&mut app, window, bounds, "test", |cx| {
            popper_wrapper_panel_at(cx, placed, insets, Overflow::Clip, |_cx| Vec::new())
        });

        assert_eq!(wrapper.children.len(), 1);
        let panel = &wrapper.children[0];
        assert!(panel.children.is_empty());
        assert_ne!(wrapper.id, panel.id);
    }
}
