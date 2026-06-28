use std::sync::{Arc, Mutex};

use fret_ui::element::AnyElement;
use fret_ui::{ElementContext, Invalidation, Theme, UiHost};

use super::DragValue;
use super::model::{DragValueMode, DragValueState};
use super::scrub_element::{DragValueScrubElementArgs, drag_value_scrub_element};
use super::typing::{DragValueTypingInputArgs, drag_value_typing_input};
use crate::primitives::drag_value_core::DragValueScalar;
use crate::primitives::input_group::derived_test_id;
use crate::primitives::numeric_format::suppress_duplicate_chrome_affixes;
use crate::primitives::numeric_text_entry::NumericTextEntryFocusHandoffState;
use crate::primitives::style::EditorStyle;

pub(super) fn drag_value_into_element_keyed<H, T>(
    drag_value: DragValue<T>,
    cx: &mut ElementContext<'_, H>,
) -> AnyElement
where
    H: UiHost,
    T: DragValueScalar + Default,
{
    let DragValue {
        model,
        format,
        parse,
        validate,
        on_outcome,
        options,
    } = drag_value;

    let state: Arc<Mutex<DragValueState>> = cx.slot_state(
        || Arc::new(Mutex::new(DragValueState::default())),
        |s| s.clone(),
    );
    let focus_handoff: Arc<Mutex<NumericTextEntryFocusHandoffState>> = cx.slot_state(
        || Arc::new(Mutex::new(NumericTextEntryFocusHandoffState::default())),
        |s| s.clone(),
    );

    let value = cx
        .get_model_copied(&model, Invalidation::Paint)
        .unwrap_or_default();
    let value_text = (format)(value);

    let mode = {
        let st = state.lock().unwrap_or_else(|e| e.into_inner());
        st.mode
    };

    let typing = mode == DragValueMode::Typing;
    let (prefix, suffix) = suppress_duplicate_chrome_affixes(
        value_text.as_ref(),
        options.prefix.clone(),
        options.suffix.clone(),
    );
    let scrub_test_id = options.test_id.clone();
    let typing_test_id = derived_test_id(options.test_id.as_ref(), "typing");
    let active_typing_test_id = if typing { typing_test_id.clone() } else { None };
    let prefix_test_id = derived_test_id(scrub_test_id.as_ref(), "prefix");
    let suffix_test_id = derived_test_id(scrub_test_id.as_ref(), "suffix");
    let value_test_id = derived_test_id(scrub_test_id.as_ref(), "value");

    let control_height = {
        let style = EditorStyle::resolve(Theme::global(&*cx.app));
        style
            .frame_chrome_small()
            .control_outer_height(style.density.row_height)
    };
    let shell_layout =
        crate::controls::session_shell::session_shell_layout(options.layout, control_height);
    let active_branch_layout = crate::controls::session_shell::session_branch_layout();

    let scrub_layout = if typing {
        crate::controls::session_shell::hidden_session_branch_layout(active_branch_layout)
    } else {
        active_branch_layout
    };
    let scrub = drag_value_scrub_element(
        cx,
        DragValueScrubElementArgs {
            model: model.clone(),
            value,
            value_text: value_text.clone(),
            layout: scrub_layout,
            scrub_enabled: mode == DragValueMode::Scrub,
            constraints: options.constraints,
            state: state.clone(),
            focus_handoff: focus_handoff.clone(),
            on_outcome: on_outcome.clone(),
            prefix: prefix.clone(),
            suffix: suffix.clone(),
            scrub_test_id: scrub_test_id.clone(),
            prefix_test_id: prefix_test_id.clone(),
            suffix_test_id: suffix_test_id.clone(),
            value_test_id: value_test_id.clone(),
        },
    );

    let mut input_layout = active_branch_layout;
    if !typing {
        input_layout = crate::controls::session_shell::hidden_session_branch_layout(input_layout);
    }

    let input = drag_value_typing_input(
        cx,
        DragValueTypingInputArgs {
            model: model.clone(),
            format: format.clone(),
            parse: parse.clone(),
            validate: validate.clone(),
            constraints: options.constraints,
            input_layout,
            typing,
            prefix: prefix.clone(),
            suffix: suffix.clone(),
            selection_behavior: options.selection_behavior,
            active_typing_test_id,
            state: state.clone(),
            focus_handoff: focus_handoff.clone(),
            on_outcome: on_outcome.clone(),
        },
    );

    // Render both: scrub stays mounted so focus can restore, input stays mounted so focus
    // requests have a stable target.
    crate::controls::session_shell::session_shell(cx, shell_layout, vec![scrub, input])
}
