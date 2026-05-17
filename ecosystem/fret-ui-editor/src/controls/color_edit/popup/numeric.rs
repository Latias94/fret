use std::sync::Arc;

use fret_core::{Axis, Color, Edges, KeyCode, Px, SemanticsInvalid, TextStyle};
use fret_runtime::Model;
use fret_ui::action::ActionCx;
use fret_ui::element::{
    AnyElement, CrossAlign, FlexProps, LayoutStyle, Length, MainAlign, SizeStyle, SpacingLength,
    TextInputProps,
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
    ColorNumericInputMode, color_numeric_input_modes, color_numeric_text, format_hex,
    hsv_numeric_text, parse_color_numeric_input, rgb_numeric_text,
};

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
            let mut out = Vec::new();
            for mode in color_numeric_input_modes(numeric_inputs) {
                let (draft, display_text, test_id) = match *mode {
                    ColorNumericInputMode::Rgb => {
                        (rgb_draft.clone(), rgb.clone(), rgb_test_id.clone())
                    }
                    ColorNumericInputMode::Hsv => {
                        (hsv_draft.clone(), hsv.clone(), hsv_test_id.clone())
                    }
                };
                out.push(color_numeric_input_field(
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

fn color_numeric_input_field<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
    mode: ColorNumericInputMode,
    model: Model<Color>,
    hex_draft: Model<String>,
    draft: Model<String>,
    error: Model<Option<Arc<str>>>,
    display_text: Arc<str>,
    show_alpha: bool,
    enabled: bool,
    chrome: fret_ui::TextInputStyle,
    text_style: TextStyle,
    has_error: bool,
    test_id: Option<Arc<str>>,
) -> AnyElement {
    let mut props = TextInputProps::new(draft.clone());
    props.layout = LayoutStyle {
        size: SizeStyle {
            width: Length::Fill,
            height: Length::Auto,
            min_height: Some(Length::Px(row_height_from_style(&text_style))),
            ..Default::default()
        },
        ..Default::default()
    };
    props.enabled = enabled;
    props.focusable = enabled;
    props.test_id = test_id;
    props.placeholder = Some(color_numeric_placeholder(mode, show_alpha));
    props.a11y_label = Some(mode.a11y_label());
    props.a11y_invalid = has_error.then_some(SemanticsInvalid::True);
    props.chrome = chrome;
    props.text_style = text_style;

    let input = cx.text_input(props);
    let input_id = input.id;
    let is_focused = cx.is_focused_element(input_id);

    if !is_focused {
        let _ = cx
            .app
            .models_mut()
            .update(&draft, |s| *s = display_text.as_ref().to_string());
    }

    let model_for_key = model;
    let hex_draft_for_key = hex_draft;
    let draft_for_key = draft;
    let error_for_key = error;
    cx.key_add_on_key_down_capture_for(
        input_id,
        Arc::new(move |host, action_cx: ActionCx, down| match down.key {
            KeyCode::Enter | KeyCode::NumpadEnter => {
                let text = host
                    .models_mut()
                    .read(&draft_for_key, |s| s.clone())
                    .unwrap_or_default();
                let current = host
                    .models_mut()
                    .get_copied(&model_for_key)
                    .unwrap_or(Color::TRANSPARENT);
                if let Some(next) = parse_color_numeric_input(mode, &text, show_alpha, current) {
                    let _ = host.models_mut().update(&model_for_key, |c| *c = next);
                    let formatted = format_hex(next, show_alpha);
                    let numeric = color_numeric_text(next, show_alpha, mode);
                    let _ = host
                        .models_mut()
                        .update(&hex_draft_for_key, |s| *s = formatted.as_ref().to_string());
                    let _ = host
                        .models_mut()
                        .update(&draft_for_key, |s| *s = numeric.as_ref().to_string());
                    let _ = host.models_mut().update(&error_for_key, |e| *e = None);
                } else {
                    let message = mode.invalid_message();
                    let _ = host
                        .models_mut()
                        .update(&error_for_key, |e| *e = Some(message));
                }
                host.request_redraw(action_cx.window);
                true
            }
            KeyCode::Escape => {
                let current = host
                    .models_mut()
                    .get_copied(&model_for_key)
                    .unwrap_or(Color::TRANSPARENT);
                let numeric = color_numeric_text(current, show_alpha, mode);
                let _ = host
                    .models_mut()
                    .update(&draft_for_key, |s| *s = numeric.as_ref().to_string());
                let _ = host.models_mut().update(&error_for_key, |e| *e = None);
                host.request_redraw(action_cx.window);
                true
            }
            _ => false,
        }),
    );

    input
}

fn color_numeric_error_line<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
    text: Arc<str>,
    color: Color,
    row_height: Px,
) -> AnyElement {
    cx.text_props(editor_inline_error_text_props(text, color, row_height))
}

fn row_height_from_style(style: &TextStyle) -> Px {
    style.line_height.unwrap_or(style.size)
}

fn color_numeric_placeholder(mode: ColorNumericInputMode, show_alpha: bool) -> Arc<str> {
    match (mode, show_alpha) {
        (ColorNumericInputMode::Rgb, true) => Arc::from("RGB 255 255 255 | A 100%"),
        (ColorNumericInputMode::Rgb, false) => Arc::from("RGB 255 255 255"),
        (ColorNumericInputMode::Hsv, _) => Arc::from("HSV 0deg | S 100% | V 100%"),
    }
}
