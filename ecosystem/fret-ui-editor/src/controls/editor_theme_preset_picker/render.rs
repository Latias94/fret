use std::sync::Arc;

use fret_core::{Color, Px};
use fret_runtime::Model;
use fret_ui::element::AnyElement;
use fret_ui::{ElementContext, UiHost};

mod listbox;
mod row;

use crate::theme::EditorThemePreset;

pub(super) struct EditorThemePresetPickerRenderInput {
    pub(super) selected: EditorThemePreset,
    pub(super) label: Arc<str>,
    pub(super) item_prefix: Option<Arc<str>>,
    pub(super) options: super::EditorThemePresetPickerOptions,
    pub(super) model: Model<EditorThemePreset>,
    pub(super) total: usize,
    pub(super) row_height: Px,
    pub(super) padding_x: Px,
    pub(super) border: Color,
    pub(super) ring: Color,
    pub(super) fg: Color,
    pub(super) muted_fg: Color,
    pub(super) subtle_bg: Color,
    pub(super) accent: Color,
    pub(super) text_px: Px,
}

pub(super) fn build_editor_theme_preset_picker_element<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
    input: EditorThemePresetPickerRenderInput,
) -> AnyElement {
    listbox::theme_preset_picker_listbox(cx, input)
}
