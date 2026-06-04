use std::sync::Arc;

use fret_core::{Axis, Color, Edges, Px};
use fret_runtime::Model;
use fret_ui::element::{
    AnyElement, CrossAlign, FlexProps, LayoutStyle, Length, MainAlign, SizeStyle, SpacingLength,
};
use fret_ui::{ElementContext, UiHost};

use crate::primitives::input_group::derived_test_id;

use super::super::super::{ColorEditPopupPicker, ColorEditPopupRuntimeOptions};

mod card;

use card::picker_option_button;

pub(super) fn picker_options_row<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
    current: Color,
    runtime_options: ColorEditPopupRuntimeOptions,
    runtime_model: Model<ColorEditPopupRuntimeOptions>,
    enabled: bool,
    row_height: Px,
    test_id: Option<Arc<str>>,
) -> AnyElement {
    let hue_bar_test_id = derived_test_id(test_id.as_ref(), "hue-bar");
    let hue_wheel_test_id = derived_test_id(test_id.as_ref(), "hue-wheel");
    let mut row = cx.flex(
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
            gap: SpacingLength::Px(Px(4.0)),
            padding: Edges::all(Px(0.0)).into(),
            justify: MainAlign::Start,
            align: CrossAlign::Center,
            wrap: false,
        },
        move |cx| {
            vec![
                picker_option_button(
                    cx,
                    "Hue Bar",
                    ColorEditPopupPicker::HsvHueBar,
                    current,
                    runtime_options.picker == ColorEditPopupPicker::HsvHueBar,
                    runtime_model.clone(),
                    enabled,
                    row_height,
                    hue_bar_test_id.clone(),
                ),
                picker_option_button(
                    cx,
                    "Hue Wheel",
                    ColorEditPopupPicker::HsvHueWheel,
                    current,
                    runtime_options.picker == ColorEditPopupPicker::HsvHueWheel,
                    runtime_model.clone(),
                    enabled,
                    row_height,
                    hue_wheel_test_id.clone(),
                ),
            ]
        },
    );

    if let Some(test_id) = test_id {
        row = row.test_id(test_id);
    }
    row
}
