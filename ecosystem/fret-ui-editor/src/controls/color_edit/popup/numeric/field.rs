use std::sync::Arc;

use fret_core::{Color, KeyCode, Px, SemanticsInvalid, TextStyle};
use fret_runtime::Model;
use fret_ui::action::ActionCx;
use fret_ui::element::{AnyElement, LayoutStyle, Length, SizeStyle, TextInputProps};
use fret_ui::{ElementContext, UiHost};

use super::super::super::model::{
    ColorNumericInputMode, color_numeric_text, format_hex, parse_color_numeric_input,
};

pub(super) fn color_numeric_input_field<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
    mode: ColorNumericInputMode,
    model: Model<Color>,
    hex_draft: Model<String>,
    draft: Model<String>,
    error: Model<Option<Arc<str>>>,
    display_text: Arc<str>,
    show_alpha: bool,
    enabled: bool,
    row_height: Px,
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
            min_height: Some(Length::Px(row_height)),
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
        let draft_needs_sync = cx
            .app
            .models()
            .read(&draft, |s| s.as_str() != display_text.as_ref())
            .unwrap_or(true);
        if draft_needs_sync {
            let _ = cx
                .app
                .models_mut()
                .update(&draft, |s| *s = display_text.as_ref().to_string());
        }
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

fn color_numeric_placeholder(mode: ColorNumericInputMode, show_alpha: bool) -> Arc<str> {
    match (mode, show_alpha) {
        (ColorNumericInputMode::Rgb, true) => Arc::from("RGB 255 255 255 | A 100%"),
        (ColorNumericInputMode::Rgb, false) => Arc::from("RGB 255 255 255"),
        (ColorNumericInputMode::Hsv, _) => Arc::from("HSV 0deg | S 100% | V 100%"),
    }
}
