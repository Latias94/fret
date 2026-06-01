use std::panic::Location;

use fret_runtime::Model;
use fret_ui::element::AnyElement;
use fret_ui::{ElementContext, Invalidation, Theme, UiHost};

use super::{OnTextFieldOutcome, TextField, TextFieldOptions, buffered};
use crate::primitives::input_group::editor_joined_input_frame;
use crate::primitives::style::EditorStyle;
use crate::primitives::text_entry::editor_text_entry_focus_state;

mod buffered_keys;
mod clear_button;
mod entry;
mod entry_props;
mod escape_clear;
mod focus;

use clear_button::{TextFieldClearButtonArgs, text_field_clear_button_segments};
use entry::{TextFieldEntryArgs, text_field_entry};

impl TextField {
    pub fn new(model: Model<String>) -> Self {
        Self {
            model,
            on_outcome: None,
            options: TextFieldOptions::default(),
        }
    }

    pub fn on_outcome(mut self, on_outcome: Option<OnTextFieldOutcome>) -> Self {
        self.on_outcome = on_outcome;
        self
    }

    pub fn options(mut self, options: TextFieldOptions) -> Self {
        self.options = options;
        self
    }

    #[track_caller]
    pub fn into_element<H: UiHost>(self, cx: &mut ElementContext<'_, H>) -> AnyElement {
        let model_id = self.model.id();
        let loc = Location::caller();
        let callsite = (loc.file(), loc.line(), loc.column());
        let id_source = self.options.id_source.clone();

        if let Some(id_source) = id_source.as_deref() {
            cx.keyed(("fret-ui-editor.text_field", id_source, model_id), |cx| {
                self.into_element_keyed(cx)
            })
        } else {
            cx.keyed(("fret-ui-editor.text_field", callsite, model_id), |cx| {
                self.into_element_keyed(cx)
            })
        }
    }

    fn into_element_keyed<H: UiHost>(self, cx: &mut ElementContext<'_, H>) -> AnyElement {
        let TextField {
            model,
            on_outcome,
            options,
        } = self;

        let layout = options.layout;
        let size = options.size;
        let placeholder = options.placeholder.clone();
        let enabled_for_paint = options.enabled;
        let focusable = options.focusable;
        let clear_button = options.clear_button;
        let a11y_label = options.a11y_label.clone();
        let test_id = options.test_id.clone();
        let clear_test_id = options.clear_test_id.clone();
        let field_id_out = options.field_id_out.clone();
        let input_id_out = options.input_id_out.clone();
        let mode = options.mode;
        let buffered = options.buffered;
        let blur_behavior = options.blur_behavior;
        let draft_controller = options.draft_controller.clone();
        let submit_command = options.submit_command.clone();
        let assistive_semantics = options.assistive_semantics;
        let selection_behavior = options.selection_behavior;
        let cancel_behavior = options.cancel_behavior;
        let multiline = options.multiline;
        let stable_line_boxes = options.stable_line_boxes;
        let min_height = options.min_height;
        let focus_state = editor_text_entry_focus_state(cx);
        let draft = buffered.then(|| buffered::draft_model(cx));
        let buffered_state = buffered.then(|| buffered::buffered_state(cx));
        let current_text = cx
            .get_model_cloned(&model, Invalidation::Paint)
            .unwrap_or_default();

        if let (Some(draft), Some(buffered_state)) = (draft.as_ref(), buffered_state.as_ref()) {
            buffered::sync_draft_from_model_when_session_inactive(
                cx,
                draft,
                buffered_state,
                &current_text,
            );
        }

        let (density, frame_chrome) = {
            let theme = Theme::global(&*cx.app);
            let style = EditorStyle::resolve(theme);
            (style.density, style.frame_chrome(size))
        };

        let model_for_trailing = model.clone();
        let draft_for_trailing = draft.clone();
        let buffered_state_for_trailing = buffered_state.clone();
        if !buffered && let Some(controller) = draft_controller.as_ref() {
            controller.unbind();
        }

        let field = editor_joined_input_frame(
            cx,
            layout,
            density,
            frame_chrome,
            enabled_for_paint,
            false,
            None,
            move |cx| {
                text_field_entry(
                    cx,
                    TextFieldEntryArgs {
                        model: model.clone(),
                        draft: draft.clone(),
                        buffered_state: buffered_state.clone(),
                        current_text,
                        draft_controller,
                        on_outcome: on_outcome.clone(),
                        submit_command: submit_command.clone(),
                        focus_state,
                        size,
                        density,
                        enabled: enabled_for_paint,
                        focusable,
                        placeholder: placeholder.clone(),
                        a11y_label: a11y_label.clone(),
                        test_id: test_id.clone(),
                        input_id_out: input_id_out.clone(),
                        mode,
                        buffered,
                        blur_behavior,
                        assistive_semantics,
                        selection_behavior,
                        cancel_behavior,
                        multiline,
                        stable_line_boxes,
                        min_height,
                    },
                )
            },
            move |cx| {
                text_field_clear_button_segments(
                    cx,
                    TextFieldClearButtonArgs {
                        density,
                        frame_chrome,
                        enabled: enabled_for_paint,
                        multiline,
                        clear_button,
                        clear_test_id: clear_test_id.clone(),
                        model: model_for_trailing.clone(),
                        draft: draft_for_trailing.clone(),
                        buffered_state: buffered_state_for_trailing.clone(),
                    },
                )
            },
        );

        if let Some(out) = field_id_out.as_ref() {
            out.set(Some(field.id));
        }

        field
    }
}
