use fret_core::{Axis, Corners, Edges, Px};
use fret_ui::element::{
    AnyElement, ContainerProps, CrossAlign, FlexProps, LayoutStyle, Length, MainAlign, Overflow,
    SizeStyle, SpacingLength,
};
use fret_ui::{ElementContext, UiHost};

use super::super::super::ColorEditPopupPicker;
use super::super::super::model::{HsvColor, sanitize_hue};
use super::super::picker::{hue_bar_preview_stack, hue_wheel_canvas, sv_picker_preview_stack};

const PICKER_OPTION_THUMBNAIL_WIDTH: Px = Px(64.0);
pub(super) const PICKER_OPTION_THUMBNAIL_HEIGHT: Px = Px(44.0);

pub(super) fn picker_option_thumbnail<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
    picker: ColorEditPopupPicker,
    hsv: HsvColor,
) -> AnyElement {
    match picker {
        ColorEditPopupPicker::HsvHueBar => hue_bar_picker_thumbnail(cx, hsv),
        ColorEditPopupPicker::HsvHueWheel => hue_wheel_picker_thumbnail(cx, hsv),
        ColorEditPopupPicker::Hidden => cx.spacer(Default::default()),
    }
}

fn hue_bar_picker_thumbnail<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
    hsv: HsvColor,
) -> AnyElement {
    cx.flex(
        FlexProps {
            layout: LayoutStyle {
                size: SizeStyle {
                    width: Length::Px(PICKER_OPTION_THUMBNAIL_WIDTH),
                    height: Length::Px(PICKER_OPTION_THUMBNAIL_HEIGHT),
                    ..Default::default()
                },
                ..Default::default()
            },
            direction: Axis::Horizontal,
            gap: SpacingLength::Px(Px(3.0)),
            padding: Edges::all(Px(0.0)).into(),
            justify: MainAlign::Center,
            align: CrossAlign::Stretch,
            wrap: false,
        },
        move |cx| {
            vec![
                picker_thumbnail_clip(cx, Length::Px(Px(50.0)), move |cx| {
                    vec![sv_picker_preview_stack(cx, hsv)]
                }),
                picker_thumbnail_clip(cx, Length::Px(Px(8.0)), move |cx| {
                    vec![hue_bar_preview_stack(cx, sanitize_hue(hsv.hue))]
                }),
            ]
        },
    )
}

fn hue_wheel_picker_thumbnail<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
    hsv: HsvColor,
) -> AnyElement {
    picker_thumbnail_clip(cx, Length::Px(PICKER_OPTION_THUMBNAIL_WIDTH), move |cx| {
        vec![hue_wheel_canvas(cx, hsv)]
    })
}

fn picker_thumbnail_clip<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
    width: Length,
    f: impl FnOnce(&mut ElementContext<'_, H>) -> Vec<AnyElement> + 'static,
) -> AnyElement {
    cx.container(
        ContainerProps {
            layout: LayoutStyle {
                size: SizeStyle {
                    width,
                    height: Length::Px(PICKER_OPTION_THUMBNAIL_HEIGHT),
                    ..Default::default()
                },
                overflow: Overflow::Clip,
                ..Default::default()
            },
            corner_radii: Corners::all(Px(4.0)),
            ..Default::default()
        },
        f,
    )
}
