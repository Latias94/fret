use std::sync::Arc;

use fret_runtime::{CommandId, Model};
use fret_ui::element::AnyElement;
use fret_ui::{ElementContext, UiHost};

use crate::bool_model::IntoBoolModel;
use crate::button::{Button, ButtonVariant};
use crate::dropdown_menu::{DropdownMenu, DropdownMenuCheckboxItem, DropdownMenuEntry};
use crate::input::Input;
use crate::text_value_model::IntoTextValueModel;

#[derive(Debug, Clone)]
pub struct DataTableRowState {
    pub selected: bool,
    pub enabled: bool,
    pub on_click: Option<CommandId>,
}

impl Default for DataTableRowState {
    fn default() -> Self {
        Self {
            selected: false,
            enabled: true,
            on_click: None,
        }
    }
}

#[derive(Debug, Clone)]
pub struct DataTableColumnOption {
    pub id: Arc<str>,
    pub label: Arc<str>,
    pub hideable: bool,
}

impl DataTableColumnOption {
    pub fn new(id: impl Into<Arc<str>>, label: impl Into<Arc<str>>) -> Self {
        Self {
            id: id.into(),
            label: label.into(),
            hideable: true,
        }
    }

    pub fn hideable(mut self, hideable: bool) -> Self {
        self.hideable = hideable;
        self
    }
}

#[derive(Debug, Clone)]
pub struct DataTableViewOptionItem {
    pub label: Arc<str>,
    pub checked: Model<bool>,
    pub disabled: bool,
}

impl DataTableViewOptionItem {
    pub fn new(checked: impl IntoBoolModel, label: impl Into<Arc<str>>) -> Self {
        Self {
            label: label.into(),
            checked: checked.into_bool_model(),
            disabled: false,
        }
    }

    pub fn disabled(mut self, disabled: bool) -> Self {
        self.disabled = disabled;
        self
    }
}

#[derive(Debug, Clone)]
pub struct DataTableViewOptions {
    pub open: Model<bool>,
    pub items: Vec<DataTableViewOptionItem>,
}

impl DataTableViewOptions {
    pub fn new(
        open: impl IntoBoolModel,
        items: impl IntoIterator<Item = DataTableViewOptionItem>,
    ) -> Self {
        Self {
            open: open.into_bool_model(),
            items: items.into_iter().collect(),
        }
    }

    #[track_caller]
    pub fn into_element<H: UiHost>(self, cx: &mut ElementContext<'_, H>) -> AnyElement {
        let open = self.open;
        let items = self.items;

        DropdownMenu::from_open(open).build(
            cx,
            Button::new("Columns")
                .variant(ButtonVariant::Outline)
                .trailing_icon(fret_icons::IconId::new_static("lucide.chevron-down")),
            move |_cx| {
                items
                    .iter()
                    .cloned()
                    .map(|it| {
                        DropdownMenuEntry::CheckboxItem(
                            DropdownMenuCheckboxItem::new(it.checked, it.label)
                                .disabled(it.disabled),
                        )
                    })
                    .collect::<Vec<_>>()
            },
        )
    }
}

#[derive(Debug, Clone)]
pub struct DataTableGlobalFilterInput {
    pub model: Model<String>,
    pub placeholder: Arc<str>,
}

impl DataTableGlobalFilterInput {
    pub fn new(model: impl IntoTextValueModel) -> Self {
        Self {
            model: model.into_text_value_model(),
            placeholder: Arc::from("Filter..."),
        }
    }

    pub fn placeholder(mut self, placeholder: impl Into<Arc<str>>) -> Self {
        self.placeholder = placeholder.into();
        self
    }

    #[track_caller]
    pub fn into_element<H: UiHost>(self, cx: &mut ElementContext<'_, H>) -> AnyElement {
        Input::new(self.model)
            .placeholder(self.placeholder)
            .into_element(cx)
    }
}
