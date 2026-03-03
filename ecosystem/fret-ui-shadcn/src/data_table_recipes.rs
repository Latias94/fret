use std::cell::Cell;
use std::collections::HashMap;
use std::rc::Rc;
use std::sync::Arc;

use fret_core::{Px, SemanticsRole};
use fret_runtime::Model;
use fret_ui::action::OnActivate;
use fret_ui::element::{AnyElement, LayoutQueryRegionProps, LayoutStyle, Length};
use fret_ui::elements::GlobalElementId;
use fret_ui::{ElementContext, Theme, UiHost};
use fret_ui_headless::table::{ColumnDef, ColumnId, ColumnPinPosition, TableState, pin_column};
use fret_ui_kit::declarative::model_watch::ModelWatchExt as _;
use fret_ui_kit::declarative::stack::{HStackProps, hstack};
use fret_ui_kit::declarative::table::TableViewOutput;
use fret_ui_kit::{ChromeRefinement, ColorRef, LayoutRefinement, Radius, Space, ui};
use serde_json::Value;

use crate::button::{Button, ButtonSize, ButtonVariant};
use crate::direction::{LayoutDirection, use_direction};
use crate::dropdown_menu::{
    DropdownMenu, DropdownMenuAlign, DropdownMenuCheckboxItem, DropdownMenuEntry,
    DropdownMenuLabel, DropdownMenuRadioGroup, DropdownMenuRadioItemSpec,
};
use crate::input::Input;
use crate::{
    CommandEntry, CommandGroup, CommandItem, CommandPalette, CommandSeparator, Popover,
    PopoverAlign, PopoverContent, PopoverTrigger,
};

fn sanitize_test_id_segment(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    for c in s.chars() {
        if c.is_ascii_alphanumeric() {
            out.push(c.to_ascii_lowercase());
        } else {
            out.push('-');
        }
    }
    out.trim_matches('-').to_string()
}

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

#[derive(Debug, Clone)]
pub struct DataTableFacetedFilterOption {
    pub value: Arc<str>,
    pub label: Arc<str>,
    pub icon: Option<fret_icons::IconId>,
}

impl DataTableFacetedFilterOption {
    pub fn new(value: impl Into<Arc<str>>, label: impl Into<Arc<str>>) -> Self {
        Self {
            value: value.into(),
            label: label.into(),
            icon: None,
        }
    }

    pub fn icon(mut self, icon: fret_icons::IconId) -> Self {
        self.icon = Some(icon);
        self
    }
}

#[derive(Clone)]
struct FacetedFilterConfig {
    column_id: ColumnId,
    button_label: Arc<str>,
    options: Arc<[DataTableFacetedFilterOption]>,
    counts: Option<Model<HashMap<Arc<str>, usize>>>,
}

struct FacetedFilterItemBinding {
    value: Arc<str>,
    label: Arc<str>,
    icon: Option<fret_icons::IconId>,
    model: Model<bool>,
}

impl Clone for FacetedFilterItemBinding {
    fn clone(&self) -> Self {
        Self {
            value: self.value.clone(),
            label: self.label.clone(),
            icon: self.icon.clone(),
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
    faceted_query: Option<Model<String>>,
    faceted_last_open: Option<bool>,
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
pub struct DataTableToolbar<TData> {
    state: Model<TableState>,
    columns: Arc<[ColumnDef<TData>]>,
    column_label: Arc<dyn Fn(&ColumnDef<TData>) -> Arc<str>>,
    show_global_filter: bool,
    filter_placeholder: Arc<str>,
    filter_layout: LayoutRefinement,
    column_filter: Option<ColumnId>,
    column_filter_placeholder: Arc<str>,
    column_filter_a11y_label: Arc<str>,
    show_columns_menu: bool,
    columns_button_label: Arc<str>,
    show_pinning_menu: bool,
    pinning_button_label: Arc<str>,
    show_selected_text: bool,
    faceted_filter: Option<FacetedFilterConfig>,
    faceted_selected_badges_query: DataTableToolbarResponsiveQuery,
    trailing: Vec<AnyElement>,
}

/// Responsive query source used by `DataTableToolbar` recipes.
///
/// Upstream shadcn/Tailwind `lg:*` variants are viewport-driven. In editor-grade layouts, some
/// toolbar behaviors may instead want to follow the width of the hosting panel/container.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DataTableToolbarResponsiveQuery {
    /// Match upstream Tailwind viewport breakpoint behavior.
    Viewport,
    /// Drive responsive variants from the toolbar's container-query region (ADR 0231).
    Container,
}

impl Default for DataTableToolbarResponsiveQuery {
    fn default() -> Self {
        Self::Viewport
    }
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
            show_global_filter: true,
            filter_placeholder: Arc::from("Filter..."),
            // Match upstream shadcn/ui tasks:
            // - `h-8` (32px)
            // - `w-[150px]` (recipe callers can override, e.g. `lg:w-[250px]` equivalents).
            //
            // This also avoids `Input`'s default `width: Fill` shrinking unexpectedly when used in
            // a shrink-wrapped toolbar row.
            filter_layout: LayoutRefinement::default()
                .h_px(Px(32.0))
                .w_px(Px(150.0))
                .flex_none(),
            column_filter: None,
            column_filter_placeholder: Arc::from("Filter..."),
            column_filter_a11y_label: Arc::from("Column filter"),
            show_columns_menu: true,
            columns_button_label: Arc::from("Columns"),
            show_pinning_menu: true,
            pinning_button_label: Arc::from("Pin"),
            show_selected_text: true,
            faceted_filter: None,
            faceted_selected_badges_query: DataTableToolbarResponsiveQuery::Viewport,
            trailing: Vec::new(),
        }
    }

    pub fn show_global_filter(mut self, show: bool) -> Self {
        self.show_global_filter = show;
        self
    }

    pub fn filter_placeholder(mut self, placeholder: impl Into<Arc<str>>) -> Self {
        self.filter_placeholder = placeholder.into();
        self
    }

    pub fn filter_layout(mut self, layout: LayoutRefinement) -> Self {
        self.filter_layout = self.filter_layout.merge(layout);
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

    pub fn show_columns_menu(mut self, show: bool) -> Self {
        self.show_columns_menu = show;
        self
    }

    pub fn columns_button_label(mut self, label: impl Into<Arc<str>>) -> Self {
        self.columns_button_label = label.into();
        self
    }

    pub fn show_pinning_menu(mut self, show: bool) -> Self {
        self.show_pinning_menu = show;
        self
    }

    pub fn pinning_button_label(mut self, label: impl Into<Arc<str>>) -> Self {
        self.pinning_button_label = label.into();
        self
    }

    pub fn show_selected_text(mut self, show: bool) -> Self {
        self.show_selected_text = show;
        self
    }

    pub fn trailing(mut self, trailing: impl IntoIterator<Item = AnyElement>) -> Self {
        self.trailing = trailing.into_iter().collect();
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
        let options: Arc<[Arc<str>]> = options.into();
        self.faceted_filter = Some(FacetedFilterConfig {
            column_id: column_id.into(),
            button_label: button_label.into(),
            options: Arc::from(
                options
                    .iter()
                    .map(|s| DataTableFacetedFilterOption::new(s.clone(), s.clone()))
                    .collect::<Vec<_>>(),
            ),
            counts: None,
        });
        self
    }

    pub fn faceted_filter_options(
        mut self,
        column_id: impl Into<ColumnId>,
        button_label: impl Into<Arc<str>>,
        options: impl Into<Arc<[DataTableFacetedFilterOption]>>,
    ) -> Self {
        self.faceted_filter = Some(FacetedFilterConfig {
            column_id: column_id.into(),
            button_label: button_label.into(),
            options: options.into(),
            counts: None,
        });
        self
    }

    pub fn faceted_filter_counts(mut self, counts: Model<HashMap<Arc<str>, usize>>) -> Self {
        if let Some(cfg) = self.faceted_filter.as_mut() {
            cfg.counts = Some(counts);
        }
        self
    }

    /// Controls how the faceted-filter trigger decides when to show label badges (shadcn uses
    /// `lg:hidden` / `hidden lg:flex`).
    ///
    /// Default: [`DataTableToolbarResponsiveQuery::Viewport`] (web parity).
    pub fn faceted_selected_badges_query(mut self, query: DataTableToolbarResponsiveQuery) -> Self {
        self.faceted_selected_badges_query = query;
        self
    }

    #[track_caller]
    pub fn into_element<H: UiHost>(self, cx: &mut ElementContext<'_, H>) -> AnyElement
    where
        TData: 'static,
    {
        let mut region_layout = LayoutStyle::default();
        region_layout.size.width = Length::Fill;
        let region_props = LayoutQueryRegionProps {
            layout: region_layout,
            name: None,
        };

        fret_ui_kit::declarative::container_query_region_with_id(
            cx,
            "shadcn.data_table.toolbar",
            region_props,
            move |cx, toolbar_region_id| {
                let dir = use_direction(cx, None);
                let is_rtl = dir == LayoutDirection::Rtl;
                let menu_align_inline_end = if is_rtl {
                    DropdownMenuAlign::Start
                } else {
                    DropdownMenuAlign::End
                };

                let state_value = cx
                    .watch_model(&self.state)
                    .layout()
                    .cloned()
                    .unwrap_or_default();
                let state_revision = self.state.revision(&*cx.app).unwrap_or(0);

                let filter_model =
                    cx.with_state(DataTableToolbarState::default, |st| st.filter_model.clone());
                let filter_model = match (self.show_global_filter, filter_model) {
                    (false, _) => None,
                    (true, Some(m)) => Some(m),
                    (true, None) => {
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
                        Some(m)
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

                let faceted_query = cx.with_state(DataTableToolbarState::default, |st| {
                    st.faceted_query.clone()
                });
                let faceted_query = match (self.faceted_filter.as_ref(), faceted_query) {
                    (None, _) => None,
                    (Some(_), Some(m)) => Some(m),
                    (Some(_), None) => {
                        let m = cx.app.models_mut().insert(String::new());
                        let m_for_state = m.clone();
                        cx.with_state(DataTableToolbarState::default, move |st| {
                            st.faceted_query = Some(m_for_state);
                        });
                        Some(m)
                    }
                };

                if let (Some(open), Some(query)) = (faceted_open.as_ref(), faceted_query.as_ref()) {
                    let open_now = cx.watch_model(open).layout().copied().unwrap_or(false);
                    let last_open =
                        cx.with_state(DataTableToolbarState::default, |st| st.faceted_last_open);
                    if last_open == Some(true) && !open_now {
                        let _ = cx.app.models_mut().update(query, |s| s.clear());
                    }
                    cx.with_state(DataTableToolbarState::default, move |st| {
                        st.faceted_last_open = Some(open_now);
                    });
                }

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
                    let column_changed = faceted_column
                        .as_ref()
                        .is_none_or(|id| id.as_ref() != cfg.column_id.as_ref());
                    let should_rebuild = column_changed || faceted_items.len() != cfg.options.len();
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
                                value: opt.value.clone(),
                                label: opt.label.clone(),
                                icon: opt.icon.clone(),
                                model: cx.app.models_mut().insert(selected.contains(&opt.value)),
                            })
                            .collect();
                        let next_items = faceted_items.clone();
                        let next_column = cfg.column_id.clone();
                        cx.with_state(DataTableToolbarState::default, move |st| {
                            st.faceted_items = next_items;
                            st.faceted_column = Some(next_column);
                        });

                        if column_changed {
                            if let Some(model) = faceted_query.as_ref() {
                                let _ = cx.app.models_mut().update(model, |s| s.clear());
                            }
                        }
                    }
                }

                let last_synced_state_revision = cx
                    .with_state(DataTableToolbarState::default, |st| {
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

                if let Some(filter_model) = filter_model.as_ref() {
                    let filter_value = cx
                        .watch_model(filter_model)
                        .layout()
                        .cloned()
                        .unwrap_or_default();
                    sync_global_filter(&mut *cx.app, &self.state, &filter_value);
                }

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

                let desired_pinning: HashMap<ColumnId, Option<ColumnPinPosition>> =
                    pinning_bindings
                        .iter()
                        .map(|b| {
                            let raw = cx.watch_model(&b.model).layout().cloned().flatten();
                            (b.id.clone(), pin_position_from_model_value(raw.as_ref()))
                        })
                        .collect();
                sync_column_pinning(&mut *cx.app, &self.state, &desired_pinning);

                let selected_count = state_value.row_selection.len();
                let theme = Theme::global(&*cx.app).snapshot();

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

                let columns_button_label = self.columns_button_label.clone();
                let cols_menu = self.show_columns_menu.then(|| {
                    DropdownMenu::new(columns_open.clone())
                        .align(menu_align_inline_end)
                        .into_element(
                            cx,
                            move |cx| {
                                Button::new(columns_button_label.clone())
                                    .variant(ButtonVariant::Outline)
                                    .size(ButtonSize::Sm)
                                    // Upstream shadcn: "Columns <ChevronDown />"
                                    .trailing_icon(fret_icons::IconId::new_static(
                                        "lucide.chevron-down",
                                    ))
                                    .into_element(cx)
                            },
                            move |_cx| {
                                let mut entries = Vec::new();
                                entries.push(DropdownMenuEntry::Label(
                                    DropdownMenuLabel::new("Toggle columns").inset(true),
                                ));
                                entries.push(DropdownMenuEntry::Separator);
                                entries.extend(visibility_items);
                                entries
                            },
                        )
                });

                let mut pin_items: Vec<DropdownMenuEntry> = pinning_bindings
                    .iter()
                    .filter_map(|b| {
                        let col = columns.iter().find(|c| c.id.as_ref() == b.id.as_ref())?;
                        let label = (column_label)(col);

                        let radio_group =
                            DropdownMenuRadioGroup::new(b.model.clone())
                                .item(
                                    DropdownMenuRadioItemSpec::new("none", "Unpinned").a11y_label(
                                        Arc::<str>::from(format!("Pin {label} unpinned")),
                                    ),
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

                let pinning_button_label = self.pinning_button_label.clone();
                let pin_menu = self.show_pinning_menu.then(|| {
                    DropdownMenu::new(pinning_open.clone())
                        .align(menu_align_inline_end)
                        .into_element(
                            cx,
                            move |cx| {
                                Button::new(pinning_button_label.clone())
                                    .variant(ButtonVariant::Outline)
                                    .size(ButtonSize::Sm)
                                    .into_element(cx)
                            },
                            move |_cx| pin_items,
                        )
                });

                let faceted_menu = self
            .faceted_filter
            .as_ref()
            .and_then(|cfg| {
                faceted_open
                    .clone()
                    .zip(faceted_query.clone())
                    .map(|(open, query)| (cfg.clone(), open, query))
            })
            .map(|(cfg, open, query)| {
                let faceted_items = faceted_items.clone();
                let button_label = cfg.button_label.clone();
                let selected_labels: Vec<Arc<str>> = faceted_items
                    .iter()
                    .filter_map(|it| {
                        cx.watch_model(&it.model)
                            .layout()
                            .copied()
                            .unwrap_or(false)
                            .then(|| it.label.clone())
                    })
                    .collect();
                let selected_count = selected_labels.len();

                let counts = cfg
                    .counts
                    .as_ref()
                    .and_then(|m| cx.watch_model(m).layout().cloned())
                    .unwrap_or_default();

                let col_seg = sanitize_test_id_segment(cfg.column_id.as_ref());
                let trigger_test_id =
                    Arc::<str>::from(format!("data-table-toolbar-faceted-{col_seg}-trigger"));
                let input_test_id =
                    Arc::<str>::from(format!("data-table-toolbar-faceted-{col_seg}-input"));
                let item_prefix =
                    Arc::<str>::from(format!("data-table-toolbar-faceted-{col_seg}-item-"));

                let input_id_cell: Rc<Cell<Option<GlobalElementId>>> = Rc::new(Cell::new(None));

                let trigger_button_label = button_label.clone();
                let content_button_label = button_label.clone();
                let trigger_test_id = trigger_test_id.clone();
                let trigger_selected_labels = selected_labels.clone();
                let trigger_selected_count = selected_count;
                let trigger_badges_query = self.faceted_selected_badges_query;
                let trigger_badges_query_region_id = toolbar_region_id;

                let faceted_items_for_content = faceted_items.clone();
                let query = query.clone();
                let input_test_id = input_test_id.clone();
                let item_prefix = item_prefix.clone();
                let counts_for_content = Arc::new(counts);
                let input_id_cell_for_content = input_id_cell.clone();

                Popover::new(open)
                    .align(PopoverAlign::Start)
                    .auto_focus(true)
                    .initial_focus_from_cell(input_id_cell)
                    .into_element(
                        cx,
                        move |cx| {
                             PopoverTrigger::new(
                                 Button::new(trigger_button_label.clone())
                                     .variant(ButtonVariant::Outline)
                                     .size(ButtonSize::Sm)
                                     .refine_style(
                                         ChromeRefinement::default().border_dash(
                                             fret_core::scene::DashPatternV1::new(
                                                 Px(4.0),
                                                 Px(4.0),
                                                 Px(0.0),
                                             ),
                                         ),
                                     )
                                     .test_id(trigger_test_id.clone())
                                     .children({
                                         let mut children = Vec::new();
                                        children.push(crate::icon::icon(
                                            cx,
                                            fret_icons::IconId::new_static("lucide.circle-plus"),
                                        ));
                                        children.push(
                                            ui::text(cx, trigger_button_label.clone())
                                                .into_element(cx),
                                        );
                                        if trigger_selected_count > 0 {
                                            children.push(
                                                crate::Separator::new()
                                                    .orientation(
                                                        crate::SeparatorOrientation::Vertical,
                                                    )
                                                    .refine_layout(
                                                        LayoutRefinement::default()
                                                            .h_px(Px(16.0))
                                                            .mx(Space::N2),
                                                    )
                                                    .into_element(cx),
                                            );

                                            let show_labels = fret_ui_kit::declarative::viewport_width_at_least(
                                                cx,
                                                fret_ui::Invalidation::Layout,
                                                fret_ui_kit::declarative::viewport_tailwind::LG,
                                                fret_ui_kit::declarative::ViewportQueryHysteresis::default(),
                                            );
                                            let show_labels = match trigger_badges_query {
                                                DataTableToolbarResponsiveQuery::Viewport => {
                                                    show_labels
                                                }
                                                DataTableToolbarResponsiveQuery::Container => {
                                                    // Container queries are frame-lagged. When the region width is
                                                    // temporarily unknown (e.g. in single-pass layout test harnesses),
                                                    // fall back to viewport-width behavior so we avoid branching on a
                                                    // missing measurement.
                                                    let default_when_unknown = cx
                                                        .environment_viewport_width(
                                                            fret_ui::Invalidation::Layout,
                                                        )
                                                        .0
                                                        >= fret_ui_kit::declarative::container_queries::tailwind::LG.0;
                                                    fret_ui_kit::declarative::container_width_at_least(
                                                        cx,
                                                        trigger_badges_query_region_id,
                                                        fret_ui::Invalidation::Layout,
                                                        default_when_unknown,
                                                        fret_ui_kit::declarative::container_queries::tailwind::LG,
                                                        fret_ui_kit::declarative::ContainerQueryHysteresis::default(),
                                                    )
                                                }
                                            };

                                            let badge_style = ChromeRefinement::default()
                                                .rounded(Radius::Sm)
                                                .px(Space::N1)
                                                .py(Space::N0p5);

                                            if !show_labels {
                                                let count_test_id = Arc::<str>::from(format!(
                                                    "data-table-toolbar-faceted-{col_seg}-badge-count"
                                                ));
                                                children.push(
                                                    crate::badge::Badge::new(Arc::<str>::from(
                                                        trigger_selected_count.to_string(),
                                                    ))
                                                    .variant(crate::badge::BadgeVariant::Secondary)
                                                    .test_id(count_test_id)
                                                    .refine_style(badge_style.clone())
                                                    .into_element(cx),
                                                );
                                            } else if trigger_selected_count > 2 {
                                                let summary_test_id = Arc::<str>::from(format!(
                                                    "data-table-toolbar-faceted-{col_seg}-badge-summary"
                                                ));
                                                children.push(
                                                    crate::badge::Badge::new(Arc::<str>::from(
                                                        format!("{trigger_selected_count} selected"),
                                                    ))
                                                    .variant(crate::badge::BadgeVariant::Secondary)
                                                    .test_id(summary_test_id)
                                                    .refine_style(badge_style.clone())
                                                    .into_element(cx),
                                                );
                                            } else {
                                                for (idx, label) in
                                                    trigger_selected_labels.iter().cloned().enumerate()
                                                {
                                                    let label_test_id = Arc::<str>::from(format!(
                                                        "data-table-toolbar-faceted-{col_seg}-badge-label-{idx}"
                                                    ));
                                                    children.push(
                                                        crate::badge::Badge::new(label)
                                                            .variant(
                                                                crate::badge::BadgeVariant::Secondary,
                                                            )
                                                            .test_id(label_test_id)
                                                            .refine_style(badge_style.clone())
                                                            .into_element(cx),
                                                    );
                                                }
                                            }
                                        }
                                        children
                                    })
                                    .into_element(cx),
                            )
                            .into_element(cx)
                        },
                        move |cx| {
                            let theme = Theme::global(&*cx.app).snapshot();
                            let transparent = fret_core::Color::TRANSPARENT;

                            let items: Vec<CommandEntry> = faceted_items_for_content
                                .iter()
                                .map(|it| {
                                    let checked = cx
                                        .watch_model(&it.model)
                                        .layout()
                                        .copied()
                                        .unwrap_or(false);
                                    let model_for_toggle = it.model.clone();
                                    let on_select_action: OnActivate =
                                        Arc::new(move |host, acx, _reason| {
                                            let _ =
                                                host.models_mut().update(&model_for_toggle, |v| {
                                                    *v = !*v;
                                                });
                                            host.notify(acx);
                                        });

                                    let maybe_icon = it.icon.as_ref().map(|icon_id| {
                                        let icon = crate::icon::icon(cx, icon_id.clone());
                                        cx.opacity(0.6, move |_cx| vec![icon])
                                    });

                                    let label = ui::raw_text(cx, it.label.clone())
                                        .nowrap()
                                        .into_element(cx);

                                    let check = crate::icon::icon(
                                        cx,
                                        fret_icons::IconId::new_static("lucide.check"),
                                    );
                                    let check = cx.opacity(if checked { 1.0 } else { 0.0 }, move |_cx| {
                                        vec![check]
                                    });
                                    let indicator = {
                                        let border = theme.color_token("input");
                                        let primary = theme.color_token("primary");

                                        let mut props = fret_ui::element::ContainerProps::default();
                                        props.layout = fret_ui_kit::declarative::style::layout_style(
                                            &theme,
                                            LayoutRefinement::default()
                                                .w_px(Px(16.0))
                                                .h_px(Px(16.0))
                                                .min_w_0()
                                                .min_h_0(),
                                        );
                                        props.border = fret_core::Edges::all(Px(1.0));
                                        props.border_color = Some(if checked { primary } else { border });
                                        props.corner_radii = fret_core::Corners::all(Px(4.0));
                                        props.background = checked.then_some(primary);

                                        let child = hstack(
                                            cx,
                                            HStackProps::default()
                                                .layout(
                                                    LayoutRefinement::default()
                                                        .w_full()
                                                        .h_full(),
                                                )
                                                .justify_center()
                                                .items_center(),
                                            move |_cx| vec![check],
                                        );
                                        cx.container(props, move |_cx| vec![child])
                                    };

                                    let count_el = counts_for_content
                                        .get(it.value.as_ref())
                                        .copied()
                                        .map(|n| {
                                            let fg_muted = theme.color_token("muted-foreground");
                                            let count = ui::text(cx, Arc::<str>::from(n.to_string()))
                                                .text_xs()
                                                .text_color(ColorRef::Color(fg_muted))
                                                .nowrap()
                                                .into_element(cx);

                                            hstack(
                                                cx,
                                                HStackProps::default()
                                                    .layout(
                                                        LayoutRefinement::default()
                                                            .w_px(Px(16.0))
                                                            .h_px(Px(16.0))
                                                            .min_w_0()
                                                            .min_h_0(),
                                                    )
                                                    .items_center()
                                                    .justify_center(),
                                                move |_cx| vec![count],
                                            )
                                        });

                                    let left = hstack(
                                        cx,
                                        HStackProps::default().gap_x(Space::N2).items_center(),
                                        move |_cx| {
                                            let mut out = Vec::new();
                                            out.push(indicator);
                                            if let Some(icon) = maybe_icon {
                                                out.push(icon);
                                            }
                                            out.push(label);
                                            out
                                        },
                                    );

                                    let row = hstack(
                                        cx,
                                        HStackProps::default()
                                            .layout(LayoutRefinement::default().w_full())
                                            .items_center()
                                            .justify_between(),
                                        move |_cx| {
                                            let mut out = vec![left];
                                            if let Some(count_el) = count_el {
                                                out.push(count_el);
                                            }
                                            out
                                        },
                                    );

                                    CommandItem::new(it.label.clone())
                                        .value(it.value.clone())
                                        .checkmark(checked)
                                        .on_select_action(on_select_action)
                                        .children([row])
                                        .into()
                                })
                                .collect();

                            let option_items = items
                                .into_iter()
                                .filter_map(|entry| match entry {
                                    CommandEntry::Item(item) => Some(item),
                                    _ => None,
                                })
                                .collect::<Vec<_>>();
                            let mut entries: Vec<CommandEntry> =
                                vec![CommandGroup::new(option_items).into()];

                            if selected_count > 0 {
                                let models_for_clear: Vec<Model<bool>> = faceted_items_for_content
                                    .iter()
                                    .map(|it| it.model.clone())
                                    .collect();
                                let query_for_clear = query.clone();
                                let on_clear: OnActivate = Arc::new(move |host, acx, _reason| {
                                    for model in models_for_clear.iter() {
                                        let _ = host.models_mut().update(model, |v| *v = false);
                                    }
                                    let _ =
                                        host.models_mut().update(&query_for_clear, |s| s.clear());
                                    host.notify(acx);
                                });

                                entries.push(CommandSeparator::new().into());
                                let clear_row = hstack(
                                    cx,
                                    HStackProps::default()
                                        .layout(LayoutRefinement::default().w_full())
                                        .items_center()
                                        .justify_center(),
                                    move |_cx| {
                                        vec![
                                            ui::text(_cx, Arc::<str>::from("Clear filters"))
                                                .into_element(_cx),
                                        ]
                                    },
                                );
                                entries.push(
                                    CommandGroup::new(vec![
                                        CommandItem::new("Clear filters")
                                            .value(Arc::<str>::from("__clear_filters"))
                                            .on_select_action(on_clear)
                                            .children([clear_row]),
                                    ])
                                    .into(),
                                );
                            }

                            let palette =
                                CommandPalette::new(query.clone(), Vec::<CommandItem>::new())
                                    .entries(entries)
                                    .a11y_label(Arc::<str>::from("Faceted filter"))
                                    .placeholder(content_button_label.clone())
                                    .empty_text("No results found.")
                                    .a11y_selected_mode(
                                        crate::command::CommandPaletteA11ySelectedMode::Checked,
                                    )
                                    .test_id_input(input_test_id.clone())
                                    .test_id_item_prefix(item_prefix.clone())
                                    .input_id_out_cell(input_id_cell_for_content.clone())
                                    .refine_style(
                                        ChromeRefinement::default()
                                            .radius(Px(0.0))
                                            .border_width(Px(0.0))
                                            .bg(ColorRef::Color(transparent))
                                            .border_color(ColorRef::Color(transparent)),
                                    );

                            PopoverContent::new(vec![palette.into_element(cx)])
                                .refine_layout(LayoutRefinement::default().w_px(Px(200.0)))
                                .refine_style(ChromeRefinement::default().p(Space::N0))
                                .a11y_label(content_button_label.clone())
                                .into_element(cx)
                        },
                    )
            });

                let filter_layout = self.filter_layout.clone();
                let global_filter = filter_model.as_ref().map(|m| {
                    Input::new(m.clone())
                        .a11y_label("Global filter")
                        .a11y_role(SemanticsRole::TextField)
                        .placeholder(self.filter_placeholder.clone())
                        .refine_layout(filter_layout.clone())
                        .into_element(cx)
                });

                let column_filter = column_filter_model.as_ref().map(|m| {
                    Input::new(m.clone())
                        .a11y_label(self.column_filter_a11y_label.clone())
                        .a11y_role(SemanticsRole::TextField)
                        .placeholder(self.column_filter_placeholder.clone())
                        .refine_layout(filter_layout.clone())
                        .into_element(cx)
                });

                let reset_filters = cx
                    .app
                    .models()
                    .read(&self.state, |st| {
                        st.global_filter.is_some() || !st.column_filters.is_empty()
                    })
                    .ok()
                    .unwrap_or(false);
                let reset_button = reset_filters.then(|| {
                    let state = self.state.clone();
                    let filter_model = filter_model.clone();
                    let column_filter_model = column_filter_model.clone();
                    let columns_open = columns_open.clone();
                    let pinning_open = pinning_open.clone();
                    let faceted_open = faceted_open.clone();
                    let faceted_query = faceted_query.clone();
                    let faceted_models: Vec<Model<bool>> =
                        faceted_items.iter().map(|it| it.model.clone()).collect();

                    let on_activate: OnActivate = Arc::new(move |host, acx, _reason| {
                        let _ = host.models_mut().update(&state, |st| {
                            st.global_filter = None;
                            st.column_filters.clear();
                            st.pagination.page_index = 0;
                        });

                        if let Some(filter_model) = filter_model.as_ref() {
                            let _ = host.models_mut().update(filter_model, |s| s.clear());
                        }
                        if let Some(model) = column_filter_model.as_ref() {
                            let _ = host.models_mut().update(model, |s| s.clear());
                        }
                        for model in faceted_models.iter() {
                            let _ = host.models_mut().update(model, |v| *v = false);
                        }
                        if let Some(model) = faceted_query.as_ref() {
                            let _ = host.models_mut().update(model, |s| s.clear());
                        }

                        let _ = host.models_mut().update(&columns_open, |v| *v = false);
                        let _ = host.models_mut().update(&pinning_open, |v| *v = false);
                        if let Some(open) = faceted_open.as_ref() {
                            let _ = host.models_mut().update(open, |v| *v = false);
                        }

                        host.notify(acx);
                    });

                    Button::new("Reset")
                        .variant(ButtonVariant::Ghost)
                        .size(ButtonSize::Sm)
                        .test_id("data-table-toolbar-reset-filters")
                        .children([
                            ui::text(cx, Arc::<str>::from("Reset")).into_element(cx),
                            crate::icon::icon(cx, fret_icons::IconId::new_static("lucide.x")),
                        ])
                        .on_activate(on_activate)
                        .into_element(cx)
                });

                let selected_text: Option<AnyElement> =
                    (self.show_selected_text && selected_count > 0).then(|| {
                        let mut text =
                            ui::raw_text(cx, Arc::from(format!("Selected: {selected_count}")))
                                .nowrap();
                        if let Some(color) = theme.color_by_key("muted-foreground") {
                            text = text.text_color(ColorRef::Color(color));
                        }
                        text.into_element(cx)
                    });

                let trailing = self.trailing;

                let left_group = hstack(
                    cx,
                    HStackProps::default().gap_x(Space::N2).items_center(),
                    move |_cx| {
                        let mut children = Vec::new();
                        if let Some(global_filter) = global_filter {
                            children.push(global_filter);
                        }
                        if let Some(filter) = column_filter {
                            children.push(filter);
                        }
                        if let Some(menu) = faceted_menu {
                            children.push(menu);
                        }
                        if let Some(btn) = reset_button {
                            children.push(btn);
                        }
                        if let Some(sel) = selected_text {
                            children.push(sel);
                        }
                        children
                    },
                );

                let right_group = hstack(
                    cx,
                    HStackProps::default().gap_x(Space::N2).items_center(),
                    move |_cx| {
                        let mut children = Vec::new();
                        if let Some(cols_menu) = cols_menu {
                            children.push(cols_menu);
                        }
                        if let Some(pin_menu) = pin_menu {
                            children.push(pin_menu);
                        }
                        children.extend(trailing);
                        children
                    },
                );

                vec![hstack(
                    cx,
                    HStackProps::default()
                        .layout(LayoutRefinement::default().w_full())
                        .items_center()
                        .justify_between()
                        .gap_x(Space::N2),
                    move |_cx| {
                        if is_rtl {
                            vec![right_group, left_group]
                        } else {
                            vec![left_group, right_group]
                        }
                    },
                )]
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
            page_sizes: Arc::from([10usize, 20, 25, 30, 40, 50]),
        }
    }

    pub fn page_sizes(mut self, sizes: impl Into<Arc<[usize]>>) -> Self {
        self.page_sizes = sizes.into();
        self
    }

    #[track_caller]
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
        let first_enabled = output_value.pagination.page_index > 0;
        let last_enabled = output_value.pagination.page_count > 0
            && output_value.pagination.page_index + 1 < output_value.pagination.page_count;

        let selected_count = state_value.row_selection.len();
        let filtered_count = output_value.filtered_row_count;
        let selected_label: Arc<str> = Arc::from(format!(
            "{selected_count} of {filtered_count} row(s) selected."
        ));

        let first_on_activate: OnActivate = {
            let state = self.state.clone();
            Arc::new(move |host, _acx, _reason| {
                let _ = host.models_mut().update(&state, |st| {
                    st.pagination.page_index = 0;
                });
            })
        };
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
        let last_on_activate: OnActivate = {
            let state = self.state.clone();
            let page_count = output_value.pagination.page_count;
            Arc::new(move |host, _acx, _reason| {
                let _ = host.models_mut().update(&state, |st| {
                    st.pagination.page_index = page_count.saturating_sub(1);
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
                Button::new(Arc::from(format!("Rows per page: {current_size}")))
                    .variant(ButtonVariant::Outline)
                    .size(ButtonSize::Sm)
                    .label_tabular_nums()
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
                let theme = Theme::global(&*cx.app);
                let muted_fg = theme.color_by_key("muted-foreground");
                let mut text = ui::text(cx, selected_label.clone())
                    .text_sm()
                    .tabular_nums()
                    .nowrap();
                if let Some(color) = muted_fg {
                    text = text.text_color(ColorRef::Color(color));
                }

                vec![
                    text.into_element(cx),
                    cx.spacer(fret_ui::element::SpacerProps::default()),
                    Button::new("Go to first page")
                        .variant(ButtonVariant::Outline)
                        .size(ButtonSize::Icon)
                        .disabled(!first_enabled)
                        .on_activate(first_on_activate.clone())
                        .children([crate::icon::icon(
                            cx,
                            fret_icons::IconId::new_static("lucide.chevrons-left"),
                        )])
                        .into_element(cx),
                    Button::new("Go to previous page")
                        .variant(ButtonVariant::Outline)
                        .size(ButtonSize::Icon)
                        .disabled(!prev_enabled)
                        .on_activate(prev_on_activate.clone())
                        .children([crate::icon::icon(
                            cx,
                            fret_icons::IconId::new_static("lucide.chevron-left"),
                        )])
                        .into_element(cx),
                    Button::new(page_label.clone())
                        .variant(ButtonVariant::Ghost)
                        .size(ButtonSize::Sm)
                        .label_tabular_nums()
                        .into_element(cx),
                    Button::new("Go to next page")
                        .variant(ButtonVariant::Outline)
                        .size(ButtonSize::Icon)
                        .disabled(!next_enabled)
                        .on_activate(next_on_activate.clone())
                        .children([crate::icon::icon(
                            cx,
                            fret_icons::IconId::new_static("lucide.chevron-right"),
                        )])
                        .into_element(cx),
                    Button::new("Go to last page")
                        .variant(ButtonVariant::Outline)
                        .size(ButtonSize::Icon)
                        .disabled(!last_enabled)
                        .on_activate(last_on_activate.clone())
                        .children([crate::icon::icon(
                            cx,
                            fret_icons::IconId::new_static("lucide.chevrons-right"),
                        )])
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
