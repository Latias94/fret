//! Popper arrow primitives (Radix-aligned outcomes).
//!
//! Upstream reference:
//! - `repo-ref/primitives/packages/react/popper/src/popper.tsx`
//!
//! Radix exposes "arrow" building blocks as part of the popper primitives and reuses them across
//! overlay-ish components (tooltip, popover, hover-card, select, menus).
//!
//! In Fret, placement math lives in `crate::primitives::popper`, while this module provides a
//! small renderer-agnostic element builder for the common "diamond" arrow shape.

use fret_core::{Color, Corners, Edges, Point, Px, Transform2D};
use fret_ui::element::{
    AnyElement, ContainerProps, InsetStyle, LayoutStyle, Length, Overflow, PositionStyle,
    SizeStyle, VisualTransformProps,
};
use fret_ui::overlay_placement::{AnchoredPanelLayout, Side};
use fret_ui::{ElementContext, UiHost};
use std::sync::Arc;

use crate::primitives::popper;

#[derive(Debug, Clone, Copy)]
pub struct DiamondArrowStyle {
    pub bg: Color,
    pub border: Option<Color>,
    pub border_width: Px,
}

impl DiamondArrowStyle {
    pub fn new(bg: Color) -> Self {
        Self {
            bg,
            border: None,
            border_width: Px(0.0),
        }
    }
}

/// Compute wrapper insets that reserve space for an arrow protrusion.
pub fn wrapper_insets(layout: &AnchoredPanelLayout, protrusion: Px) -> Edges {
    popper::wrapper_insets_for_arrow(layout, protrusion)
}

/// Render a diamond arrow element for a placed popper panel.
///
/// Returns `None` when the placement layout does not include arrow geometry.
#[track_caller]
pub fn diamond_arrow_element<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
    layout: &AnchoredPanelLayout,
    wrapper_insets: Edges,
    arrow_size: Px,
    style: DiamondArrowStyle,
) -> Option<AnyElement> {
    diamond_arrow_element_refined(
        cx,
        layout,
        wrapper_insets,
        arrow_size,
        style,
        Px(0.0),
        Px(0.0),
        None,
    )
}

pub fn diamond_arrow_element_refined<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
    layout: &AnchoredPanelLayout,
    wrapper_insets: Edges,
    arrow_size: Px,
    style: DiamondArrowStyle,
    corner_radius: Px,
    outset: Px,
    test_id: Option<Arc<str>>,
) -> Option<AnyElement> {
    if popper::should_hide_arrow(layout) {
        return None;
    }
    let arrow = layout.arrow?;
    let placed = layout.rect;

    let (left, top) = match arrow.side {
        Side::Top => (
            Px(wrapper_insets.left.0 + arrow.offset.0),
            Px(wrapper_insets.top.0 - arrow_size.0 * 0.5),
        ),
        Side::Bottom => (
            Px(wrapper_insets.left.0 + arrow.offset.0),
            Px(wrapper_insets.top.0 + placed.size.height.0 - arrow_size.0 * 0.5),
        ),
        Side::Left => (
            Px(wrapper_insets.left.0 - arrow_size.0 * 0.5),
            Px(wrapper_insets.top.0 + arrow.offset.0),
        ),
        Side::Right => (
            Px(wrapper_insets.left.0 + placed.size.width.0 - arrow_size.0 * 0.5),
            Px(wrapper_insets.top.0 + arrow.offset.0),
        ),
    };

    let (left, top) = match arrow.side {
        Side::Top => (left, Px(top.0 - outset.0)),
        Side::Bottom => (left, Px(top.0 + outset.0)),
        Side::Left => (Px(left.0 - outset.0), top),
        Side::Right => (Px(left.0 + outset.0), top),
    };

    let layout = LayoutStyle {
        position: PositionStyle::Absolute,
        inset: InsetStyle {
            left: Some(left),
            top: Some(top),
            ..Default::default()
        },
        size: SizeStyle {
            width: Length::Px(arrow_size),
            height: Length::Px(arrow_size),
            ..Default::default()
        },
        overflow: Overflow::Visible,
        ..Default::default()
    };

    let center = Point::new(Px(arrow_size.0 * 0.5), Px(arrow_size.0 * 0.5));
    let transform = Transform2D::rotation_about_degrees(45.0, center);

    let border_color = style.border;
    let test_id = test_id.clone();
    Some(
        cx.visual_transform_props(VisualTransformProps { layout, transform }, move |cx| {
            let container = cx.container(
                ContainerProps {
                    layout: LayoutStyle {
                        size: SizeStyle {
                            width: Length::Fill,
                            height: Length::Fill,
                            ..Default::default()
                        },
                        ..Default::default()
                    },
                    padding: Edges::all(Px(0.0)),
                    background: Some(style.bg),
                    background_paint: None,
                    shadow: None,
                    border: Edges::all(style.border_width),
                    border_color,
                    border_paint: None,
                    focus_ring: None,
                    focus_border_color: None,
                    focus_within: false,
                    corner_radii: Corners::all(corner_radius),
                    snap_to_device_pixels: false,
                },
                |_cx| Vec::new(),
            );

            if let Some(test_id) = &test_id {
                vec![container.test_id(test_id.clone())]
            } else {
                vec![container]
            }
        }),
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    use fret_app::App;
    use fret_core::{Point, Rect, Size};
    use fret_ui::overlay_placement::{Align, ArrowLayout, Side};

    #[test]
    fn diamond_arrow_element_returns_none_without_arrow_geometry() {
        let mut app = App::new();
        fret_ui::elements::with_element_cx(
            &mut app,
            Default::default(),
            Default::default(),
            "test",
            |cx| {
                let layout = AnchoredPanelLayout {
                    rect: Rect::new(Point::new(Px(0.0), Px(0.0)), Size::new(Px(10.0), Px(10.0))),
                    side: Side::Bottom,
                    align: Align::Center,
                    arrow: None,
                };
                let el = diamond_arrow_element(
                    cx,
                    &layout,
                    Edges::all(Px(0.0)),
                    Px(10.0),
                    DiamondArrowStyle::new(fret_core::Color {
                        r: 0.0,
                        g: 0.0,
                        b: 0.0,
                        a: 1.0,
                    }),
                );
                assert!(el.is_none());
            },
        );
    }

    #[test]
    fn diamond_arrow_element_renders_when_layout_has_arrow() {
        let mut app = App::new();
        fret_ui::elements::with_element_cx(
            &mut app,
            Default::default(),
            Default::default(),
            "test",
            |cx| {
                let layout = AnchoredPanelLayout {
                    rect: Rect::new(Point::new(Px(0.0), Px(0.0)), Size::new(Px(10.0), Px(10.0))),
                    side: Side::Bottom,
                    align: Align::Center,
                    arrow: Some(ArrowLayout {
                        side: Side::Bottom,
                        offset: Px(1.0),
                        alignment_offset: Px(0.0),
                        center_offset: Px(0.0),
                    }),
                };
                let el = diamond_arrow_element(
                    cx,
                    &layout,
                    Edges::all(Px(0.0)),
                    Px(10.0),
                    DiamondArrowStyle::new(fret_core::Color {
                        r: 0.0,
                        g: 0.0,
                        b: 0.0,
                        a: 1.0,
                    }),
                );
                assert!(el.is_some());
            },
        );
    }

    #[test]
    fn diamond_arrow_element_hides_when_arrow_cannot_center() {
        let mut app = App::new();
        fret_ui::elements::with_element_cx(
            &mut app,
            Default::default(),
            Default::default(),
            "test",
            |cx| {
                let layout = AnchoredPanelLayout {
                    rect: Rect::new(Point::new(Px(0.0), Px(0.0)), Size::new(Px(10.0), Px(10.0))),
                    side: Side::Bottom,
                    align: Align::Center,
                    arrow: Some(ArrowLayout {
                        side: Side::Bottom,
                        offset: Px(1.0),
                        alignment_offset: Px(0.0),
                        center_offset: Px(10.0),
                    }),
                };
                let el = diamond_arrow_element(
                    cx,
                    &layout,
                    Edges::all(Px(0.0)),
                    Px(10.0),
                    DiamondArrowStyle::new(fret_core::Color {
                        r: 0.0,
                        g: 0.0,
                        b: 0.0,
                        a: 1.0,
                    }),
                );
                assert!(el.is_none());
            },
        );
    }
}
