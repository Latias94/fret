use fret_core::{Axis, Color, Corners, Edges, Px};
use fret_ui::element::{
    AnyElement, ContainerProps, CrossAlign, FlexItemStyle, FlexProps, GridProps, GridTrackSizing,
    LayoutStyle, Length, MainAlign, SizeStyle, SpacingLength, StackProps,
};
use fret_ui::{ElementContext, UiHost};

use super::super::super::super::HUE_BAR_STEPS;
use super::super::super::super::model::{HsvColor, hsv_to_color_preserving_alpha};
use super::super::super::preview::fill_preview_layout;

pub(in crate::controls::color_edit::popup) fn hue_bar_preview_stack<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
    hue: f32,
) -> AnyElement {
    cx.stack_props(
        StackProps {
            layout: fill_preview_layout(),
        },
        move |cx| {
            vec![
                vertical_hue_gradient_overlay(cx),
                vertical_bar_thumb_overlay(cx, hue),
            ]
        },
    )
}

fn vertical_hue_gradient_overlay<H: UiHost>(cx: &mut ElementContext<'_, H>) -> AnyElement {
    cx.grid(
        GridProps {
            layout: fill_preview_layout(),
            cols: 1,
            rows: Some(HUE_BAR_STEPS as u16),
            template_columns: Some(vec![GridTrackSizing::Flex(1.0)]),
            template_rows: Some(
                (0..HUE_BAR_STEPS)
                    .map(|_| GridTrackSizing::Flex(1.0))
                    .collect(),
            ),
            gap: SpacingLength::Px(Px(0.0)),
            padding: Edges::all(Px(0.0)).into(),
            ..Default::default()
        },
        |cx| {
            (0..HUE_BAR_STEPS)
                .map(|idx| {
                    let hue = idx as f32 / HUE_BAR_STEPS as f32;
                    cx.container(
                        ContainerProps {
                            layout: fill_preview_layout(),
                            background: Some(hsv_to_color_preserving_alpha(
                                HsvColor {
                                    hue,
                                    saturation: 1.0,
                                    value: 1.0,
                                },
                                1.0,
                            )),
                            ..Default::default()
                        },
                        |_cx| vec![],
                    )
                })
                .collect::<Vec<_>>()
        },
    )
}

fn vertical_bar_thumb_overlay<H: UiHost>(
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
                ),
                vertical_bar_thumb_spacer(cx, bottom_grow),
            ]
        },
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
