use fret_core::{Axis, Color, Corners, Edges, Px};
use fret_ui::element::{
    AnyElement, ContainerProps, CrossAlign, FlexItemStyle, FlexProps, GridProps, GridTrackSizing,
    LayoutStyle, Length, MainAlign, SizeStyle, SpacingLength, StackProps,
};
use fret_ui::{ElementContext, UiHost};

use super::super::super::super::SV_PICKER_STEPS;
use super::super::super::super::model::{HsvColor, hsv_to_color_preserving_alpha, unit_from_step};
use super::super::super::preview::fill_preview_layout;
use super::super::horizontal_bar_thumb_spacer;

pub(in crate::controls::color_edit::popup) fn sv_picker_preview_stack<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
    hsv: HsvColor,
) -> AnyElement {
    cx.stack_props(
        StackProps {
            layout: fill_preview_layout(),
        },
        move |cx| {
            vec![
                sv_picker_grid(cx, hsv.hue),
                sv_picker_thumb_overlay(cx, hsv.saturation, hsv.value),
            ]
        },
    )
}

fn sv_picker_grid<H: UiHost>(cx: &mut ElementContext<'_, H>, hue: f32) -> AnyElement {
    cx.grid(
        GridProps {
            layout: fill_preview_layout(),
            cols: SV_PICKER_STEPS as u16,
            rows: Some(SV_PICKER_STEPS as u16),
            template_columns: Some(
                (0..SV_PICKER_STEPS)
                    .map(|_| GridTrackSizing::Flex(1.0))
                    .collect(),
            ),
            template_rows: Some(
                (0..SV_PICKER_STEPS)
                    .map(|_| GridTrackSizing::Flex(1.0))
                    .collect(),
            ),
            gap: SpacingLength::Px(Px(0.0)),
            padding: Edges::all(Px(0.0)).into(),
            ..Default::default()
        },
        move |cx| {
            (0..SV_PICKER_STEPS * SV_PICKER_STEPS)
                .map(|idx| {
                    let row = idx / SV_PICKER_STEPS;
                    let col = idx % SV_PICKER_STEPS;
                    let saturation = unit_from_step(col, SV_PICKER_STEPS);
                    let value = 1.0 - unit_from_step(row, SV_PICKER_STEPS);
                    cx.container(
                        ContainerProps {
                            layout: fill_preview_layout(),
                            background: Some(hsv_to_color_preserving_alpha(
                                HsvColor {
                                    hue,
                                    saturation,
                                    value,
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

fn sv_picker_thumb_overlay<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
    saturation: f32,
    value: f32,
) -> AnyElement {
    let left_grow = saturation.clamp(0.0, 1.0);
    let right_grow = (1.0 - left_grow).max(0.0);
    let top_grow = (1.0 - value.clamp(0.0, 1.0)).max(0.0);
    let bottom_grow = value.clamp(0.0, 1.0);

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
                sv_thumb_vertical_spacer(cx, top_grow),
                cx.flex(
                    FlexProps {
                        layout: LayoutStyle {
                            size: SizeStyle {
                                width: Length::Fill,
                                height: Length::Px(Px(9.0)),
                                ..Default::default()
                            },
                            flex: FlexItemStyle {
                                grow: 0.0,
                                shrink: 0.0,
                                basis: Length::Px(Px(9.0)),
                                ..Default::default()
                            },
                            ..Default::default()
                        },
                        direction: Axis::Horizontal,
                        gap: SpacingLength::Px(Px(0.0)),
                        padding: Edges::all(Px(0.0)).into(),
                        justify: MainAlign::Start,
                        align: CrossAlign::Center,
                        wrap: false,
                    },
                    move |cx| {
                        vec![
                            horizontal_bar_thumb_spacer(cx, left_grow),
                            cx.container(
                                ContainerProps {
                                    layout: LayoutStyle {
                                        size: SizeStyle {
                                            width: Length::Px(Px(9.0)),
                                            height: Length::Px(Px(9.0)),
                                            ..Default::default()
                                        },
                                        flex: FlexItemStyle {
                                            grow: 0.0,
                                            shrink: 0.0,
                                            basis: Length::Px(Px(9.0)),
                                            ..Default::default()
                                        },
                                        ..Default::default()
                                    },
                                    background: Some(Color::TRANSPARENT),
                                    border: Edges::all(Px(2.0)),
                                    border_color: Some(Color::from_srgb_hex_rgb(0xff_ff_ff)),
                                    corner_radii: Corners::all(Px(10.0)),
                                    ..Default::default()
                                },
                                |_cx| vec![],
                            ),
                            horizontal_bar_thumb_spacer(cx, right_grow),
                        ]
                    },
                ),
                sv_thumb_vertical_spacer(cx, bottom_grow),
            ]
        },
    )
}

fn sv_thumb_vertical_spacer<H: UiHost>(cx: &mut ElementContext<'_, H>, grow: f32) -> AnyElement {
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
