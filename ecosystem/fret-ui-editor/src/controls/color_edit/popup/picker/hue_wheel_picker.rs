use std::sync::Arc;

use fret_core::{Color, Corners, Edges, Px};
use fret_runtime::Model;
use fret_ui::element::{
    AnyElement, ContainerProps, LayoutStyle, Length, Overflow, PressableA11y, PressableProps,
    SizeStyle,
};
use fret_ui::{ElementContext, Theme, UiHost};

use super::super::super::model::hsv_from_color;
use super::super::preview::fill_preview_layout;
use super::{HSV_PICKER_SIZE, HUE_WHEEL_PICKER_WIDTH, hue_wheel_canvas, picker_border_and_ring};

mod pointer;

use pointer::install_hue_wheel_pointer_handlers;

pub(super) fn hue_wheel_picker<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
    current: Color,
    model: Model<Color>,
    draft: Model<String>,
    error: Model<Option<Arc<str>>>,
    show_alpha: bool,
    enabled: bool,
    test_id: Option<Arc<str>>,
) -> AnyElement {
    let hsv = hsv_from_color(current);
    let value = Arc::from(format!(
        "Hue {}%, S {}%, V {}%",
        (hsv.hue.clamp(0.0, 1.0) * 100.0).round() as u8,
        (hsv.saturation.clamp(0.0, 1.0) * 100.0).round() as u8,
        (hsv.value.clamp(0.0, 1.0) * 100.0).round() as u8
    ));

    let mut picker = cx.pressable(
        PressableProps {
            layout: LayoutStyle {
                size: SizeStyle {
                    width: Length::Px(HUE_WHEEL_PICKER_WIDTH),
                    height: Length::Px(HSV_PICKER_SIZE),
                    min_width: Some(Length::Px(HUE_WHEEL_PICKER_WIDTH)),
                    min_height: Some(Length::Px(HSV_PICKER_SIZE)),
                    ..Default::default()
                },
                ..Default::default()
            },
            enabled,
            focusable: enabled,
            a11y: PressableA11y {
                role: Some(fret_core::SemanticsRole::Slider),
                label: Some(Arc::from("Hue wheel and saturation/value triangle")),
                ..Default::default()
            },
            ..Default::default()
        },
        move |cx, st| {
            install_hue_wheel_pointer_handlers(cx, model, draft, error, show_alpha);

            let theme = Theme::global(&*cx.app);
            let (border, ring) = picker_border_and_ring(theme);
            let border_color = if st.focused { ring } else { border };

            vec![cx.container(
                ContainerProps {
                    layout: LayoutStyle {
                        overflow: Overflow::Clip,
                        ..fill_preview_layout()
                    },
                    border: Edges::all(Px(1.0)),
                    border_color: Some(border_color),
                    corner_radii: Corners::all(Px(5.0)),
                    ..Default::default()
                },
                move |cx| vec![hue_wheel_canvas(cx, hsv)],
            )]
        },
    );

    if let Some(test_id) = test_id {
        picker = picker.test_id(test_id);
    }
    picker.a11y_value(value)
}
