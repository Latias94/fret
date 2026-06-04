use fret_core::{Axis, Color, Corners, Edges, Px};
use fret_ui::element::{
    AnyElement, ContainerProps, CrossAlign, FlexItemStyle, FlexProps, LayoutStyle, Length,
    MainAlign, SizeStyle, SpacingLength,
};
use fret_ui::{ElementContext, UiHost};

use super::super::super::super::preview::fill_preview_layout;
use super::super::super::horizontal_bar_thumb_spacer;

pub(super) fn horizontal_bar_thumb_overlay<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
    position: f32,
) -> AnyElement {
    let left_grow = position.clamp(0.0, 1.0);
    let right_grow = (1.0 - left_grow).max(0.0);
    cx.flex(
        FlexProps {
            layout: fill_preview_layout(),
            direction: Axis::Horizontal,
            gap: SpacingLength::Px(Px(0.0)),
            padding: Edges::all(Px(0.0)).into(),
            justify: MainAlign::Start,
            align: CrossAlign::Stretch,
            wrap: false,
        },
        move |cx| {
            vec![
                horizontal_bar_thumb_spacer(cx, left_grow),
                horizontal_thumb_marker(cx),
                horizontal_bar_thumb_spacer(cx, right_grow),
            ]
        },
    )
}

pub(super) fn vertical_bar_thumb_overlay<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
    position: f32,
) -> AnyElement {
    let top_grow = position.clamp(0.0, 1.0);
    let bottom_grow = (1.0 - top_grow).max(0.0);
    cx.flex(
        FlexProps {
            layout: fill_preview_layout(),
            direction: Axis::Vertical,
            gap: SpacingLength::Px(Px(0.0)),
            padding: Edges::all(Px(0.0)).into(),
            justify: MainAlign::Start,
            align: CrossAlign::Stretch,
            wrap: false,
        },
        move |cx| {
            vec![
                vertical_bar_thumb_spacer(cx, top_grow),
                vertical_thumb_marker(cx),
                vertical_bar_thumb_spacer(cx, bottom_grow),
            ]
        },
    )
}

fn horizontal_thumb_marker<H: UiHost>(cx: &mut ElementContext<'_, H>) -> AnyElement {
    cx.container(
        ContainerProps {
            layout: LayoutStyle {
                size: SizeStyle {
                    width: Length::Px(Px(3.0)),
                    height: Length::Fill,
                    ..Default::default()
                },
                flex: FlexItemStyle {
                    grow: 0.0,
                    shrink: 0.0,
                    basis: Length::Px(Px(3.0)),
                    ..Default::default()
                },
                ..Default::default()
            },
            background: Some(Color::from_srgb_hex_rgb(0xff_ff_ff)),
            border: Edges::all(Px(1.0)),
            border_color: Some(Color::from_srgb_hex_rgb(0x1f_29_37)),
            corner_radii: Corners::all(Px(2.0)),
            ..Default::default()
        },
        |_cx| vec![],
    )
}

fn vertical_thumb_marker<H: UiHost>(cx: &mut ElementContext<'_, H>) -> AnyElement {
    cx.container(
        ContainerProps {
            layout: LayoutStyle {
                size: SizeStyle {
                    width: Length::Fill,
                    height: Length::Px(Px(3.0)),
                    ..Default::default()
                },
                flex: FlexItemStyle {
                    grow: 0.0,
                    shrink: 0.0,
                    basis: Length::Px(Px(3.0)),
                    ..Default::default()
                },
                ..Default::default()
            },
            background: Some(Color::from_srgb_hex_rgb(0xff_ff_ff)),
            border: Edges::all(Px(1.0)),
            border_color: Some(Color::from_srgb_hex_rgb(0x1f_29_37)),
            corner_radii: Corners::all(Px(2.0)),
            ..Default::default()
        },
        |_cx| vec![],
    )
}

fn vertical_bar_thumb_spacer<H: UiHost>(cx: &mut ElementContext<'_, H>, grow: f32) -> AnyElement {
    cx.container(
        ContainerProps {
            layout: LayoutStyle {
                size: SizeStyle {
                    width: Length::Fill,
                    height: Length::Auto,
                    ..Default::default()
                },
                flex: FlexItemStyle {
                    grow,
                    shrink: 1.0,
                    basis: Length::Px(Px(0.0)),
                    ..Default::default()
                },
                ..Default::default()
            },
            ..Default::default()
        },
        |_cx| vec![],
    )
}
