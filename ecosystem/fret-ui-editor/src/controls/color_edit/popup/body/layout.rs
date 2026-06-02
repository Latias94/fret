use fret_core::{Axis, Px};
use fret_ui::element::{
    AnyElement, CrossAlign, FlexProps, LayoutStyle, Length, MainAlign, SizeStyle, SpacingLength,
};
use fret_ui::{ElementContext, UiHost};

use super::super::super::ColorEditPopupPicker;

const COLOR_POPUP_WIDTH: Px = Px(216.0);
const COLOR_POPUP_WITH_SIDE_PREVIEW_WIDTH: Px = Px(272.0);

pub(super) struct ColorPopupContentArgs {
    pub(super) picker: Option<AnyElement>,
    pub(super) side_preview: Option<AnyElement>,
    pub(super) picker_options: Option<AnyElement>,
    pub(super) eyedropper: Option<AnyElement>,
    pub(super) numbers: Option<AnyElement>,
    pub(super) history_swatches: Option<AnyElement>,
    pub(super) swatches: Option<AnyElement>,
    pub(super) standalone_alpha_bar: Option<AnyElement>,
}

pub(super) fn color_popup_width(picker: ColorEditPopupPicker, has_side_preview: bool) -> Px {
    if picker != ColorEditPopupPicker::Hidden && has_side_preview {
        COLOR_POPUP_WITH_SIDE_PREVIEW_WIDTH
    } else {
        COLOR_POPUP_WIDTH
    }
}

pub(super) fn color_popup_content<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
    args: ColorPopupContentArgs,
) -> AnyElement {
    let ColorPopupContentArgs {
        picker,
        side_preview,
        picker_options,
        eyedropper,
        numbers,
        history_swatches,
        swatches,
        standalone_alpha_bar,
    } = args;

    cx.flex(
        FlexProps {
            layout: LayoutStyle {
                size: SizeStyle {
                    width: Length::Fill,
                    height: Length::Auto,
                    ..Default::default()
                },
                ..Default::default()
            },
            direction: Axis::Vertical,
            gap: SpacingLength::Px(Px(8.0)),
            padding: Default::default(),
            justify: MainAlign::Start,
            align: CrossAlign::Stretch,
            wrap: false,
        },
        move |cx| {
            let mut out = Vec::new();
            match (picker, side_preview) {
                (Some(picker), Some(side_preview)) => {
                    out.push(picker_side_preview_row(cx, picker, side_preview));
                }
                (Some(picker), None) => out.push(picker),
                (None, Some(side_preview)) => out.push(side_preview),
                (None, None) => {}
            }
            if let Some(picker_options) = picker_options {
                out.push(picker_options);
            }
            if let Some(eyedropper) = eyedropper {
                out.push(eyedropper);
            }
            if let Some(numbers) = numbers {
                out.push(numbers);
            }
            if let Some(history_swatches) = history_swatches {
                out.push(history_swatches);
            }
            if let Some(swatches) = swatches {
                out.push(swatches);
            }
            if let Some(standalone_alpha_bar) = standalone_alpha_bar {
                out.push(standalone_alpha_bar);
            }
            out
        },
    )
}

fn picker_side_preview_row<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
    picker: AnyElement,
    side_preview: AnyElement,
) -> AnyElement {
    cx.flex(
        FlexProps {
            layout: LayoutStyle {
                size: SizeStyle {
                    width: Length::Fill,
                    height: Length::Auto,
                    ..Default::default()
                },
                ..Default::default()
            },
            direction: Axis::Horizontal,
            gap: SpacingLength::Px(Px(8.0)),
            padding: Default::default(),
            justify: MainAlign::Start,
            align: CrossAlign::Start,
            wrap: false,
        },
        move |_cx| vec![picker, side_preview],
    )
}
