use std::sync::{Arc, Mutex};

use fret_runtime::Model;
use fret_ui::action::OnActivate;
use fret_ui::element::AnyElement;
use fret_ui::{ElementContext, Invalidation, UiHost};

use crate::primitives::EditorDensity;
use crate::primitives::chrome::ResolvedEditorFrameChrome;
use crate::primitives::input_group::{
    editor_clear_button_segment, editor_clear_button_segment_multiline,
};

use super::super::buffered::{self, BufferedTextFieldState};

pub(super) struct TextFieldClearButtonArgs {
    pub(super) density: EditorDensity,
    pub(super) frame_chrome: ResolvedEditorFrameChrome,
    pub(super) enabled: bool,
    pub(super) multiline: bool,
    pub(super) clear_button: bool,
    pub(super) clear_test_id: Option<Arc<str>>,
    pub(super) model: Model<String>,
    pub(super) draft: Option<Model<String>>,
    pub(super) buffered_state: Option<Arc<Mutex<BufferedTextFieldState>>>,
}

pub(super) fn text_field_clear_button_segments<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
    args: TextFieldClearButtonArgs,
) -> Vec<AnyElement> {
    let TextFieldClearButtonArgs {
        density,
        frame_chrome,
        enabled,
        multiline,
        clear_button,
        clear_test_id,
        model,
        draft,
        buffered_state,
    } = args;

    let has_value = if let Some(draft) = draft.as_ref() {
        cx.read_model_ref(draft, Invalidation::Layout, |s| !s.is_empty())
            .unwrap_or(false)
    } else {
        cx.read_model_ref(&model, Invalidation::Layout, |s| !s.is_empty())
            .unwrap_or(false)
    };
    if !(clear_button && has_value && enabled) {
        return Vec::new();
    }

    let model_for_clear = model.clone();
    let on_activate: OnActivate =
        if let (Some(draft), Some(buffered_state)) = (draft.clone(), buffered_state.clone()) {
            Arc::new(move |host, action_cx, _reason| {
                let _ = host.models_mut().update(&draft, |s| s.clear());
                let _ = host.models_mut().update(&model_for_clear, |s| s.clear());
                let mut state = buffered_state.lock().unwrap_or_else(|e| e.into_inner());
                buffered::clear_buffered_text_field_state(&mut state);
                host.request_redraw(action_cx.window);
            })
        } else {
            Arc::new(move |host, action_cx, _reason| {
                let _ = host.models_mut().update(&model_for_clear, |s| s.clear());
                host.request_redraw(action_cx.window);
            })
        };

    if multiline {
        vec![editor_clear_button_segment_multiline(
            cx,
            density,
            frame_chrome,
            enabled,
            Arc::from("Clear text"),
            clear_test_id,
            on_activate,
        )]
    } else {
        vec![editor_clear_button_segment(
            cx,
            density,
            enabled,
            Arc::from("Clear text"),
            clear_test_id,
            on_activate,
        )]
    }
}
