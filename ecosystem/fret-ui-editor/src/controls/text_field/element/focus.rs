//! Text-field element focus-selection owner.

use std::sync::{Arc, Mutex};

use fret_runtime::Model;
use fret_ui::{ElementContext, GlobalElementId, Invalidation, UiHost};

use crate::primitives::text_entry::{
    EditorTextEntryFocusState, EditorTextSelectionBehavior, sync_editor_text_entry_focus_selection,
};

pub(super) struct TextFieldFocusSelectionArgs<'a> {
    pub(super) focus_state: &'a Arc<Mutex<EditorTextEntryFocusState>>,
    pub(super) entry_id: GlobalElementId,
    pub(super) is_focused: bool,
    pub(super) model: &'a Model<String>,
    pub(super) draft: Option<&'a Model<String>>,
    pub(super) selection_behavior: EditorTextSelectionBehavior,
}

pub(super) fn sync_text_field_focus_selection<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
    args: TextFieldFocusSelectionArgs<'_>,
) {
    let TextFieldFocusSelectionArgs {
        focus_state,
        entry_id,
        is_focused,
        model,
        draft,
        selection_behavior,
    } = args;

    let has_value = if let Some(draft) = draft {
        cx.read_model_ref(draft, Invalidation::Paint, |s| !s.is_empty())
            .unwrap_or(false)
    } else {
        cx.read_model_ref(model, Invalidation::Paint, |s| !s.is_empty())
            .unwrap_or(false)
    };

    sync_editor_text_entry_focus_selection(
        cx,
        focus_state,
        entry_id,
        is_focused,
        has_value,
        selection_behavior,
    );
}
