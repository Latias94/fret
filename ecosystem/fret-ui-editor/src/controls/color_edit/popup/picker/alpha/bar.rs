use std::sync::Arc;

use fret_core::{Color, Px};
use fret_runtime::Model;
use fret_ui::element::{AnyElement, LayoutStyle, Length, PressableA11y, PressableProps, SizeStyle};
use fret_ui::{ElementContext, UiHost};

use super::super::HSV_PICKER_SIZE;
use super::alpha_percent_text;

mod pointer;
mod surface;

use pointer::{install_alpha_bar_pointer_handlers, install_vertical_alpha_bar_pointer_handlers};
use surface::{alpha_bar_surface, vertical_alpha_bar_surface};

pub(in crate::controls::color_edit::popup) fn vertical_alpha_bar<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
    current: Color,
    model: Model<Color>,
    draft: Model<String>,
    error: Model<Option<Arc<str>>>,
    enabled: bool,
    test_id: Option<Arc<str>>,
) -> AnyElement {
    let rgb = fret_ui_kit::colors::hex_rgb_from_linear(current);
    let alpha = current.a.clamp(0.0, 1.0);
    let value = alpha_percent_text(alpha);

    let mut bar = cx.pressable(
        PressableProps {
            layout: LayoutStyle {
                size: SizeStyle {
                    width: Length::Px(Px(18.0)),
                    height: Length::Px(HSV_PICKER_SIZE),
                    min_height: Some(Length::Px(HSV_PICKER_SIZE)),
                    ..Default::default()
                },
                ..Default::default()
            },
            enabled,
            focusable: enabled,
            a11y: PressableA11y {
                role: Some(fret_core::SemanticsRole::Slider),
                label: Some(Arc::from("Alpha")),
                ..Default::default()
            },
            ..Default::default()
        },
        move |cx, st| {
            install_vertical_alpha_bar_pointer_handlers(cx, model, draft, error);
            vec![vertical_alpha_bar_surface(cx, st.focused, rgb, alpha)]
        },
    );

    if let Some(test_id) = test_id {
        bar = bar.test_id(test_id);
    }
    bar.a11y_value(value)
}

pub(in crate::controls::color_edit::popup) fn alpha_bar<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
    current: Color,
    model: Model<Color>,
    draft: Model<String>,
    error: Model<Option<Arc<str>>>,
    enabled: bool,
    test_id: Option<Arc<str>>,
) -> AnyElement {
    let rgb = fret_ui_kit::colors::hex_rgb_from_linear(current);
    let alpha = current.a.clamp(0.0, 1.0);
    let value = alpha_percent_text(alpha);

    let mut bar = cx.pressable(
        PressableProps {
            layout: LayoutStyle {
                size: SizeStyle {
                    width: Length::Fill,
                    height: Length::Px(Px(18.0)),
                    min_height: Some(Length::Px(Px(18.0))),
                    ..Default::default()
                },
                ..Default::default()
            },
            enabled,
            focusable: enabled,
            a11y: PressableA11y {
                role: Some(fret_core::SemanticsRole::Slider),
                label: Some(Arc::from("Alpha")),
                ..Default::default()
            },
            ..Default::default()
        },
        move |cx, st| {
            install_alpha_bar_pointer_handlers(cx, model, draft, error);
            vec![alpha_bar_surface(cx, st.focused, rgb, alpha)]
        },
    );

    if let Some(test_id) = test_id {
        bar = bar.test_id(test_id);
    }
    bar.a11y_value(value)
}
