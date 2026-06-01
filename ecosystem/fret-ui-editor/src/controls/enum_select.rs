//! Filterable enum select control (editor-grade combobox-like widget).
//!
//! This is an ecosystem/policy control:
//! - it uses `fret-ui` mechanisms (pressable, focus, overlays),
//! - and `fret-ui-kit` infrastructure (overlay controller + popper placement),
//! - without depending on any design-system crate.

use std::panic::Location;
use std::sync::{Arc, Mutex};

use fret_runtime::Model;
use fret_ui::element::AnyElement;
use fret_ui::elements::GlobalElementId;
use fret_ui::{ElementContext, Invalidation, Theme, UiHost};
use fret_ui_kit::primitives::combobox as kit_combobox;

use crate::primitives::popup_surface::resolve_editor_popup_surface_chrome;
use crate::primitives::style::EditorStyle;

mod options;
mod overlay;
mod row;
mod trigger;
mod trigger_keys;

pub use options::EnumSelectOptions;

use trigger::{EnumSelectTriggerArgs, enum_select_trigger};
use trigger_keys::enum_select_trigger_open_keys;

#[derive(Debug, Clone)]
pub struct EnumSelectItem {
    pub value: Arc<str>,
    pub label: Arc<str>,
}

impl EnumSelectItem {
    pub fn new(value: impl Into<Arc<str>>, label: impl Into<Arc<str>>) -> Self {
        Self {
            value: value.into(),
            label: label.into(),
        }
    }
}

#[derive(Clone)]
pub struct EnumSelect {
    model: Model<Option<Arc<str>>>,
    items: Arc<[EnumSelectItem]>,
    options: EnumSelectOptions,
}

impl EnumSelect {
    pub fn new(model: Model<Option<Arc<str>>>, items: Arc<[EnumSelectItem]>) -> Self {
        Self {
            model,
            items,
            options: EnumSelectOptions::default(),
        }
    }

    pub fn options(mut self, options: EnumSelectOptions) -> Self {
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
            cx.keyed(("fret-ui-editor.enum_select", id_source, model_id), |cx| {
                self.into_element_keyed(cx)
            })
        } else {
            cx.keyed(("fret-ui-editor.enum_select", callsite, model_id), |cx| {
                self.into_element_keyed(cx)
            })
        }
    }

    fn into_element_keyed<H: UiHost>(self, cx: &mut ElementContext<'_, H>) -> AnyElement {
        let open = open_model(cx);
        let filter = filter_model(cx);
        let open_change_reason = cx.local_model_keyed("open_change_reason", || {
            None::<kit_combobox::ComboboxOpenChangeReason>
        });
        let focus_restore_target = cx.slot_state(
            || Arc::new(Mutex::new(None::<GlobalElementId>)),
            |cell| cell.clone(),
        );

        let is_open = cx
            .get_model_copied(&open, Invalidation::Layout)
            .unwrap_or(false);

        let should_clear_filter = cx
            .slot_state(kit_combobox::ClearQueryOnCloseState::default, |state| {
                kit_combobox::should_clear_query_on_close(state, is_open)
            });
        if should_clear_filter {
            let _ = cx.app.models_mut().update(&filter, |s| s.clear());
        }

        let selected_value = cx
            .get_model_cloned(&self.model, Invalidation::Paint)
            .unwrap_or(None);

        let (density, frame_chrome, ring, popup_chrome) = {
            let theme = Theme::global(&*cx.app);
            let style = EditorStyle::resolve(theme);
            let density = style.density;
            let frame_chrome = style.frame_chrome_small();
            let ring = theme
                .color_by_key("ring")
                .unwrap_or_else(|| theme.color_token("primary"));
            let popup_chrome = resolve_editor_popup_surface_chrome(theme, true);
            (density, frame_chrome, ring, popup_chrome)
        };

        let selected_label = selected_value
            .as_deref()
            .and_then(|v| self.items.iter().find(|it| it.value.as_ref() == v))
            .map(|it| it.label.clone());

        let trigger_text = match (selected_value.as_ref(), selected_label.as_ref()) {
            (Some(_), Some(label)) => label.clone(),
            (Some(v), None) => Arc::from(format!("<unknown: {v}>")),
            (None, _) => self.options.none_label.clone(),
        };

        let trigger_model = self.model.clone();
        let items_for_overlay = self.items.clone();
        let options_for_overlay = self.options.clone();
        let trigger = enum_select_trigger(
            cx,
            EnumSelectTriggerArgs {
                layout: self.options.layout,
                enabled: self.options.enabled,
                focusable: self.options.focusable,
                a11y_label: self.options.a11y_label.clone(),
                density,
                frame_chrome,
                ring,
                is_open,
                trigger_text,
                open: open.clone(),
                open_change_reason: open_change_reason.clone(),
            },
        );

        let trigger_id = trigger.id;
        *focus_restore_target
            .lock()
            .unwrap_or_else(|e| e.into_inner()) = Some(trigger_id);

        let on_trigger_open_keys = enum_select_trigger_open_keys(
            self.options.enabled,
            open.clone(),
            open_change_reason.clone(),
        );
        cx.key_add_on_key_down_capture_for(trigger_id, on_trigger_open_keys);

        if let Some(test_id) = self.options.test_id.as_ref() {
            // Attach on the returned element, not inside the pressable body, to keep the trigger
            // subtree stable across internal composition changes.
            let trigger = trigger.test_id(test_id.clone());
            overlay::request_overlay(
                cx,
                trigger_id,
                trigger_model,
                items_for_overlay,
                open,
                filter,
                open_change_reason,
                focus_restore_target,
                options_for_overlay,
                density,
                popup_chrome,
            );
            trigger
        } else {
            overlay::request_overlay(
                cx,
                trigger_id,
                trigger_model,
                items_for_overlay,
                open,
                filter,
                open_change_reason,
                focus_restore_target,
                options_for_overlay,
                density,
                popup_chrome,
            );
            trigger
        }
    }
}

#[track_caller]
fn open_model<H: UiHost>(cx: &mut ElementContext<'_, H>) -> Model<bool> {
    cx.local_model(|| false)
}

#[track_caller]
fn filter_model<H: UiHost>(cx: &mut ElementContext<'_, H>) -> Model<String> {
    cx.local_model(String::new)
}
