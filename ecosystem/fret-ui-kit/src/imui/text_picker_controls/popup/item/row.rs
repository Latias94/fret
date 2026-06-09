use std::sync::Arc;

use fret_runtime::Model;
use fret_ui::{GlobalElementId, UiHost};

use super::super::super::keyboard::InputTextPickerKeyboardState;
use crate::imui::{ImUiFacade, SelectableOptions};

pub(super) struct TextPickerPopupItemRowInput<'a> {
    pub(super) source_index: usize,
    pub(super) visible_index: usize,
    pub(super) candidate: Arc<str>,
    pub(super) selected_value: &'a str,
    pub(super) active_source_index: Option<usize>,
    pub(super) item_test_id_base: Option<Arc<str>>,
    pub(super) keyboard_state: Option<Model<InputTextPickerKeyboardState>>,
}

pub(super) struct TextPickerPopupItemRow {
    pub(super) clicked: bool,
    pub(super) element: Option<GlobalElementId>,
}

pub(super) fn render_text_picker_popup_item_row<H: UiHost>(
    ui: &mut ImUiFacade<'_, '_, H>,
    input: TextPickerPopupItemRowInput<'_>,
) -> TextPickerPopupItemRow {
    let checked = input.selected_value == input.candidate.as_ref();
    let active = input.active_source_index == Some(input.source_index);
    let item_test_id = input
        .item_test_id_base
        .as_ref()
        .map(|base| Arc::from(format!("{base}.option.{}", input.visible_index)));
    let response = ui.selectable_with_options(
        input.candidate,
        SelectableOptions {
            selected: checked,
            highlighted: active,
            test_id: item_test_id,
            ..Default::default()
        },
    );
    let element = response.id();

    if active && let (Some(state), Some(element)) = (input.keyboard_state.as_ref(), element) {
        let _ = ui
            .cx_mut()
            .app
            .models_mut()
            .update(state, |state| state.active_element = Some(element));
    }

    TextPickerPopupItemRow {
        clicked: response.clicked(),
        element,
    }
}
