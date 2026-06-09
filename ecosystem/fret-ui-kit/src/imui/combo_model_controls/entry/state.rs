use std::sync::Arc;

use fret_runtime::Model;
use fret_ui::UiHost;

use crate::imui::{ComboModelOptions, UiWriterImUiFacadeExt};

pub(super) struct ComboModelEntryState {
    pub(super) enabled: bool,
    pub(super) selected: Option<Arc<str>>,
    pub(super) preview: Arc<str>,
    pub(super) popup_open: Model<bool>,
}

pub(super) fn read_combo_model_entry_state<H: UiHost, W: UiWriterImUiFacadeExt<H> + ?Sized>(
    ui: &mut W,
    id: &str,
    model: &Model<Option<Arc<str>>>,
    options: &ComboModelOptions,
) -> ComboModelEntryState {
    let enabled = options.enabled && ui.with_cx_mut(|cx| !crate::imui::imui_is_disabled(cx));
    let selected = read_combo_model_selected(ui, model);
    let preview = combo_model_preview(selected.clone(), options.placeholder.clone());
    let popup_open = ui.popup_open_model(id);

    ComboModelEntryState {
        enabled,
        selected,
        preview,
        popup_open,
    }
}

fn read_combo_model_selected<H: UiHost, W: UiWriterImUiFacadeExt<H> + ?Sized>(
    ui: &mut W,
    model: &Model<Option<Arc<str>>>,
) -> Option<Arc<str>> {
    ui.with_cx_mut(|cx| {
        cx.read_model(model, fret_ui::Invalidation::Paint, |_app, value| {
            value.clone()
        })
        .unwrap_or(None)
    })
}

fn combo_model_preview(selected: Option<Arc<str>>, placeholder: Option<Arc<str>>) -> Arc<str> {
    selected
        .or(placeholder)
        .unwrap_or_else(|| Arc::from("Select..."))
}
