//! Material 3 exposed dropdown (searchable select).
//!
//! Outcome-oriented composition over `Autocomplete`:
//! - selection is committed into `selected_value` (the option `value`),
//! - the text field stays editable for filtering,
//! - when the input is not focused, the query is synchronized from the committed selection.

use std::cell::Cell;
use std::rc::Rc;
use std::sync::Arc;

use fret_icons::IconId;
use fret_runtime::Model;
use fret_ui::action::OnPressablePointerDown;
use fret_ui::element::AnyElement;
use fret_ui::elements::{ElementContext, GlobalElementId};
use fret_ui::{Invalidation, UiHost};

use crate::{Autocomplete, AutocompleteItem, AutocompleteVariant, OnAutocompleteSelect};

#[derive(Clone)]
pub struct ExposedDropdown {
    selected_value: Model<Option<Arc<str>>>,
    query: Option<Model<String>>,
    items: Arc<[AutocompleteItem]>,
    variant: AutocompleteVariant,
    open_on_focus: bool,
    sync_query_from_selected_on_blur: bool,
    on_select: Option<OnAutocompleteSelect>,
    leading_icon: Option<IconId>,
    leading_icon_a11y_label: Option<Arc<str>>,
    leading_icon_test_id: Option<Arc<str>>,
    on_leading_icon_pointer_down: Option<OnPressablePointerDown>,
    disabled: bool,
    error: bool,
    label: Option<Arc<str>>,
    placeholder: Option<Arc<str>>,
    supporting_text: Option<Arc<str>>,
    a11y_label: Option<Arc<str>>,
    test_id: Option<Arc<str>>,
}

impl std::fmt::Debug for ExposedDropdown {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ExposedDropdown")
            .field("variant", &self.variant)
            .field("open_on_focus", &self.open_on_focus)
            .field(
                "sync_query_from_selected_on_blur",
                &self.sync_query_from_selected_on_blur,
            )
            .field("disabled", &self.disabled)
            .field("error", &self.error)
            .field(
                "leading_icon",
                &self.leading_icon.as_ref().map(|i| i.as_str()),
            )
            .field("label", &self.label)
            .field("placeholder", &self.placeholder)
            .field("supporting_text", &self.supporting_text)
            .field("a11y_label", &self.a11y_label)
            .field("test_id", &self.test_id)
            .finish()
    }
}

impl ExposedDropdown {
    pub fn new(selected_value: Model<Option<Arc<str>>>) -> Self {
        Self {
            selected_value,
            query: None,
            items: Arc::from([]),
            variant: AutocompleteVariant::default(),
            open_on_focus: false,
            sync_query_from_selected_on_blur: true,
            on_select: None,
            leading_icon: None,
            leading_icon_a11y_label: None,
            leading_icon_test_id: None,
            on_leading_icon_pointer_down: None,
            disabled: false,
            error: false,
            label: None,
            placeholder: None,
            supporting_text: None,
            a11y_label: None,
            test_id: None,
        }
    }

    pub fn query(mut self, query: Model<String>) -> Self {
        self.query = Some(query);
        self
    }

    pub fn items(mut self, items: impl Into<Arc<[AutocompleteItem]>>) -> Self {
        self.items = items.into();
        self
    }

    pub fn variant(mut self, variant: AutocompleteVariant) -> Self {
        self.variant = variant;
        self
    }

    pub fn open_on_focus(mut self, open_on_focus: bool) -> Self {
        self.open_on_focus = open_on_focus;
        self
    }

    pub fn sync_query_from_selected_on_blur(mut self, sync: bool) -> Self {
        self.sync_query_from_selected_on_blur = sync;
        self
    }

    pub fn on_select(mut self, on_select: OnAutocompleteSelect) -> Self {
        self.on_select = Some(on_select);
        self
    }

    pub fn leading_icon(mut self, icon: IconId) -> Self {
        self.leading_icon = Some(icon);
        self
    }

    pub fn leading_icon_a11y_label(mut self, label: impl Into<Arc<str>>) -> Self {
        self.leading_icon_a11y_label = Some(label.into());
        self
    }

    pub fn leading_icon_test_id(mut self, id: impl Into<Arc<str>>) -> Self {
        self.leading_icon_test_id = Some(id.into());
        self
    }

    pub fn on_leading_icon_pointer_down(mut self, on_pointer_down: OnPressablePointerDown) -> Self {
        self.on_leading_icon_pointer_down = Some(on_pointer_down);
        self
    }

    pub fn label(mut self, label: impl Into<Arc<str>>) -> Self {
        self.label = Some(label.into());
        self
    }

    pub fn placeholder(mut self, placeholder: impl Into<Arc<str>>) -> Self {
        self.placeholder = Some(placeholder.into());
        self
    }

    pub fn supporting_text(mut self, text: impl Into<Arc<str>>) -> Self {
        self.supporting_text = Some(text.into());
        self
    }

    pub fn disabled(mut self, disabled: bool) -> Self {
        self.disabled = disabled;
        self
    }

    pub fn error(mut self, error: bool) -> Self {
        self.error = error;
        self
    }

    pub fn a11y_label(mut self, label: impl Into<Arc<str>>) -> Self {
        self.a11y_label = Some(label.into());
        self
    }

    pub fn test_id(mut self, id: impl Into<Arc<str>>) -> Self {
        self.test_id = Some(id.into());
        self
    }

    #[track_caller]
    pub fn into_element<H: UiHost>(self, cx: &mut ElementContext<'_, H>) -> AnyElement {
        exposed_dropdown_into_element(cx, self)
    }
}

fn label_for_value<'a>(items: &'a [AutocompleteItem], value: &str) -> Option<&'a Arc<str>> {
    items
        .iter()
        .find(|it| it.value.as_ref() == value)
        .map(|it| &it.label)
}

fn exposed_dropdown_into_element<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
    exposed: ExposedDropdown,
) -> AnyElement {
    cx.scope(|cx| {
        let query = match exposed.query.clone() {
            Some(v) => v,
            None => cx.local_model(String::new),
        };

        let input_id_out = cx.slot_state(
            || Rc::new(Cell::new(None::<GlobalElementId>)),
            |id| id.clone(),
        );

        let selected_value = cx
            .get_model_cloned(&exposed.selected_value, Invalidation::Layout)
            .unwrap_or(None);

        let focused_input = input_id_out
            .get()
            .is_some_and(|id| cx.is_focused_element(id));

        if exposed.sync_query_from_selected_on_blur && !focused_input {
            let desired = match selected_value.as_ref() {
                None => String::new(),
                Some(v) => label_for_value(&exposed.items, v.as_ref())
                    .map(|label| label.as_ref().to_string())
                    .unwrap_or_else(|| v.as_ref().to_string()),
            };

            let current = cx
                .get_model_cloned(&query, Invalidation::Layout)
                .unwrap_or_default();
            if current != desired {
                let _ = cx.app.models_mut().update(&query, |v| *v = desired);
                cx.app.request_redraw(cx.window);
            }
        }

        let mut ac = Autocomplete::new(query)
            .selected_value(exposed.selected_value)
            .items(exposed.items)
            .variant(exposed.variant)
            .open_on_focus(exposed.open_on_focus)
            .set_query_on_select(true)
            .trailing_dropdown_icon(true)
            .input_id_out(input_id_out)
            .disabled(exposed.disabled)
            .error(exposed.error);

        if let Some(icon) = exposed.leading_icon {
            ac = ac.leading_icon(icon);
            if let Some(label) = exposed.leading_icon_a11y_label {
                ac = ac.leading_icon_a11y_label(label);
            }
            if let Some(id) = exposed.leading_icon_test_id {
                ac = ac.leading_icon_test_id(id);
            }
            if let Some(handler) = exposed.on_leading_icon_pointer_down {
                ac = ac.on_leading_icon_pointer_down(handler);
            }
        }

        if let Some(label) = exposed.label {
            ac = ac.label(label);
        }
        if let Some(placeholder) = exposed.placeholder {
            ac = ac.placeholder(placeholder);
        }
        if let Some(text) = exposed.supporting_text {
            ac = ac.supporting_text(text);
        }
        if let Some(label) = exposed.a11y_label {
            ac = ac.a11y_label(label);
        }
        if let Some(id) = exposed.test_id {
            ac = ac.test_id(id);
        }
        if let Some(on_select) = exposed.on_select {
            ac = ac.on_select(on_select);
        }

        ac.into_element(cx)
    })
}
