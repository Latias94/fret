use std::collections::HashMap;
use std::sync::Arc;

use fret_core::SemanticsRole;
use fret_runtime::Model;
use fret_ui::action::OnActivate;
use fret_ui::element::AnyElement;
use fret_ui::{ElementContext, Theme, UiHost};
use fret_ui_headless::table::{ColumnDef, ColumnId, ColumnPinPosition, TableState, pin_column};
use fret_ui_kit::declarative::model_watch::ModelWatchExt as _;
use fret_ui_kit::declarative::stack::{HStackProps, hstack};
use fret_ui_kit::declarative::table::TableViewOutput;
use fret_ui_kit::{ColorRef, LayoutRefinement, Space, ui};
use serde_json::Value;

use crate::button::{Button, ButtonSize, ButtonVariant};
use crate::dropdown_menu::{
    DropdownMenu, DropdownMenuCheckboxItem, DropdownMenuEntry, DropdownMenuLabel,
    DropdownMenuRadioGroup, DropdownMenuRadioItemSpec,
};
use crate::input::Input;

fn is_column_visible(state: &TableState, id: &ColumnId) -> bool {
    state.column_visibility.get(id).copied().unwrap_or(true)
}

fn column_pin_position(state: &TableState, id: &ColumnId) -> Option<ColumnPinPosition> {
    if state
        .column_pinning
        .left
        .iter()
        .any(|c| c.as_ref() == id.as_ref())
    {
        return Some(ColumnPinPosition::Left);
    }
    if state
        .column_pinning
        .right
        .iter()
        .any(|c| c.as_ref() == id.as_ref())
    {
        return Some(ColumnPinPosition::Right);
    }
    None
}

fn pin_position_model_value(state: &TableState, id: &ColumnId) -> Arc<str> {
    match column_pin_position(state, id) {
        None => Arc::from("none"),
        Some(ColumnPinPosition::Left) => Arc::from("left"),
        Some(ColumnPinPosition::Right) => Arc::from("right"),
    }
}

fn pin_position_from_model_value(value: Option<&Arc<str>>) -> Option<ColumnPinPosition> {
    match value.map(|v| v.as_ref()) {
        Some("left") => Some(ColumnPinPosition::Left),
        Some("right") => Some(ColumnPinPosition::Right),
        _ => None,
    }
}

fn normalized_global_filter(value: &str) -> Option<Value> {
    let next = value.trim();
    if next.is_empty() {
        None
    } else {
        Some(Value::String(next.to_string()))
    }
}

fn apply_global_filter_change(state: &mut TableState, value: &str) -> bool {
    let next = normalized_global_filter(value);
    if state.global_filter == next {
        return false;
    }
    state.global_filter = next;
    state.pagination.page_index = 0;
    true
}

fn sync_global_filter<H: UiHost>(app: &mut H, state: &Model<TableState>, value: &str) {
    let _ = app.models_mut().update(state, |st| {
        let _ = apply_global_filter_change(st, value);
    });
}

fn apply_column_visibility_change(
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

fn sync_column_visibility(
    app: &mut impl UiHost,
    state: &Model<TableState>,
    desired: &HashMap<ColumnId, bool>,
) {
    let _ = app.models_mut().update(state, |st| {
        let _ = apply_column_visibility_change(st, desired);
    });
}

fn apply_column_pinning_change(
    state: &mut TableState,
    desired: &HashMap<ColumnId, Option<ColumnPinPosition>>,
) -> bool {
    let mut changed = false;
    for (id, desired_position) in desired {
        let current = column_pin_position(state, id);
        if current == *desired_position {
            continue;
        }
        changed = true;
        pin_column(&mut state.column_pinning, id, *desired_position);
    }

    if changed {
        state.pagination.page_index = 0;
    }

    changed
}

fn sync_column_pinning(
    app: &mut impl UiHost,
    state: &Model<TableState>,
    desired: &HashMap<ColumnId, Option<ColumnPinPosition>>,
) {
    let _ = app.models_mut().update(state, |st| {
        let _ = apply_column_pinning_change(st, desired);
    });
}

struct ColumnVisibilityBinding {
    id: ColumnId,
    model: Model<bool>,
}

impl Clone for ColumnVisibilityBinding {
    fn clone(&self) -> Self {
        Self {
            id: self.id.clone(),
            model: self.model.clone(),
        }
    }
}

struct ColumnPinningBinding {
    id: ColumnId,
    model: Model<Option<Arc<str>>>,
}

impl Clone for ColumnPinningBinding {
    fn clone(&self) -> Self {
        Self {
            id: self.id.clone(),
            model: self.model.clone(),
        }
    }
}

#[derive(Clone)]
struct FacetedFilterConfig {
    column_id: ColumnId,
    button_label: Arc<str>,
    options: Arc<[Arc<str>]>,
}

struct FacetedFilterItemBinding {
    value: Arc<str>,
    model: Model<bool>,
}

impl Clone for FacetedFilterItemBinding {
    fn clone(&self) -> Self {
        Self {
            value: self.value.clone(),
            model: self.model.clone(),
        }
    }
}

#[derive(Default)]
struct DataTableToolbarState {
    filter_model: Option<Model<String>>,
    column_filter_model: Option<Model<String>>,
    columns_open: Option<Model<bool>>,
    pinning_open: Option<Model<bool>>,
    faceted_open: Option<Model<bool>>,
    faceted_column: Option<ColumnId>,
    faceted_items: Vec<FacetedFilterItemBinding>,
    column_visibility: Vec<ColumnVisibilityBinding>,
    column_pinning: Vec<ColumnPinningBinding>,
    last_synced_state_revision: Option<u64>,
}

/// shadcn/ui `DataTable` toolbar (recipe).
///
/// This is a v1 convenience surface that wires common controls to `TableState`:
/// - global filter input (`TableState.global_filter`)
/// - column visibility dropdown (`TableState.column_visibility`)
/// - selected row count (`TableState.row_selection`)
#[derive(Clone)]
pub struct DataTableToolbar<TData> {
    state: Model<TableState>,
    columns: Arc<[ColumnDef<TData>]>,
    column_label: Arc<dyn Fn(&ColumnDef<TData>) -> Arc<str>>,
    filter_placeholder: Arc<str>,
    column_filter: Option<ColumnId>,
    column_filter_placeholder: Arc<str>,
    column_filter_a11y_label: Arc<str>,
    faceted_filter: Option<FacetedFilterConfig>,
}

impl<TData> std::fmt::Debug for DataTableToolbar<TData> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("DataTableToolbar")
            .field("columns_len", &self.columns.len())
            .finish_non_exhaustive()
    }
}

impl<TData> DataTableToolbar<TData> {
    pub fn new(
        state: Model<TableState>,
        columns: impl Into<Arc<[ColumnDef<TData>]>>,
        column_label: impl Fn(&ColumnDef<TData>) -> Arc<str> + 'static,
    ) -> Self {
        Self {
            state,
            columns: columns.into(),
            column_label: Arc::new(column_label),
            filter_placeholder: Arc::from("Filter..."),
            column_filter: None,
            column_filter_placeholder: Arc::from("Filter..."),
            column_filter_a11y_label: Arc::from("Column filter"),
            faceted_filter: None,
        }
    }

    pub fn filter_placeholder(mut self, placeholder: impl Into<Arc<str>>) -> Self {
        self.filter_placeholder = placeholder.into();
        self
    }

    /// Adds a single per-column text filter input bound to `TableState.column_filters`.
    ///
    /// This is a v1 convenience surface intended to match the common TanStack/shadcn recipes
    /// where one “primary” column gets a dedicated filter input (e.g. “Filter emails...”).
    pub fn column_filter(mut self, column_id: impl Into<ColumnId>) -> Self {
        self.column_filter = Some(column_id.into());
        self
    }

    pub fn column_filter_placeholder(mut self, placeholder: impl Into<Arc<str>>) -> Self {
        self.column_filter_placeholder = placeholder.into();
        self
    }

    pub fn column_filter_a11y_label(mut self, label: impl Into<Arc<str>>) -> Self {
        self.column_filter_a11y_label = label.into();
        self
    }

    /// Adds a simple faceted multi-select filter control for a categorical column.
    ///
    /// The selected values are stored as a JSON array of strings in `TableState.column_filters`
    /// for the given `column_id`.
    pub fn faceted_filter(
        mut self,
        column_id: impl Into<ColumnId>,
        button_label: impl Into<Arc<str>>,
        options: impl Into<Arc<[Arc<str>]>>,
    ) -> Self {
        self.faceted_filter = Some(FacetedFilterConfig {
            column_id: column_id.into(),
            button_label: button_label.into(),
            options: options.into(),
        });
        self
    }

    pub fn into_element<H: UiHost>(self, cx: &mut ElementContext<'_, H>) -> AnyElement
    where
        TData: 'static,
    {
        let state_value = cx
            .watch_model(&self.state)
            .layout()
            .cloned()
            .unwrap_or_default();
        let state_revision = self.state.revision(&*cx.app).unwrap_or(0);

        let filter_model =
            cx.with_state(DataTableToolbarState::default, |st| st.filter_model.clone());
        let filter_model = match filter_model {
            Some(m) => m,
            None => {
                let initial = state_value
                    .global_filter
                    .as_ref()
                    .and_then(|v| v.as_str())
                    .unwrap_or_default()
                    .to_string();
                let m = cx.app.models_mut().insert(initial);
                let m_for_state = m.clone();
                cx.with_state(DataTableToolbarState::default, move |st| {
                    st.filter_model = Some(m_for_state);
                });
                m
            }
        };

        let column_filter_model = cx.with_state(DataTableToolbarState::default, |st| {
            st.column_filter_model.clone()
        });
        let column_filter_model = match (self.column_filter.as_ref(), column_filter_model) {
            (Some(_), Some(m)) => Some(m),
            (None, _) => None,
            (Some(column_id), None) => {
                let initial = state_value
                    .column_filters
                    .iter()
                    .find(|f| f.column.as_ref() == column_id.as_ref())
                    .and_then(|f| f.value.as_str())
                    .unwrap_or_default()
                    .to_string();
                let m = cx.app.models_mut().insert(initial);
                let m_for_state = m.clone();
                cx.with_state(DataTableToolbarState::default, move |st| {
                    st.column_filter_model = Some(m_for_state);
                });
                Some(m)
            }
        };

        let columns_open =
            cx.with_state(DataTableToolbarState::default, |st| st.columns_open.clone());
        let columns_open = match columns_open {
            Some(m) => m,
            None => {
                let m = cx.app.models_mut().insert(false);
                let m_for_state = m.clone();
                cx.with_state(DataTableToolbarState::default, move |st| {
                    st.columns_open = Some(m_for_state);
                });
                m
            }
        };

        let pinning_open =
            cx.with_state(DataTableToolbarState::default, |st| st.pinning_open.clone());
        let pinning_open = match pinning_open {
            Some(m) => m,
            None => {
                let m = cx.app.models_mut().insert(false);
                let m_for_state = m.clone();
                cx.with_state(DataTableToolbarState::default, move |st| {
                    st.pinning_open = Some(m_for_state);
                });
                m
            }
        };

        let faceted_open =
            cx.with_state(DataTableToolbarState::default, |st| st.faceted_open.clone());
        let faceted_open = match (self.faceted_filter.as_ref(), faceted_open) {
            (None, _) => None,
            (Some(_), Some(m)) => Some(m),
            (Some(_), None) => {
                let m = cx.app.models_mut().insert(false);
                let m_for_state = m.clone();
                cx.with_state(DataTableToolbarState::default, move |st| {
                    st.faceted_open = Some(m_for_state);
                });
                Some(m)
            }
        };

        let mut bindings = cx.with_state(DataTableToolbarState::default, |st| {
            st.column_visibility.clone()
        });
        if bindings.is_empty() {
            bindings = self
                .columns
                .iter()
                .filter(|c| c.enable_hiding)
                .map(|c| ColumnVisibilityBinding {
                    id: c.id.clone(),
                    model: cx
                        .app
                        .models_mut()
                        .insert(is_column_visible(&state_value, &c.id)),
                })
                .collect();
            let next = bindings.clone();
            cx.with_state(DataTableToolbarState::default, |st| {
                st.column_visibility = next
            });
        }

        let mut pinning_bindings = cx.with_state(DataTableToolbarState::default, |st| {
            st.column_pinning.clone()
        });
        if pinning_bindings.is_empty() {
            pinning_bindings = self
                .columns
                .iter()
                .filter(|c| c.enable_pinning)
                .map(|c| ColumnPinningBinding {
                    id: c.id.clone(),
                    model: cx
                        .app
                        .models_mut()
                        .insert(Some(pin_position_model_value(&state_value, &c.id))),
                })
                .collect();
            let next = pinning_bindings.clone();
            cx.with_state(DataTableToolbarState::default, |st| {
                st.column_pinning = next
            });
        }

        let mut faceted_items = cx.with_state(DataTableToolbarState::default, |st| {
            st.faceted_items.clone()
        });
        let faceted_column = cx.with_state(DataTableToolbarState::default, |st| {
            st.faceted_column.clone()
        });
        if let Some(cfg) = self.faceted_filter.as_ref() {
            let should_rebuild = faceted_column
                .as_ref()
                .is_none_or(|id| id.as_ref() != cfg.column_id.as_ref())
                || faceted_items.len() != cfg.options.len();
            if should_rebuild {
                let selected: std::collections::HashSet<Arc<str>> = state_value
                    .column_filters
                    .iter()
                    .find(|f| f.column.as_ref() == cfg.column_id.as_ref())
                    .map(|f| f.value.clone())
                    .and_then(|v| match v {
                        Value::String(s) => Some(vec![Arc::<str>::from(s)]),
                        Value::Array(items) => Some(
                            items
                                .into_iter()
                                .filter_map(|it| it.as_str().map(|s| Arc::<str>::from(s)))
                                .collect::<Vec<_>>(),
                        ),
                        _ => None,
                    })
                    .unwrap_or_default()
                    .into_iter()
                    .collect();

                faceted_items = cfg
                    .options
                    .iter()
                    .map(|opt| FacetedFilterItemBinding {
                        value: opt.clone(),
                        model: cx.app.models_mut().insert(selected.contains(opt)),
                    })
                    .collect();
                let next_items = faceted_items.clone();
                let next_column = cfg.column_id.clone();
                cx.with_state(DataTableToolbarState::default, move |st| {
                    st.faceted_items = next_items;
                    st.faceted_column = Some(next_column);
                });
            }
        }

        let last_synced_state_revision = cx.with_state(DataTableToolbarState::default, |st| {
            st.last_synced_state_revision
        });
        if last_synced_state_revision != Some(state_revision) {
            for binding in pinning_bindings.iter() {
                let desired = Some(pin_position_model_value(&state_value, &binding.id));
                let _ = cx
                    .app
                    .models_mut()
                    .update(&binding.model, |v| *v = desired.clone());
            }

            if let Some(cfg) = self.faceted_filter.as_ref() {
                let selected: std::collections::HashSet<Arc<str>> = state_value
                    .column_filters
                    .iter()
                    .find(|f| f.column.as_ref() == cfg.column_id.as_ref())
                    .map(|f| f.value.clone())
                    .and_then(|v| match v {
                        Value::String(s) => Some(vec![Arc::<str>::from(s)]),
                        Value::Array(items) => Some(
                            items
                                .into_iter()
                                .filter_map(|it| it.as_str().map(|s| Arc::<str>::from(s)))
                                .collect::<Vec<_>>(),
                        ),
                        _ => None,
                    })
                    .unwrap_or_default()
                    .into_iter()
                    .collect();

                for item in faceted_items.iter() {
                    let desired = selected.contains(&item.value);
                    let _ = cx.app.models_mut().update(&item.model, |v| *v = desired);
                }
            }

            cx.with_state(DataTableToolbarState::default, |st| {
                st.last_synced_state_revision = Some(state_revision);
            });
        }

        let filter_value = cx
            .watch_model(&filter_model)
            .layout()
            .cloned()
            .unwrap_or_default();
        sync_global_filter(&mut *cx.app, &self.state, &filter_value);

        if let (Some(column_id), Some(model)) =
            (self.column_filter.as_ref(), column_filter_model.as_ref())
        {
            let value = cx.watch_model(model).layout().cloned().unwrap_or_default();
            let _ = cx.app.models_mut().update(&self.state, |st| {
                let next = normalized_global_filter(&value);
                let existing = st
                    .column_filters
                    .iter()
                    .position(|f| f.column.as_ref() == column_id.as_ref());

                match (existing, next) {
                    (None, None) => {}
                    (Some(idx), None) => {
                        st.column_filters.remove(idx);
                        st.pagination.page_index = 0;
                    }
                    (Some(idx), Some(next)) => {
                        if st.column_filters[idx].value != next {
                            st.column_filters[idx].value = next;
                            st.pagination.page_index = 0;
                        }
                    }
                    (None, Some(next)) => {
                        st.column_filters
                            .push(fret_ui_headless::table::ColumnFilter {
                                column: column_id.clone(),
                                value: next,
                            });
                        st.pagination.page_index = 0;
                    }
                }
            });
        }

        if let Some(cfg) = self.faceted_filter.as_ref() {
            let selected: Vec<Arc<str>> = faceted_items
                .iter()
                .filter_map(|it| {
                    cx.watch_model(&it.model)
                        .layout()
                        .copied()
                        .unwrap_or(false)
                        .then(|| it.value.clone())
                })
                .collect();

            let next = if selected.is_empty() {
                None
            } else {
                Some(Value::Array(
                    selected
                        .iter()
                        .map(|v| Value::String(v.as_ref().to_string()))
                        .collect(),
                ))
            };

            let _ = cx.app.models_mut().update(&self.state, |st| {
                let existing = st
                    .column_filters
                    .iter()
                    .position(|f| f.column.as_ref() == cfg.column_id.as_ref());

                match (existing, next.clone()) {
                    (None, None) => {}
                    (Some(idx), None) => {
                        st.column_filters.remove(idx);
                        st.pagination.page_index = 0;
                    }
                    (Some(idx), Some(next)) => {
                        if st.column_filters[idx].value != next {
                            st.column_filters[idx].value = next;
                            st.pagination.page_index = 0;
                        }
                    }
                    (None, Some(next)) => {
                        st.column_filters
                            .push(fret_ui_headless::table::ColumnFilter {
                                column: cfg.column_id.clone(),
                                value: next,
                            });
                        st.pagination.page_index = 0;
                    }
                }
            });
        }

        let desired_visibility: HashMap<ColumnId, bool> = bindings
            .iter()
            .map(|b| {
                (
                    b.id.clone(),
                    cx.watch_model(&b.model).layout().copied().unwrap_or(true),
                )
            })
            .collect();
        sync_column_visibility(&mut *cx.app, &self.state, &desired_visibility);

        let desired_pinning: HashMap<ColumnId, Option<ColumnPinPosition>> = pinning_bindings
            .iter()
            .map(|b| {
                let raw = cx.watch_model(&b.model).layout().cloned().flatten();
                (b.id.clone(), pin_position_from_model_value(raw.as_ref()))
            })
            .collect();
        sync_column_pinning(&mut *cx.app, &self.state, &desired_pinning);

        let selected_count = state_value.row_selection.len();
        let theme = Theme::global(&*cx.app).clone();

        let column_label = Arc::clone(&self.column_label);
        let columns = Arc::clone(&self.columns);
        let visibility_items: Vec<DropdownMenuEntry> = bindings
            .iter()
            .filter_map(|b| {
                let col = columns.iter().find(|c| c.id.as_ref() == b.id.as_ref())?;
                let label = (column_label)(col);
                Some(DropdownMenuEntry::CheckboxItem(
                    DropdownMenuCheckboxItem::new(b.model.clone(), label),
                ))
            })
            .collect();

        let cols_menu = DropdownMenu::new(columns_open).into_element(
            cx,
            |cx| {
                Button::new("Columns")
                    .variant(ButtonVariant::Outline)
                    .size(ButtonSize::Sm)
                    .into_element(cx)
            },
            move |_cx| visibility_items.clone(),
        );

        let mut pin_items: Vec<DropdownMenuEntry> = pinning_bindings
            .iter()
            .filter_map(|b| {
                let col = columns.iter().find(|c| c.id.as_ref() == b.id.as_ref())?;
                let label = (column_label)(col);

                let radio_group = DropdownMenuRadioGroup::new(b.model.clone())
                    .item(
                        DropdownMenuRadioItemSpec::new("none", "Unpinned")
                            .a11y_label(Arc::<str>::from(format!("Pin {label} unpinned"))),
                    )
                    .item(
                        DropdownMenuRadioItemSpec::new("left", "Left")
                            .a11y_label(Arc::<str>::from(format!("Pin {label} left"))),
                    )
                    .item(
                        DropdownMenuRadioItemSpec::new("right", "Right")
                            .a11y_label(Arc::<str>::from(format!("Pin {label} right"))),
                    );

                Some(vec![
                    DropdownMenuEntry::Label(DropdownMenuLabel::new(label).inset(true)),
                    DropdownMenuEntry::RadioGroup(radio_group),
                    DropdownMenuEntry::Separator,
                ])
            })
            .flatten()
            .collect();
        if matches!(pin_items.last(), Some(DropdownMenuEntry::Separator)) {
            pin_items.pop();
        }

        let pin_menu = DropdownMenu::new(pinning_open).into_element(
            cx,
            |cx| {
                Button::new("Pin")
                    .variant(ButtonVariant::Outline)
                    .size(ButtonSize::Sm)
                    .into_element(cx)
            },
            move |_cx| pin_items.clone(),
        );

        let faceted_menu = self
            .faceted_filter
            .as_ref()
            .and_then(|cfg| faceted_open.clone().map(|open| (cfg.clone(), open)))
            .map(|(cfg, open)| {
                let button_label = cfg.button_label.clone();
                let entries: Vec<DropdownMenuEntry> = faceted_items
                    .iter()
                    .map(|it| {
                        DropdownMenuEntry::CheckboxItem(DropdownMenuCheckboxItem::new(
                            it.model.clone(),
                            it.value.clone(),
                        ))
                    })
                    .collect();

                DropdownMenu::new(open).into_element(
                    cx,
                    |cx| {
                        Button::new(button_label.clone())
                            .variant(ButtonVariant::Outline)
                            .size(ButtonSize::Sm)
                            .into_element(cx)
                    },
                    move |_cx| entries.clone(),
                )
            });

        let global_filter = Input::new(filter_model)
            .a11y_label("Global filter")
            .a11y_role(SemanticsRole::TextField)
            .placeholder(self.filter_placeholder.clone())
            .into_element(cx);

        let column_filter = column_filter_model.map(|m| {
            Input::new(m)
                .a11y_label(self.column_filter_a11y_label.clone())
                .a11y_role(SemanticsRole::TextField)
                .placeholder(self.column_filter_placeholder.clone())
                .into_element(cx)
        });

        let selected_text: Option<AnyElement> = (selected_count > 0).then(|| {
            let mut text =
                ui::raw_text(cx, Arc::from(format!("Selected: {selected_count}"))).nowrap();
            if let Some(color) = theme.color_by_key("muted-foreground") {
                text = text.text_color(ColorRef::Color(color));
            }
            text.into_element(cx)
        });

        hstack(
            cx,
            HStackProps::default()
                .layout(LayoutRefinement::default().w_full())
                .items_center()
                .gap_x(Space::N2),
            move |_cx| {
                let mut children = Vec::new();
                children.push(global_filter);
                if let Some(filter) = column_filter.clone() {
                    children.push(filter);
                }
                if let Some(menu) = faceted_menu.clone() {
                    children.push(menu);
                }
                children.push(cols_menu);
                children.push(pin_menu);
                if let Some(sel) = selected_text.clone() {
                    children.push(sel);
                }
                children
            },
        )
    }
}

#[derive(Default)]
struct DataTablePaginationState {
    page_size_open: Option<Model<bool>>,
    page_size_value: Option<Model<Option<Arc<str>>>>,
    last_synced_page_size: Option<usize>,
}

/// shadcn/ui `DataTable` pagination (recipe).
///
/// This is a v1 surface wired to `TableState.pagination`.
#[derive(Clone)]
pub struct DataTablePagination {
    state: Model<TableState>,
    output: Model<TableViewOutput>,
    page_sizes: Arc<[usize]>,
}

impl std::fmt::Debug for DataTablePagination {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("DataTablePagination")
            .field("page_sizes_len", &self.page_sizes.len())
            .finish_non_exhaustive()
    }
}

impl DataTablePagination {
    pub fn new(state: Model<TableState>, output: Model<TableViewOutput>) -> Self {
        Self {
            state,
            output,
            page_sizes: Arc::from([10usize, 20, 50, 100]),
        }
    }

    pub fn page_sizes(mut self, sizes: impl Into<Arc<[usize]>>) -> Self {
        self.page_sizes = sizes.into();
        self
    }

    pub fn into_element<H: UiHost>(self, cx: &mut ElementContext<'_, H>) -> AnyElement {
        let state_value = cx
            .watch_model(&self.state)
            .layout()
            .cloned()
            .unwrap_or_default();
        let output_value = cx
            .watch_model(&self.output)
            .layout()
            .cloned()
            .unwrap_or_default();

        let page_size_open = cx.with_state(DataTablePaginationState::default, |st| {
            st.page_size_open.clone()
        });
        let page_size_open = match page_size_open {
            Some(m) => m,
            None => {
                let m = cx.app.models_mut().insert(false);
                let m_for_state = m.clone();
                cx.with_state(DataTablePaginationState::default, move |st| {
                    st.page_size_open = Some(m_for_state);
                });
                m
            }
        };

        let page_size_value = cx.with_state(DataTablePaginationState::default, |st| {
            st.page_size_value.clone()
        });
        let page_size_value = match page_size_value {
            Some(m) => m,
            None => {
                let m = cx.app.models_mut().insert(None::<Arc<str>>);
                let m_for_state = m.clone();
                cx.with_state(DataTablePaginationState::default, move |st| {
                    st.page_size_value = Some(m_for_state);
                });
                m
            }
        };

        let current_size = state_value.pagination.page_size;
        let current_size_str: Arc<str> = Arc::from(current_size.to_string());

        let selected_value = cx
            .watch_model(&page_size_value)
            .layout()
            .cloned()
            .unwrap_or(None);

        let last_synced_page_size = cx.with_state(DataTablePaginationState::default, |st| {
            st.last_synced_page_size
        });

        // Treat `TableState.pagination.page_size` as the source of truth. The dropdown's internal
        // model must follow external updates (e.g. programmatic page size changes) and only drive
        // `TableState` when the user makes a new selection.
        let should_sync_to_state =
            selected_value.is_none() || last_synced_page_size != Some(current_size);
        if should_sync_to_state {
            let _ = cx
                .app
                .models_mut()
                .update(&page_size_value, |v| *v = Some(current_size_str.clone()));
            cx.with_state(DataTablePaginationState::default, |st| {
                st.last_synced_page_size = Some(current_size);
            });
        } else if let Some(sel) = selected_value {
            match sel.as_ref().parse::<usize>() {
                Ok(next) if next != current_size => {
                    let state = self.state.clone();
                    let _ = cx.app.models_mut().update(&state, |st| {
                        st.pagination.page_size = next;
                        st.pagination.page_index = 0;
                    });
                    cx.with_state(DataTablePaginationState::default, |st| {
                        st.last_synced_page_size = Some(next);
                    });
                }
                Ok(_) => {}
                Err(_) => {
                    let _ = cx
                        .app
                        .models_mut()
                        .update(&page_size_value, |v| *v = Some(current_size_str.clone()));
                    cx.with_state(DataTablePaginationState::default, |st| {
                        st.last_synced_page_size = Some(current_size);
                    });
                }
            }
        }

        let prev_enabled = output_value.pagination.can_prev;
        let next_enabled = output_value.pagination.can_next;
        let prev_on_activate: OnActivate = {
            let state = self.state.clone();
            Arc::new(move |host, _acx, _reason| {
                let _ = host.models_mut().update(&state, |st| {
                    st.pagination.page_index = st.pagination.page_index.saturating_sub(1);
                });
            })
        };
        let next_on_activate: OnActivate = {
            let state = self.state.clone();
            Arc::new(move |host, _acx, _reason| {
                let _ = host.models_mut().update(&state, |st| {
                    st.pagination.page_index = st.pagination.page_index.saturating_add(1);
                });
            })
        };

        let page_label: Arc<str> = if output_value.pagination.page_count == 0 {
            Arc::from("Page 0 / 0")
        } else {
            Arc::from(format!(
                "Page {} / {}",
                output_value.pagination.page_index + 1,
                output_value.pagination.page_count
            ))
        };

        let page_sizes = Arc::clone(&self.page_sizes);
        let page_size_menu = DropdownMenu::new(page_size_open).into_element(
            cx,
            |cx| {
                Button::new(Arc::from(format!("Rows: {current_size}")))
                    .variant(ButtonVariant::Outline)
                    .size(ButtonSize::Sm)
                    .into_element(cx)
            },
            move |_cx| {
                vec![DropdownMenuEntry::RadioGroup({
                    let mut group = DropdownMenuRadioGroup::new(page_size_value);
                    for size in page_sizes.iter().copied() {
                        let value: Arc<str> = Arc::from(size.to_string());
                        group = group.item(DropdownMenuRadioItemSpec::new(value.clone(), value));
                    }
                    group
                })]
            },
        );

        hstack(
            cx,
            HStackProps::default()
                .layout(LayoutRefinement::default().w_full())
                .items_center()
                .gap_x(Space::N2),
            move |cx| {
                vec![
                    Button::new("Prev")
                        .variant(ButtonVariant::Outline)
                        .size(ButtonSize::Sm)
                        .disabled(!prev_enabled)
                        .on_activate(prev_on_activate.clone())
                        .into_element(cx),
                    Button::new(page_label.clone())
                        .variant(ButtonVariant::Ghost)
                        .size(ButtonSize::Sm)
                        .into_element(cx),
                    Button::new("Next")
                        .variant(ButtonVariant::Outline)
                        .size(ButtonSize::Sm)
                        .disabled(!next_enabled)
                        .on_activate(next_on_activate.clone())
                        .into_element(cx),
                    page_size_menu,
                ]
            },
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[derive(Debug, Clone, PartialEq, Eq)]
    enum PageSizeAction {
        None,
        SyncToState,
        SetToUserSelection(usize),
    }

    fn reconcile_page_size(
        current_size: usize,
        selected_value: Option<&str>,
        last_synced: Option<usize>,
    ) -> PageSizeAction {
        if selected_value.is_none() || last_synced != Some(current_size) {
            return PageSizeAction::SyncToState;
        }

        let Some(sel) = selected_value else {
            return PageSizeAction::SyncToState;
        };

        match sel.parse::<usize>() {
            Ok(next) if next != current_size => PageSizeAction::SetToUserSelection(next),
            Ok(_) => PageSizeAction::None,
            Err(_) => PageSizeAction::SyncToState,
        }
    }

    #[test]
    fn pagination_page_size_is_controlled_by_state() {
        assert_eq!(
            reconcile_page_size(20, None, None),
            PageSizeAction::SyncToState
        );
        assert_eq!(
            reconcile_page_size(50, Some("10"), Some(10)),
            PageSizeAction::SyncToState,
            "external page_size change must win over stale dropdown model"
        );
    }

    #[test]
    fn pagination_page_size_accepts_user_selection() {
        assert_eq!(
            reconcile_page_size(20, Some("50"), Some(20)),
            PageSizeAction::SetToUserSelection(50)
        );
        assert_eq!(
            reconcile_page_size(20, Some("abc"), Some(20)),
            PageSizeAction::SyncToState
        );
    }

    #[test]
    fn global_filter_change_resets_page_index() {
        let mut st = TableState::default();
        st.pagination.page_index = 3;
        assert!(apply_global_filter_change(&mut st, "  foo  "));
        assert_eq!(st.pagination.page_index, 0);
        assert_eq!(
            st.global_filter.as_ref().and_then(|v| v.as_str()),
            Some("foo")
        );

        st.pagination.page_index = 2;
        assert!(!apply_global_filter_change(&mut st, "foo"));
        assert_eq!(st.pagination.page_index, 2, "no change should not reset");

        assert!(apply_global_filter_change(&mut st, "   "));
        assert_eq!(st.pagination.page_index, 0);
        assert!(st.global_filter.is_none());
    }

    #[test]
    fn column_visibility_change_resets_page_index() {
        let mut st = TableState::default();
        st.pagination.page_index = 5;

        let mut desired: HashMap<ColumnId, bool> = HashMap::new();
        desired.insert(Arc::from("a"), false);
        assert!(apply_column_visibility_change(&mut st, &desired));
        assert_eq!(st.pagination.page_index, 0);
        assert_eq!(st.column_visibility.get("a").copied(), Some(false));

        st.pagination.page_index = 3;
        assert!(!apply_column_visibility_change(&mut st, &desired));
        assert_eq!(st.pagination.page_index, 3);
    }
}
