use std::sync::Arc;

use fret_runtime::Model;
use fret_ui::element::AnyElement;
use fret_ui::{ElementContext, Invalidation, Theme, UiHost};

use crate::primitives::colors::{
    editor_accent, editor_border, editor_focus_ring, editor_foreground, editor_muted_foreground,
    editor_subtle_bg,
};
use crate::primitives::input_group::derived_test_id;
use crate::primitives::style::EditorStyle;
use crate::theme::{
    EDITOR_THEME_PRESETS, EditorThemePreset, install_editor_theme_preset,
    installed_editor_theme_preset,
};

use super::EditorThemePresetPickerOptions;
use super::render::{EditorThemePresetPickerRenderInput, build_editor_theme_preset_picker_element};

#[derive(Clone)]
pub struct EditorThemePresetPicker {
    model: Model<EditorThemePreset>,
    options: EditorThemePresetPickerOptions,
}

impl EditorThemePresetPicker {
    pub fn new(model: Model<EditorThemePreset>) -> Self {
        Self {
            model,
            options: EditorThemePresetPickerOptions::default(),
        }
    }

    pub fn options(mut self, options: EditorThemePresetPickerOptions) -> Self {
        self.options = options;
        self
    }

    #[track_caller]
    pub fn into_element<H: UiHost>(self, cx: &mut ElementContext<'_, H>) -> AnyElement {
        let selected = cx
            .get_model_copied(&self.model, Invalidation::Paint)
            .unwrap_or_default();

        if installed_editor_theme_preset(&*cx.app) != Some(selected) {
            install_editor_theme_preset(cx.app, selected);
        }

        let (density, row_height, border, ring, fg, muted_fg, subtle_bg, accent, text_px) = {
            let theme = Theme::global(&*cx.app);
            let style = EditorStyle::resolve(theme);
            (
                style.density,
                style.density.row_height,
                editor_border(theme),
                editor_focus_ring(theme),
                editor_accent(theme),
                editor_foreground(theme),
                editor_muted_foreground(theme),
                editor_subtle_bg(theme),
                style.frame_chrome_small().text_px,
            )
        };

        let label = self
            .options
            .label
            .clone()
            .unwrap_or_else(|| Arc::from("Editor theme preset"));
        let item_prefix = self
            .options
            .item_test_id_prefix
            .clone()
            .or_else(|| derived_test_id(self.options.test_id.as_ref(), "item"));
        let options = self.options.clone();
        let model = self.model.clone();

        build_editor_theme_preset_picker_element(
            cx,
            EditorThemePresetPickerRenderInput {
                selected,
                label,
                item_prefix,
                options,
                model,
                total: EDITOR_THEME_PRESETS.len(),
                row_height,
                padding_x: density.padding_x,
                border,
                ring,
                fg,
                muted_fg,
                subtle_bg,
                accent,
                text_px,
            },
        )
    }
}
