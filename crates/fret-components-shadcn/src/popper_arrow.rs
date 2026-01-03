//! Internal helpers for Radix-style popper arrows (diamond arrows).
//!
//! This module intentionally lives in the shadcn facade crate:
//! - placement math is owned by `fret-ui-kit::primitives::popper` (pure + testable)
//! - arrow rendering is a styling concern and is reused across shadcn overlays

use fret_ui_kit::primitives::popper;
use fret_core::{Color, Corners, Edges, Point, Px, Transform2D};
use fret_ui::element::{
    AnyElement, ContainerProps, InsetStyle, LayoutStyle, Length, Overflow, PositionStyle,
    SizeStyle, VisualTransformProps,
};
use fret_ui::overlay_placement::{AnchoredPanelLayout, Side};
use fret_ui::{ElementContext, UiHost};

#[derive(Debug, Clone, Copy)]
pub(crate) struct DiamondArrowStyle {
    pub bg: Color,
    pub border: Option<Color>,
    pub border_width: Px,
}

pub(crate) fn wrapper_insets(layout: &AnchoredPanelLayout, protrusion: Px) -> Edges {
    popper::wrapper_insets_for_arrow(layout, protrusion)
}

pub(crate) fn diamond_arrow_element<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
    layout: &AnchoredPanelLayout,
    wrapper_insets: Edges,
    arrow_size: Px,
    style: DiamondArrowStyle,
) -> Option<AnyElement> {
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
    Some(
        cx.visual_transform_props(VisualTransformProps { layout, transform }, move |cx| {
            vec![cx.container(
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
                    shadow: None,
                    border: Edges::all(style.border_width),
                    border_color,
                    corner_radii: Corners::all(Px(0.0)),
                },
                |_cx| Vec::new(),
            )]
        }),
    )
}
