use std::sync::{Arc, Mutex};

use fret_core::{Axis, Edges, Px, SemanticsInvalid, TextStyle};
use fret_ui::element::{
    AnyElement, CrossAlign, FlexProps, LayoutStyle, Length, MainAlign, SizeStyle, SpacingLength,
    TextInputProps,
};
use fret_ui::{ElementContext, Invalidation, Theme, UiHost};
use fret_ui_kit::ChromeRefinement;

use crate::primitives::chrome::{joined_text_input_style, resolve_editor_text_field_style};
use crate::primitives::colors::{
    editor_invalid_border, editor_invalid_foreground, editor_muted_foreground,
};
use crate::primitives::input_group::{
    EditorInputGroupFrameOverrides, derived_test_id, editor_icon_segment,
    editor_joined_input_frame_segments_with_overrides, editor_text_segment,
};
use crate::primitives::numeric_format::suppress_duplicate_chrome_affixes;
use crate::primitives::numeric_text_entry::{
    clear_numeric_error_when_draft_changes, numeric_text_entry_focus_state,
    sync_numeric_text_entry_focus,
};
use crate::primitives::readout::editor_validation_message_text_props;
use crate::primitives::style::EditorStyle;

use super::NumericInput;
use super::keyboard::{NumericInputKeyHandlerArgs, numeric_input_key_down_handler};
use super::model::{NumericInputErrorDisplay, editor_numeric_input_text_style};
use super::session::{draft_model, error_model};

pub(super) fn numeric_input_into_element_keyed<T, H>(
    input: NumericInput<T>,
    cx: &mut ElementContext<'_, H>,
) -> AnyElement
where
    T: Copy + Default + 'static,
    H: UiHost,
{
    let NumericInput {
        model,
        format,
        parse,
        validate,
        on_outcome,
        options,
        focus_target,
    } = input;

    let draft = draft_model(cx);
    let error = error_model(cx);
    let focus_state = numeric_text_entry_focus_state(cx);
    let last_draft_text = cx.slot_state(|| Arc::new(Mutex::new(String::new())), |st| st.clone());
    let current_value = cx
        .get_model_copied(&model, Invalidation::Paint)
        .unwrap_or_default();
    let current_text = (format)(current_value);
    let has_error = cx
        .get_model_cloned(&error, Invalidation::Paint)
        .unwrap_or(None)
        .is_some();

    let (density, frame_chrome, chrome, text_style) = {
        let theme = Theme::global(&*cx.app);
        let style = EditorStyle::resolve(theme);
        let density = style.density;
        let frame_chrome = style.frame_chrome(options.size);
        let (chrome, text_style) =
            resolve_editor_text_field_style(theme, options.size, &ChromeRefinement::default());
        (density, frame_chrome, chrome, text_style)
    };

    let enabled_for_paint = options.enabled;
    let error_for_field = error.clone();
    let error_for_frame = error.clone();
    let error_for_trailing = error.clone();
    let text_style_for_field = editor_numeric_input_text_style(text_style.clone(), density);
    let placeholder = options.placeholder.clone();
    let focusable = options.focusable;
    let error_display = options.error_display;
    let selection_behavior = options.selection_behavior;
    let focus_target = focus_target.clone();
    let (prefix, suffix) = suppress_duplicate_chrome_affixes(
        current_text.as_ref(),
        options.prefix.clone(),
        options.suffix.clone(),
    );
    let input_test_id = derived_test_id(options.test_id.as_ref(), "input");
    let prefix_test_id = derived_test_id(options.test_id.as_ref(), "prefix");
    let suffix_test_id = derived_test_id(options.test_id.as_ref(), "suffix");
    let error_icon_test_id = derived_test_id(options.test_id.as_ref(), "error");
    let error_text_test_id = derived_test_id(options.test_id.as_ref(), "error-text");

    let field = editor_joined_input_frame_segments_with_overrides(
        cx,
        LayoutStyle {
            size: SizeStyle {
                width: Length::Fill,
                height: Length::Auto,
                ..Default::default()
            },
            ..Default::default()
        },
        density,
        frame_chrome,
        enabled_for_paint,
        false,
        options.test_id.clone(),
        move |cx, focused| {
            let has_error = cx
                .get_model_cloned(&error_for_frame, Invalidation::Paint)
                .unwrap_or(None)
                .is_some();
            EditorInputGroupFrameOverrides {
                semantic: Some(crate::primitives::visuals::EditorFrameSemanticState {
                    typing: focused,
                    invalid: has_error,
                }),
                ..EditorInputGroupFrameOverrides::none()
            }
        },
        move |cx| {
            let theme = Theme::global(&*cx.app);
            let affix_color = editor_muted_foreground(theme);
            let mut segments = Vec::new();

            if let Some(prefix) = prefix.clone() {
                let mut segment = editor_text_segment(
                    cx,
                    density,
                    frame_chrome.text_px,
                    prefix.clone(),
                    affix_color,
                    frame_chrome.padding,
                );
                if let Some(test_id) = prefix_test_id.as_ref() {
                    segment = segment.test_id(test_id.clone()).a11y_label(prefix);
                }
                segments.push(segment);
            }
            segments
        },
        move |cx| {
            let mut props = TextInputProps::new(draft.clone());
            props.layout = LayoutStyle {
                size: SizeStyle {
                    width: Length::Fill,
                    height: Length::Fill,
                    min_height: Some(Length::Px(density.row_height)),
                    ..Default::default()
                },
                ..Default::default()
            };
            props.enabled = enabled_for_paint;
            props.focusable = focusable;
            props.placeholder = placeholder.clone();
            props.test_id = input_test_id.clone();
            props.a11y_invalid = has_error.then_some(SemanticsInvalid::True);
            props.chrome = joined_text_input_style(chrome);
            props.text_style = text_style_for_field.clone();

            let input = cx.text_input(props);
            let input_id = input.id;
            if let Some(focus_target) = focus_target.as_ref() {
                let mut slot = focus_target.lock().unwrap_or_else(|e| e.into_inner());
                *slot = Some(input_id);
            }
            let is_focused = cx.is_focused_element(input_id);

            sync_numeric_text_entry_focus(
                cx,
                &focus_state,
                is_focused,
                &current_text,
                &draft,
                &error_for_field,
                selection_behavior,
            );

            if !is_focused {
                let mut last = last_draft_text.lock().unwrap_or_else(|e| e.into_inner());
                *last = current_text.as_ref().to_string();
            }

            cx.key_add_on_key_down_capture_for(
                input_id,
                numeric_input_key_down_handler(NumericInputKeyHandlerArgs {
                    model: model.clone(),
                    draft: draft.clone(),
                    error: error_for_field.clone(),
                    focus_state: focus_state.clone(),
                    last_draft_text: last_draft_text.clone(),
                    parse: parse.clone(),
                    format: format.clone(),
                    validate: validate.clone(),
                    on_outcome: on_outcome.clone(),
                }),
            );

            clear_numeric_error_when_draft_changes(
                cx,
                is_focused,
                &draft,
                &error_for_field,
                &last_draft_text,
            );

            input
        },
        move |cx| {
            let mut segments = Vec::new();
            let affix_color = {
                let theme = Theme::global(&*cx.app);
                editor_muted_foreground(theme)
            };

            if let Some(suffix) = suffix.clone() {
                let mut segment = editor_text_segment(
                    cx,
                    density,
                    frame_chrome.text_px,
                    suffix.clone(),
                    affix_color,
                    frame_chrome.padding,
                );
                if let Some(test_id) = suffix_test_id.as_ref() {
                    segment = segment.test_id(test_id.clone()).a11y_label(suffix);
                }
                segments.push(segment);
            }

            let show_icon = matches!(
                error_display,
                NumericInputErrorDisplay::TrailingIcon
                    | NumericInputErrorDisplay::InlineTextAndIcon
            );
            if !show_icon {
                return segments;
            }

            let error_msg = cx
                .get_model_cloned(&error_for_trailing, Invalidation::Paint)
                .unwrap_or(None);
            if error_msg.is_none() {
                return segments;
            }

            let error_border = {
                let theme = Theme::global(&*cx.app);
                editor_invalid_border(theme)
            };

            let mut icon = editor_icon_segment(
                cx,
                density,
                fret_icons::ids::ui::STATUS_FAILED,
                Some(Px(12.0)),
                Some(fret_ui_kit::ColorRef::Color(error_border)),
            );
            if let Some(test_id) = error_icon_test_id.as_ref() {
                icon = icon.test_id(test_id.clone());
            }
            segments.push(icon);
            segments
        },
    );

    let error_msg = cx
        .get_model_cloned(&error, Invalidation::Paint)
        .unwrap_or(None);

    let error_color = {
        let theme = Theme::global(&*cx.app);
        editor_invalid_foreground(theme)
    };
    let show_inline_error = matches!(
        error_display,
        NumericInputErrorDisplay::InlineText | NumericInputErrorDisplay::InlineTextAndIcon
    );

    let error_el = (show_inline_error).then_some(()).and_then(|_| {
        error_msg.map(|msg| {
            let mut error = cx.text_props(editor_validation_message_text_props(
                msg.clone(),
                error_color,
                TextStyle {
                    size: text_style.size,
                    line_height: text_style.line_height,
                    ..Default::default()
                },
            ));
            if let Some(test_id) = error_text_test_id.as_ref() {
                error = error.test_id(test_id.clone()).a11y_label(msg.clone());
            }
            error
        })
    });

    let mut layout = options.layout;
    if layout.size.min_height.is_none() {
        layout.size.min_height = Some(Length::Px(density.row_height));
    }

    cx.flex(
        FlexProps {
            layout,
            direction: Axis::Vertical,
            gap: SpacingLength::Px(Px(4.0)),
            padding: Edges::all(Px(0.0)).into(),
            justify: MainAlign::Start,
            align: CrossAlign::Start,
            wrap: false,
        },
        move |_cx| {
            let mut out = vec![field];
            if let Some(error) = error_el {
                out.push(error);
            }
            out
        },
    )
}
