mod field;
#[cfg(test)]
mod tests;

use std::sync::Arc;

use fret_core::{Axis, Color, Edges, Px, TextStyle};
use fret_runtime::Model;
use fret_ui::element::{
    AnyElement, CrossAlign, FlexProps, LayoutStyle, Length, MainAlign, SizeStyle, SpacingLength,
};
use fret_ui::{ElementContext, Invalidation, Theme, UiHost};
use fret_ui_kit::typography;
use fret_ui_kit::{ChromeRefinement, Size};

use crate::primitives::EditorDensity;
use crate::primitives::chrome::resolve_editor_text_field_style;
use crate::primitives::input_group::derived_test_id;
use crate::primitives::readout::editor_inline_error_text_props;

use super::super::ColorEditPopupNumericInputs;
use super::super::model::{
    ColorNumericInputMode, color_numeric_input_modes, hsv_numeric_text, rgb_numeric_text,
};
use field::color_numeric_input_field;

pub(super) fn color_numeric_inputs<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
    current: Color,
    model: Model<Color>,
    hex_draft: Model<String>,
    rgb_draft: Model<String>,
    hsv_draft: Model<String>,
    error: Model<Option<Arc<str>>>,
    numeric_inputs: ColorEditPopupNumericInputs,
    show_alpha: bool,
    enabled: bool,
    test_id: Option<Arc<str>>,
) -> AnyElement {
    let rgb = rgb_numeric_text(current, show_alpha);
    let hsv = hsv_numeric_text(current);
    let modes = color_numeric_input_modes(numeric_inputs);
    let error_msg = cx
        .get_model_cloned(&error, Invalidation::Paint)
        .unwrap_or(None);
    let (chrome, text_style, error_color, row_height) = {
        let theme = Theme::global(&*cx.app);
        let density = EditorDensity::resolve(theme);
        let (chrome, text_style) =
            resolve_editor_text_field_style(theme, Size::default(), &ChromeRefinement::default());
        (
            chrome,
            typography::as_control_text(TextStyle {
                size: Px(10.0),
                line_height: Some(density.row_height),
                ..text_style
            }),
            theme.color_token("destructive"),
            density.row_height,
        )
    };
    let rgb_test_id = derived_test_id(test_id.as_ref(), ColorNumericInputMode::Rgb.test_suffix());
    let hsv_test_id = derived_test_id(test_id.as_ref(), ColorNumericInputMode::Hsv.test_suffix());

    let mut items = Vec::with_capacity(modes.len() + error_msg.is_some() as usize);
    for mode in modes {
        let (draft, display_text, test_id) = match *mode {
            ColorNumericInputMode::Rgb => (rgb_draft.clone(), rgb.clone(), rgb_test_id.clone()),
            ColorNumericInputMode::Hsv => (hsv_draft.clone(), hsv.clone(), hsv_test_id.clone()),
        };
        items.push(color_numeric_input_field(
            cx,
            *mode,
            model.clone(),
            hex_draft.clone(),
            draft,
            error.clone(),
            display_text,
            show_alpha,
            enabled,
            chrome.clone(),
            text_style.clone(),
            error_msg.is_some(),
            test_id,
        ));
    }

    if error_msg.is_none() && items.len() == 1 {
        let mut input = items.pop().expect("single numeric input should exist");
        if let Some(test_id) = test_id.as_ref() {
            input = input.test_id(test_id.clone());
        }
        return input;
    }

    let mut inputs = cx.flex(
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
            gap: SpacingLength::Px(Px(2.0)),
            padding: Edges::all(Px(0.0)).into(),
            justify: MainAlign::Start,
            align: CrossAlign::Stretch,
            wrap: false,
        },
        move |cx| {
            let mut out = items;
            if let Some(msg) = error_msg.clone() {
                out.push(color_numeric_error_line(cx, msg, error_color, row_height));
            }
            out
        },
    );

    if let Some(test_id) = test_id {
        inputs = inputs.test_id(test_id);
    }
    inputs
}

fn color_numeric_error_line<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
    text: Arc<str>,
    color: Color,
    row_height: Px,
) -> AnyElement {
    cx.text_props(editor_inline_error_text_props(text, color, row_height))
}
