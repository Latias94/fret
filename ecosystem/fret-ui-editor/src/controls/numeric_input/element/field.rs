//! NumericInput joined field/frame assembly owner.

use std::sync::{Arc, Mutex};

use fret_core::TextStyle;
use fret_runtime::Model;
use fret_ui::TextInputStyle;
use fret_ui::element::AnyElement;
use fret_ui::element::LayoutStyle;
use fret_ui::{ElementContext, GlobalElementId, Invalidation, UiHost};

use crate::primitives::EditorDensity;
use crate::primitives::chrome::ResolvedEditorFrameChrome;
use crate::primitives::input_group::{
    EditorInputGroupFrameOverrides, editor_joined_input_frame_segments_with_overrides,
};
use crate::primitives::numeric_text_entry::{
    NumericInputSelectionBehavior, NumericTextEntryFocusState,
};
use crate::primitives::visuals::EditorFrameSemanticState;

use super::super::model::{
    NumericFormatFn, NumericInputErrorDisplay, NumericParseFn, NumericValidateFn,
    OnNumericInputOutcome,
};
use super::affix::numeric_input_affix_segment;
use super::error::numeric_input_trailing_error_icon;
use super::input::{
    NumericInputTextEntryArgs, numeric_input_text_entry, numeric_input_text_entry_fill_layout,
};

pub(super) struct NumericInputFieldArgs<T> {
    pub(super) layout: LayoutStyle,
    pub(super) model: Model<T>,
    pub(super) draft: Model<String>,
    pub(super) error_for_field: Model<Option<Arc<str>>>,
    pub(super) error_for_frame: Model<Option<Arc<str>>>,
    pub(super) error_for_trailing: Model<Option<Arc<str>>>,
    pub(super) focus_state: Arc<Mutex<NumericTextEntryFocusState>>,
    pub(super) last_draft_text: Arc<Mutex<String>>,
    pub(super) current_text: Arc<str>,
    pub(super) has_error: bool,
    pub(super) density: EditorDensity,
    pub(super) frame_chrome: ResolvedEditorFrameChrome,
    pub(super) chrome: TextInputStyle,
    pub(super) text_style: TextStyle,
    pub(super) enabled_for_paint: bool,
    pub(super) focusable: bool,
    pub(super) placeholder: Option<Arc<str>>,
    pub(super) error_display: NumericInputErrorDisplay,
    pub(super) selection_behavior: NumericInputSelectionBehavior,
    pub(super) focus_target: Option<Arc<Mutex<Option<GlobalElementId>>>>,
    pub(super) prefix: Option<Arc<str>>,
    pub(super) suffix: Option<Arc<str>>,
    pub(super) test_id: Option<Arc<str>>,
    pub(super) input_test_id: Option<Arc<str>>,
    pub(super) prefix_test_id: Option<Arc<str>>,
    pub(super) suffix_test_id: Option<Arc<str>>,
    pub(super) error_icon_test_id: Option<Arc<str>>,
    pub(super) parse: NumericParseFn<T>,
    pub(super) format: NumericFormatFn<T>,
    pub(super) validate: Option<NumericValidateFn<T>>,
    pub(super) on_outcome: Option<OnNumericInputOutcome>,
}

pub(super) fn numeric_input_field<T, H>(
    cx: &mut ElementContext<'_, H>,
    args: NumericInputFieldArgs<T>,
) -> AnyElement
where
    T: Copy + Default + 'static,
    H: UiHost,
{
    let NumericInputFieldArgs {
        layout,
        model,
        draft,
        error_for_field,
        error_for_frame,
        error_for_trailing,
        focus_state,
        last_draft_text,
        current_text,
        has_error,
        density,
        frame_chrome,
        chrome,
        text_style,
        enabled_for_paint,
        focusable,
        placeholder,
        error_display,
        selection_behavior,
        focus_target,
        prefix,
        suffix,
        test_id,
        input_test_id,
        prefix_test_id,
        suffix_test_id,
        error_icon_test_id,
        parse,
        format,
        validate,
        on_outcome,
    } = args;

    editor_joined_input_frame_segments_with_overrides(
        cx,
        layout,
        density,
        frame_chrome,
        enabled_for_paint,
        false,
        test_id,
        move |cx, focused| {
            let has_error = cx
                .get_model_cloned(&error_for_frame, Invalidation::Paint)
                .unwrap_or(None)
                .is_some();
            EditorInputGroupFrameOverrides {
                semantic: Some(EditorFrameSemanticState {
                    typing: focused,
                    invalid: has_error,
                }),
                ..EditorInputGroupFrameOverrides::none()
            }
        },
        move |cx| {
            let mut segments = Vec::new();
            if let Some(segment) = numeric_input_affix_segment(
                cx,
                density,
                frame_chrome,
                prefix.clone(),
                prefix_test_id.clone(),
            ) {
                segments.push(segment);
            }
            segments
        },
        move |cx| {
            numeric_input_text_entry(
                cx,
                NumericInputTextEntryArgs {
                    layout: numeric_input_text_entry_fill_layout(density),
                    model: model.clone(),
                    draft: draft.clone(),
                    error: error_for_field.clone(),
                    focus_state: focus_state.clone(),
                    last_draft_text: last_draft_text.clone(),
                    current_text: current_text.clone(),
                    has_error,
                    enabled: enabled_for_paint,
                    focusable,
                    placeholder: placeholder.clone(),
                    test_id: input_test_id.clone(),
                    chrome: chrome.clone(),
                    text_style: text_style.clone(),
                    focus_target: focus_target.clone(),
                    selection_behavior,
                    parse: parse.clone(),
                    format: format.clone(),
                    validate: validate.clone(),
                    on_outcome: on_outcome.clone(),
                },
            )
        },
        move |cx| {
            let mut segments = Vec::new();
            if let Some(segment) = numeric_input_affix_segment(
                cx,
                density,
                frame_chrome,
                suffix.clone(),
                suffix_test_id.clone(),
            ) {
                segments.push(segment);
            }
            if let Some(icon) = numeric_input_trailing_error_icon(
                cx,
                density,
                error_display,
                &error_for_trailing,
                error_icon_test_id.clone(),
            ) {
                segments.push(icon);
            }
            segments
        },
    )
}
