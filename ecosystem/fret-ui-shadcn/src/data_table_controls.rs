use std::collections::HashMap;
use std::sync::Arc;

use fret_core::Px;
use fret_icons::IconId;
use fret_runtime::{CommandId, Model};
use fret_ui::element::{AnyElement, LayoutStyle, SpacerProps};
use fret_ui::{ElementContext, UiHost};
use fret_ui_headless::table::{ColumnDef, ColumnId, TableState};
use fret_ui_kit::declarative::model_watch::ModelWatchExt as _;

use crate::bool_model::IntoBoolModel;
use crate::button::{Button, ButtonSize, ButtonVariant};
use crate::dropdown_menu::{
    DropdownMenu, DropdownMenuAlign, DropdownMenuCheckboxItem, DropdownMenuEntry, DropdownMenuLabel,
};
use crate::input::Input;
use crate::text_value_model::IntoTextValueModel;

pub(crate) fn is_column_visible(state: &TableState, id: &ColumnId) -> bool {
    state.column_visibility.get(id).copied().unwrap_or(true)
}

pub(crate) fn apply_column_visibility_change(
    state: &mut TableState,
    desired: &HashMap<ColumnId, bool>,
) -> bool {
    let mut changed = false;
    for (id, visible) in desired {
        let current = is_column_visible(state, id);
        if current == *visible {
            continue;
        }
        changed = true;
        if *visible {
            state.column_visibility.remove(id);
        } else {
            state.column_visibility.insert(id.clone(), false);
        }
    }

    if changed {
        state.pagination.page_index = 0;
    }
    changed
}

pub(crate) fn sync_column_visibility(
    app: &mut impl UiHost,
    state: &Model<TableState>,
    desired: &HashMap<ColumnId, bool>,
) {
    let _ = app.models_mut().update(state, |st| {
        let _ = apply_column_visibility_change(st, desired);
    });
}

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

fn hidden_element<H: UiHost>(cx: &mut ElementContext<'_, H>) -> AnyElement {
    cx.spacer(SpacerProps {
        layout: LayoutStyle::default(),
        min: Px(0.0),
    })
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum DataTableViewOptionsVisibility {
    #[default]
    AlwaysVisible,
    HideBelowLg,
}

#[derive(Debug, Default)]
struct DataTableViewOptionsRuntime {
    last_visibility: HashMap<ColumnId, bool>,
}

#[derive(Debug, Clone)]
struct DataTableViewOptionBinding {
    id: ColumnId,
    label: Arc<str>,
    model: Model<bool>,
    disabled: bool,
}

#[derive(Debug, Clone)]
pub struct DataTableViewOptions {
    pub open: Model<bool>,
    pub items: Vec<DataTableViewOptionItem>,
    bound_state: Option<Model<TableState>>,
    bound_options: Arc<[DataTableColumnOption]>,
    button_label: Arc<str>,
    menu_label: Option<Arc<str>>,
    align: DropdownMenuAlign,
    button_variant: ButtonVariant,
    button_size: ButtonSize,
    leading_icon: Option<IconId>,
    menu_min_width: Option<Px>,
    visibility: DataTableViewOptionsVisibility,
}

impl DataTableViewOptions {
    pub fn new(
        open: impl IntoBoolModel,
        items: impl IntoIterator<Item = DataTableViewOptionItem>,
    ) -> Self {
        Self {
            open: open.into_bool_model(),
            items: items.into_iter().collect(),
            bound_state: None,
            bound_options: Arc::from(Vec::<DataTableColumnOption>::new().into_boxed_slice()),
            button_label: Arc::from("Columns"),
            menu_label: None,
            align: DropdownMenuAlign::default(),
            button_variant: ButtonVariant::Outline,
            button_size: ButtonSize::Default,
            leading_icon: None,
            menu_min_width: None,
            visibility: DataTableViewOptionsVisibility::AlwaysVisible,
        }
    }

    pub fn from_column_options(
        open: impl IntoBoolModel,
        state: Model<TableState>,
        options: impl Into<Arc<[DataTableColumnOption]>>,
    ) -> Self {
        Self {
            open: open.into_bool_model(),
            items: Vec::new(),
            bound_state: Some(state),
            bound_options: options.into(),
            button_label: Arc::from("View"),
            menu_label: Some(Arc::from("Toggle columns")),
            align: DropdownMenuAlign::End,
            button_variant: ButtonVariant::Outline,
            button_size: ButtonSize::Sm,
            leading_icon: Some(IconId::new_static("lucide.settings-2")),
            // Upstream shadcn/ui tasks `DataTableViewOptions` sets
            // `DropdownMenuContent className="w-[150px]"`.
            menu_min_width: Some(Px(150.0)),
            // Upstream tasks `DataTableViewOptions` also sets
            // `className="ml-auto hidden h-8 lg:flex"` on the trigger button.
            visibility: DataTableViewOptionsVisibility::HideBelowLg,
        }
    }

    pub fn from_table_state<TData>(
        open: impl IntoBoolModel,
        state: Model<TableState>,
        columns: impl Into<Arc<[ColumnDef<TData>]>>,
        column_label: impl Fn(&ColumnDef<TData>) -> Arc<str>,
    ) -> Self {
        fn push_leaf_column_options<TData, F>(
            columns: &[ColumnDef<TData>],
            out: &mut Vec<DataTableColumnOption>,
            column_label: &F,
        ) where
            F: Fn(&ColumnDef<TData>) -> Arc<str>,
        {
            for col in columns {
                if col.columns.is_empty() {
                    out.push(
                        DataTableColumnOption::new(col.id.clone(), column_label(col))
                            .hideable(col.enable_hiding),
                    );
                } else {
                    push_leaf_column_options(&col.columns, out, column_label);
                }
            }
        }

        let columns: Arc<[ColumnDef<TData>]> = columns.into();
        let mut options_vec = Vec::new();
        push_leaf_column_options(&columns, &mut options_vec, &column_label);
        let options: Arc<[DataTableColumnOption]> = Arc::from(options_vec.into_boxed_slice());
        Self::from_column_options(open, state, options)
    }

    pub fn button_label(mut self, label: impl Into<Arc<str>>) -> Self {
        self.button_label = label.into();
        self
    }

    pub fn menu_label(mut self, label: impl Into<Arc<str>>) -> Self {
        self.menu_label = Some(label.into());
        self
    }

    pub fn no_menu_label(mut self) -> Self {
        self.menu_label = None;
        self
    }

    pub fn align(mut self, align: DropdownMenuAlign) -> Self {
        self.align = align;
        self
    }

    pub fn button_variant(mut self, variant: ButtonVariant) -> Self {
        self.button_variant = variant;
        self
    }

    pub fn button_size(mut self, size: ButtonSize) -> Self {
        self.button_size = size;
        self
    }

    pub fn leading_icon(mut self, icon: IconId) -> Self {
        self.leading_icon = Some(icon);
        self
    }

    pub fn menu_min_width(mut self, min_width: Px) -> Self {
        self.menu_min_width = Some(min_width);
        self
    }

    pub fn visibility(mut self, visibility: DataTableViewOptionsVisibility) -> Self {
        self.visibility = visibility;
        self
    }

    pub fn always_visible(mut self) -> Self {
        self.visibility = DataTableViewOptionsVisibility::AlwaysVisible;
        self
    }

    pub fn hide_below_lg(mut self) -> Self {
        self.visibility = DataTableViewOptionsVisibility::HideBelowLg;
        self
    }

    fn trigger_button(&self) -> Button {
        let mut button = Button::new(self.button_label.clone())
            .variant(self.button_variant)
            .size(self.button_size)
            .trailing_icon(fret_icons::IconId::new_static("lucide.chevron-down"));
        if let Some(icon) = self.leading_icon.clone() {
            button = button.leading_icon(icon);
        }
        button
    }

    #[track_caller]
    pub fn into_element<H: UiHost>(self, cx: &mut ElementContext<'_, H>) -> AnyElement {
        let show_trigger = match self.visibility {
            DataTableViewOptionsVisibility::AlwaysVisible => true,
            DataTableViewOptionsVisibility::HideBelowLg => {
                fret_ui_kit::declarative::viewport_width_at_least(
                    cx,
                    fret_ui::Invalidation::Layout,
                    fret_ui_kit::declarative::viewport_tailwind::LG,
                    fret_ui_kit::declarative::ViewportQueryHysteresis::default(),
                )
            }
        };
        if !show_trigger {
            return hidden_element(cx);
        }

        let button = self.trigger_button();
        let open = self.open;
        let items = self.items;
        let bound_state = self.bound_state;
        let bound_options = self.bound_options;
        let align = self.align;
        let menu_label = self.menu_label;
        let menu_min_width = self.menu_min_width;

        let bound_entries = bound_state.map(|state| {
            let state_value = cx.watch_model(&state).layout().cloned().unwrap_or_default();
            let runtime_id = cx.keyed_slot_id("data_table_view_options_runtime");

            let bindings: Vec<DataTableViewOptionBinding> = bound_options
                .iter()
                .filter(|opt| opt.hideable)
                .map(|opt| DataTableViewOptionBinding {
                    id: opt.id.clone(),
                    label: opt.label.clone(),
                    model: cx.local_model_keyed(("column_visibility", opt.id.clone()), || {
                        is_column_visible(&state_value, &opt.id)
                    }),
                    disabled: !opt.hideable,
                })
                .collect();

            let desired_visibility_from_state: HashMap<ColumnId, bool> = bindings
                .iter()
                .map(|binding| {
                    (
                        binding.id.clone(),
                        is_column_visible(&state_value, &binding.id),
                    )
                })
                .collect();

            let should_sync_visibility =
                cx.state_for(runtime_id, DataTableViewOptionsRuntime::default, |st| {
                    let changed = st.last_visibility != desired_visibility_from_state;
                    if changed {
                        st.last_visibility = desired_visibility_from_state.clone();
                    }
                    changed
                });
            if should_sync_visibility {
                for binding in bindings.iter() {
                    let desired = desired_visibility_from_state
                        .get(&binding.id)
                        .copied()
                        .unwrap_or(true);
                    let _ = cx.app.models_mut().update(&binding.model, |v| *v = desired);
                }
            }

            let desired_visibility: HashMap<ColumnId, bool> = bindings
                .iter()
                .map(|binding| {
                    (
                        binding.id.clone(),
                        cx.watch_model(&binding.model)
                            .layout()
                            .copied()
                            .unwrap_or(true),
                    )
                })
                .collect();
            sync_column_visibility(&mut *cx.app, &state, &desired_visibility);

            bindings
        });

        let mut menu = DropdownMenu::from_open(open).align(align);
        if let Some(min_width) = menu_min_width {
            menu = menu.min_width(min_width);
        }

        menu.build(cx, button, move |_cx| {
            let mut entries = Vec::new();

            match bound_entries.as_ref() {
                Some(bindings) => {
                    if let Some(label) = menu_label.clone() {
                        entries.push(DropdownMenuEntry::Label(
                            DropdownMenuLabel::new(label).inset(true),
                        ));
                        if !bindings.is_empty() {
                            entries.push(DropdownMenuEntry::Separator);
                        }
                    }

                    entries.extend(bindings.iter().cloned().map(|binding| {
                        DropdownMenuEntry::CheckboxItem(
                            DropdownMenuCheckboxItem::new(binding.model, binding.label)
                                .disabled(binding.disabled),
                        )
                    }));
                }
                None => {
                    if let Some(label) = menu_label.clone() {
                        entries.push(DropdownMenuEntry::Label(
                            DropdownMenuLabel::new(label).inset(true),
                        ));
                        if !items.is_empty() {
                            entries.push(DropdownMenuEntry::Separator);
                        }
                    }

                    entries.extend(items.iter().cloned().map(|it| {
                        DropdownMenuEntry::CheckboxItem(
                            DropdownMenuCheckboxItem::new(it.checked, it.label)
                                .disabled(it.disabled),
                        )
                    }));
                }
            }

            entries
        })
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
